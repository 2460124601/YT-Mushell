use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, fs, path::Path, time::Duration};

#[derive(Deserialize, Debug)]
struct YtmConfig {
    api_key: String,
    client_name: String,
    client_version: String, 
    hl: String,
    gl: String,
    headers: HashMap<String, String>,
}
#[derive(Debug, Serialize, Clone)]
enum Kind { Song, Playlist, Other }
#[derive(Debug, Serialize, Clone)]
struct SimpleItem {
    kind: Kind,
    title: String,
    video_id: Option<String>,
    playlist_id: Option<String>,
}
impl SimpleItem {
    fn url(&self) -> Option<String> {
        if let Some(v) = &self.video_id {
            return Some(format!("https://www.youtube.com/watch?v={}", v));
        }
        if let Some(p) = &self.playlist_id {
            return Some(format!("https://www.youtube.com/playlist?list={}", p));
        }
        None
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let query = std::env::args().nth(1).unwrap_or_else(|| "jay chou".to_string());
    
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    
    let config_path = exe_dir.join("config").join("ytmusic.json");
    let conf = load_config(&config_path).context("Failed to load config/ytmusic.json")?;
    let client = build_client(&conf).await?;

    println!("searching:{}", query);
    let songs_params = "EgWKAQIIAWoQEAQQCRAQEAMQChAVEA4QEQ==";
    let data = ytm_search(&client, &conf, &query, Some(songs_params)).await?;
    let mut list = parse_search_items(&data);
    list.retain(|it| it.video_id.is_some() || it.playlist_id.is_some());
    if list.is_empty() {
        println!("Can't Play");
        return Ok(());
    }
    let max = 20usize.min(list.len());
    list.truncate(max);

  let socket = if cfg!(windows) {
    format!(r"\\.\pipe\mushell-{}-{}",
        std::process::id(),
        std::time::Instant::now().elapsed().as_nanos()
    )
} else {
    format!("/tmp/mushell-{}-{}.sock",
        std::process::id(),
        std::time::Instant::now().elapsed().as_nanos()
    )
};

let first_url = list[0].url().unwrap()
    .replace("https://music.youtube.com/watch?", "https://www.youtube.com/watch?")
    .replace("https://music.youtube.com/playlist?", "https://www.youtube.com/playlist?");


let mut mpv = spawn_mpv_with_ipc(&socket)?;


wait_ipc_ready(&socket, Duration::from_secs(6))?;

mpv_cmd(&socket, serde_json::json!({"command": ["loadfile", first_url, "replace"]}))?;

println!("▶ Playing:{}", list[0].title);
print_help();


    let mut idx: isize = 0;
    loop {
        if let Ok(true) = poll_key_event(Duration::from_millis(250)) {
            use crossterm::event::{read, Event, KeyCode, KeyEventKind};
            if let Event::Key(k) = read()? {
                if k.kind == KeyEventKind::Press {
                    match k.code {
                        KeyCode::Char('q') => { mpv_cmd(&socket, json!({"command":["quit"]}))?; break; }
                        KeyCode::Char(' ') => { mpv_cmd(&socket, json!({"command":["cycle","pause"]}))?; }
                        KeyCode::Char('+') => { mpv_cmd(&socket, json!({"command":["add","volume",5]}))?; }
                        KeyCode::Char('-') => { mpv_cmd(&socket, json!({"command":["add","volume",-5]}))?; }
                        KeyCode::Right      => { mpv_cmd(&socket, json!({"command":["seek",5,"relative","exact"]}))?; }
                        KeyCode::Left       => { mpv_cmd(&socket, json!({"command":["seek",-5,"relative","exact"]}))?; }
                        KeyCode::Char('r')  => { mpv_cmd(&socket, json!({"command":["seek",0,"absolute","exact"]}))?; }
                        KeyCode::Char('n')  => {
                            if (idx as usize) + 1 < list.len() {
                                idx += 1;
                                let url = list[idx as usize].url().unwrap();
                                println!("»» Next：{}", list[idx as usize].title);
                                mpv_cmd(&socket, json!({"command":["loadfile",url,"replace"]}))?;
                            }
                        }
                        KeyCode::Char('p')  => {
                            if idx > 0 {
                                idx -= 1;
                                let url = list[idx as usize].url().unwrap();
                                println!("⏮ Prev：{}", list[idx as usize].title);
                                mpv_cmd(&socket, json!({"command":["loadfile",url,"replace"]}))?;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let _ = mpv.wait();
    Ok(())
}

fn print_help() {
    println!("— Controls —");
    println!("Space  Play/Pause   +/- Volume±5");
    println!("→/←  Seek±5s  n/p Next/Prev");
    println!("r  Restart   q  Quit");
}

fn load_config<P: AsRef<Path>>(p: P) -> Result<YtmConfig> {
    let s = fs::read_to_string(p)?;
    Ok(serde_json::from_str(&s)?)
}
async fn build_client(conf: &YtmConfig) -> Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    for (k, v) in &conf.headers {
        if let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
            headers.insert(name, reqwest::header::HeaderValue::from_str(v)?);
        }
    }
    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .user_agent(conf.headers.get("user-agent").map(String::as_str).unwrap_or("Mozilla/5.0"))
        .gzip(true).brotli(true).deflate(true)
        .use_rustls_tls()
        .build()?)
}

/// innertube and search
async fn ytm_search(
    client: &reqwest::Client,
    conf: &YtmConfig,
    query: &str,
    params: Option<&str>,
) -> Result<serde_json::Value> {
    let url = format!(
        "https://music.youtube.com/youtubei/v1/search?prettyPrint=false&key={}",
        conf.api_key
    );
    let mut body = json!({
      "context": {"client": {
        "clientName": conf.client_name,
        "clientVersion": conf.client_version,
        "hl": conf.hl, "gl": conf.gl
      }},
      "query": query
    });
    if let Some(p) = params {
        body.as_object_mut().unwrap().insert("params".into(), json!(p));
    }
    let resp = client.post(&url).json(&body).send().await?.error_for_status()?;
    Ok(resp.json::<serde_json::Value>().await?)
}

fn parse_search_items(root: &serde_json::Value) -> Vec<SimpleItem> {
    let mut items = vec![];
    let shelves = root["contents"]["tabbedSearchResultsRenderer"]["tabs"][0]["tabRenderer"]
        ["content"]["sectionListRenderer"]["contents"]
        .as_array().cloned().unwrap_or_default();
    for c in shelves {
        if let Some(shelf) = c.get("musicShelfRenderer") {
            if let Some(arr) = shelf.get("contents").and_then(|x| x.as_array()) {
                for it in arr {
                    if let Some(m) = it.get("musicResponsiveListItemRenderer") {
                        if let Some(parsed) = parse_music_item(m) { items.push(parsed); }
                    }
                }
            }
        }
    }
    items
}
fn parse_music_item(m: &serde_json::Value) -> Option<SimpleItem> {
    let title = text_runs_join(&m["flexColumns"][0]["musicResponsiveListItemFlexColumnRenderer"]["text"]["runs"]);
    let (video_id, playlist_id, kind) = primary_id_and_kind(m);
    Some(SimpleItem { kind, title, video_id, playlist_id })
}
fn text_runs_join(runs: &serde_json::Value) -> String {
    runs.as_array().unwrap_or(&vec![])
        .iter().filter_map(|r| r.get("text").and_then(|t| t.as_str()))
        .collect::<Vec<_>>().join("")
}
fn primary_id_and_kind(m: &serde_json::Value) -> (Option<String>, Option<String>, Kind) {
    if let Some(overlay) = m.get("overlay")
        .and_then(|o| o.get("musicItemThumbnailOverlayRenderer"))
        .and_then(|o| o.get("content"))
        .and_then(|o| o.get("musicPlayButtonRenderer")) {
        if let Some(pid) = overlay.get("playNavigationEndpoint")
             .and_then(|e| e.get("watchPlaylistEndpoint")).and_then(|e| e.get("playlistId"))
             .and_then(|s| s.as_str()) {
            return (None, Some(pid.to_string()), Kind::Playlist);
        }
        if let Some(vid) = overlay.get("playNavigationEndpoint")
             .and_then(|e| e.get("watchEndpoint")).and_then(|e| e.get("videoId"))
             .and_then(|s| s.as_str()) {
            return (Some(vid.to_string()), None, Kind::Song);
        }
    }
    if let Some(vid) = m.get("navigationEndpoint")
        .and_then(|e| e.get("watchEndpoint")).and_then(|e| e.get("videoId"))
        .and_then(|s| s.as_str()) {
        return (Some(vid.to_string()), None, Kind::Song);
    }
    if let Some(pid) = m.get("navigationEndpoint")
        .and_then(|e| e.get("watchPlaylistEndpoint")).and_then(|e| e.get("playlistId"))
        .and_then(|s| s.as_str()) {
        return (None, Some(pid.to_string()), Kind::Playlist);
    }
    (None, None, Kind::Other)
}


fn spawn_mpv_with_ipc(socket: &str) -> Result<std::process::Child> {
    let mpv_filename = if cfg!(windows) { "mpv.com" } else { "mpv" };
    
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    
    let mpv_bin = exe_dir
        .join("third_party")
        .join("mpv")
        .join(mpv_filename);

    let ytdlp = exe_dir.join(if cfg!(windows) { "yt-dlp.exe" } else { "yt-dlp" });
    let ytdlp_opt = if ytdlp.exists() {
        Some(format!("--script-opts=ytdl_hook-ytdl_path={}", ytdlp.display()))
    } else { None };

    eprintln!("[mushell] mpv: {}", mpv_bin.display());
    eprintln!("[mushell] ipc: {}", socket);
    
    if !mpv_bin.exists() {
        anyhow::bail!("mpv 不存在於: {}", mpv_bin.display());
    }

    let mut cmd = std::process::Command::new(&mpv_bin);
    cmd.arg("--no-config")
        .arg("--no-video")
        .arg("--ytdl=yes")
        .arg("--ytdl-format=bestaudio/best")
        .arg("--idle=yes")
        .arg("--force-window=no")
        .arg("--keep-open=no")
        .arg("--input-default-bindings=no")
        .arg("--input-vo-keyboard=no")
        .arg("--term-osd=no")
        .arg("--msg-level=all=warn")
        .arg(format!("--input-ipc-server={}", socket));

    if let Some(opt) = ytdlp_opt { cmd.arg(opt); }

    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let child = cmd.spawn()?;
    eprintln!("[mushell] mpv Activated PID: {}", child.id());
    Ok(child)
}




fn wait_ipc_ready(socket: &str, timeout: std::time::Duration) -> Result<()> {
    let start = std::time::Instant::now();
    let mut attempts = 0;
    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("waiting mpv IPC Gateway Timeout（{}）,{} ", socket, attempts);
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            match std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(0)
                .open(socket) 
            {
                Ok(f) => {
                    drop(f);
                    eprintln!("[mushell] IPC ok（{},{:.1}s）", attempts + 1, start.elapsed().as_secs_f32());
                    return Ok(());
                }
                Err(e) => {
                    if attempts == 0 || attempts % 10 == 0 {
                        eprintln!("[mushell] waiting IPC... ({:.1}s, error: {})", start.elapsed().as_secs_f32(), e);
                    }
                }
            }
        }
        attempts += 1;
        std::thread::sleep(std::time::Duration::from_millis(120));
    }
}


fn mpv_cmd(socket: &str, cmd: serde_json::Value) -> Result<()> {
    use std::io::Write;
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let f = std::fs::OpenOptions::new()
            .read(true).write(true)
            .custom_flags(0) 
            .open(socket)?;
        let mut w = std::io::BufWriter::new(f);
        let line = serde_json::to_string(&cmd)? + "\n";
        w.write_all(line.as_bytes())?;
        w.flush()?;
    }
    Ok(())
}

fn poll_key_event(dur: Duration) -> Result<bool> {
    use crossterm::event::poll;
    Ok(poll(dur)?)
}
