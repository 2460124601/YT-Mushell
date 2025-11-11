(function() {
    const html = document.documentElement.innerHTML;
    const config = {
        api_key: html.match(/"INNERTUBE_API_KEY":"([^"]+)"/)?.[1] || '',
        client_name: html.match(/"INNERTUBE_CLIENT_NAME":"([^"]+)"/)?.[1] || 'WEB_REMIX',
        client_version: html.match(/"INNERTUBE_CLIENT_VERSION":"([^"]+)"/)?.[1] || '1.20250101.01.00',
        hl: html.match(/"hl":"([^"]+)"/)?.[1] || 'zh-TW',
        gl: html.match(/"gl":"([^"]+)"/)?.[1] || 'TW',
        headers: {
            cookie: document.cookie,
            'x-goog-authuser': '0',
            origin: 'https://music.youtube.com',
            referer: 'https://music.youtube.com/',
            'user-agent': navigator.userAgent,
            'x-youtube-client-name': '67',
            'x-youtube-client-version': html.match(/"INNERTUBE_CLIENT_VERSION":"([^"]+)"/)?.[1] || '1.20250101.01.00'
        }
    };
    console.log(JSON.stringify(config, null, 2));
    if (navigator.clipboard) navigator.clipboard.writeText(JSON.stringify(config, null, 2));
})();
