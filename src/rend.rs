pub struct Layout;
impl Layout {
    pub fn header() -> String {
        let github_ref_name = match std::env::var("GITHUB_REF_NAME") {
            Ok(v) => v,
            Err(_) => "no GITHUB_REF_NAME variable is found".into(),
        };

        format!(
            r#"<!DOCTYPE html>
            <html lang="en-US">
        
            <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link rel="stylesheet" type="text/css" href="/css/main.css" />
            <link rel="stylesheet" href="/css/styles.css">
            <link rel="stylesheet" href="/web/hack.css">
            <title>diy site. from {}</title>
            <nav>
            <a href="/">home</a>
            </nav>
            </head>"#,
            github_ref_name
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
            </div>
            <footer id="footer">
            <p>build {}: {}</p>
            <p>updated: {}</p>
            </footer>
            </div>
         
            </html>"#,
            github_run_id,
            github_sha,
            chrono::offset::Utc::now(),
        )
    }

    pub fn body(body: &str) -> String {
        format!(
            r#"
            <body>
            <div id="page-container">
            <div id="content-wrap">
            <br />
            {}
            </body>"#,
            body
        )
    }
}
