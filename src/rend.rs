use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};

// Hash the CSS bytes at compile time and reuse the digest when templating the head
// so the generated HTML gets a cache-busting query string whenever these files change.
// Browsers treat `?v=<hash>` as a new resource, which avoids manual version bumps.
static MAIN_CSS_HASH: Lazy<String> =
    Lazy::new(|| format!("{:x}", Sha256::digest(include_bytes!("../css/main.css"))));
static HACK_CSS_HASH: Lazy<String> =
    Lazy::new(|| format!("{:x}", Sha256::digest(include_bytes!("../web/hack.css"))));

pub struct Layout;
impl Layout {
    pub fn header() -> String {
        format!(
            r#"
        <!DOCTYPE html>
        <html lang="en-US">

        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link rel="stylesheet" href="/css/main.css?v={main_hash}" type="text/css">
            <link rel="stylesheet" href="/web/hack.css?v={hack_hash}">
            <title>enk junkyard</title>
            <link rel="apple-touch-icon" sizes="180x180" href="/favicon/apple-touch-icon.png">
            <link rel="icon" type="image/png" sizes="32x32" href="/favicon/favicon-32x32.png">
            <link rel="icon" type="image/png" sizes="16x16" href="/favicon/favicon-16x16.png">
            <link rel="manifest" href="/favicon/site.webmanifest">
            <link rel="mask-icon" href="/favicon/safari-pinned-tab.svg" color="\#5bbad5">
            <link rel="shortcut icon" href="/favicon/favicon.ico">
            <meta name="msapplication-TileColor" content="\#da532c">
            <meta name="msapplication-config" content="/favicon/browserconfig.xml">
            <meta name="theme-color" content="\#ffffff">
            <nav role="navigation" class="navigation">
                <a href="/">
                    <img class="logo" src="/favicon/favicon-32x32.png" alt="-__-"/>
                </a>
                <button class="theme-toggle" id="theme-toggle" aria-label="Toggle dark mode">
                    <span id="theme-icon">☀️</span>
                </button>
                <ul>
                    <li><a href="/pub/junkyard.html">junkyard</a></li>
                    <li><a href="/cv.html">cv</a></li>
                </ul>
            </nav>
        </head>"#,
            main_hash = &*MAIN_CSS_HASH,
            hack_hash = &*HACK_CSS_HASH
        )
    }

    pub fn body(body: &str) -> String {
        format!(
            r#"
            <body>
                <div id="page-container">
                <div id="content-wrap">
                <br />
                    {body}
                </div>
            </body>"#,
        )
    }

    pub fn footer() -> String {
        let github_run_id = match std::env::var("GITHUB_RUN_NUMBER") {
            Ok(v) => v,
            Err(_) => "no GITHUB_RUN_NUMBER variable is found".into(),
        };

        let github_sha = match std::env::var("GITHUB_SHA") {
            Ok(v) => v,
            Err(_) => "no GITHUB_SHA variable is found".into(),
        };

        format!(
            r#"
            <footer>
                <div class="footer">
                    <p>build {}: {}</p>
                    <p>updated: {}</p>
                </div>
            </footer>
            </div>
            <script type="module">
                import init from '/web/pkg/enkronio.js';
                init();
            </script>
            </html>"#,
            github_run_id,
            github_sha,
            chrono::offset::Utc::now(),
        )
    }
}
