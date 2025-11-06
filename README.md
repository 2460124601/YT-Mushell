<div align="center">
  <img src="images/logo.png" alt="TY-mushell Logo" width="200"/>
</div>

# TY-Mushell

A lightweight YouTube Music command-line player powered by Rust and MPV.

## Features

- Search and play YouTube Music directly from terminal
- Keyboard controls (play/pause, volume, seek, skip)
- Fast and lightweight
- No browser required
- Portable executable

## Quick Start

```bash
# Search and play
TY-mushell.exe "taylor_swift"
TY-mushell.exe "再會啦！心愛的無緣的人"
TY-mushell.exe "hamilton_musical"
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `Space` | Play/Pause |
| `+` / `-` | Volume ±5 |
| `→` / `←` | Seek ±5s |
| `n` / `p` | Next/Previous track |
| `r` | Restart current track |
| `q` | Quit |

## Installation

### Option 1: Download Release

1. Download the latest release from [Releases](https://github.com/yourusername/TY-mushell/releases)
2. Extract to any folder
3. Configure `config/ytmusic.json` (see Configuration)
4. Run `TY-mushell.exe "your query"`

### Option 2: Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/TY-mushell.git
cd TY-mushell

# Build release
cargo build --release

# Run
.\target\release\TY-mushell.exe "song name"
```

## Configuration

### Get YouTube Music Cookie

1. Visit [YouTube Music](https://music.youtube.com) and sign in
2. Press `F12` to open DevTools
3. Go to `Console` tab
4. Copy and paste the content from `get-ytmusic-config.js`
5. Copy the output JSON to `config/ytmusic.json`

### Manual Configuration

Edit `config/ytmusic.json`:

```json
{
  "api_key": "YOUR_API_KEY",
  "client_name": "WEB_REMIX",
  "client_version": "1.20250101.01.00",
  "hl": "zh-TW",
  "gl": "TW",
  "headers": {
    "cookie": "YOUR_COOKIE_STRING",
    "user-agent": "Mozilla/5.0...",
    "origin": "https://music.youtube.com",
    "referer": "https://music.youtube.com/"
  }
}
```

**Note**: Cookie expires periodically (typically 1-2 weeks). Re-run the config script when needed.

## Requirements

- Windows 10/11 (Windows)
- macOS 10.15+ (macOS)
- Linux with ALSA/PulseAudio
- Internet connection
- Audio output device

## Project Structure

```
TY-mushell/
├── src/
│   └── main.rs              # Main application
├── config/
│   └── ytmusic.json         # User configuration
├── third_party/
│   └── mpv/                 # MPV player binaries
├── Cargo.toml               # Rust dependencies
└── README.md
```

## Dependencies

- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [tokio](https://tokio.rs/) - Async runtime
- [serde](https://serde.rs/) - Serialization
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal control
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling
- [MPV](https://mpv.io/) - Media player

## Troubleshooting

### Cannot search songs
- Check internet connection
- Verify `config/ytmusic.json` cookie is valid
- Cookie may have expired - get a new one

### Cannot play audio
- Ensure `third_party/mpv/mpv.com` exists
- Check audio device is working
- Verify firewall isn't blocking the app

### MPV IPC timeout
- Make sure you're using `mpv.com` not `mpv.exe` on Windows
- Restart the application
- Check if MPV can run independently

## Development

```bash
# Run in development mode
cargo run -- "search query"

# Build release
cargo build --release

# Run tests (if any)
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## Building Release Package

```bash
# Build executable
cargo build --release

# Create package
New-Item -ItemType Directory -Force -Path "release-package"
Copy-Item "target\release\TY-mushell.exe" "release-package\" -Force
Copy-Item -Recurse "config" "release-package\" -Force
Copy-Item -Recurse "third_party" "release-package\" -Force

# Create zip
Compress-Archive -Path release-package\* -DestinationPath TY-mushell-windows.zip
```

## License

This project is for educational purposes. Please respect YouTube's Terms of Service.

## Disclaimer

This tool is not affiliated with YouTube or Google. Use at your own risk.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Acknowledgments

- [MPV](https://mpv.io/) - Excellent media player
- [YouTube Music](https://music.youtube.com) - Music streaming service
- Rust community for amazing tools and libraries

---

Made with ☕, 🦀 and ❤️
