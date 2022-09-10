pub fn header() -> String {
    let github_run_id = match std::env::var("GITHUB_RUN_NUMBER") {
        Ok(v) => v,
        Err(_) => "no GITHUB_RUN_NUMBER variable is found".into(),
    };

    format!(
        r#"<!DOCTYPE html>
        <html lang="en-US">
    
        <head>
          <meta charset="utf-8">
          <meta name="viewport" content="width=device-width, initial-scale=1">
          <link rel="stylesheet" type="text/css" href="/css/main.css" />
          <title>workflow test v{}</title>
            <style>
              h1 {{
               text-align: center;
             }}
           </style>
        </head>"#,
        github_run_id,
    )
}

pub fn footer() -> String {
    let github_sha = match std::env::var("GITHUB_SHA") {
        Ok(v) => v,
        Err(_) => "no GITHUB_SHA variable is found".into(),
    };

    format!(
        r#"
        </div>
        <footer id="footer">
          <p>build: {}</p>
          <p>updated: {}</p>
        </footer>
      </div>
     
      </html>"#,
        github_sha,
        chrono::offset::Utc::now(),
    )
}

pub fn body(body: &str) -> String {
    format!(
        r#"  <body>
    <div id="page-container">
      <div id="content-wrap">
    <nav>
        <a href="/">home</a>
    </nav>
    <br />
    {}
  </body>"#,
        body
    )
}
