pub const HEADER: &str = r#"<!DOCTYPE html>
<html lang="en-US">

  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="stylesheet" type="text/css" href="css/main.css" />
    <title>workflow test v{}</title>
      <style>
        h1 {{
         text-align: center;
       }}
     </style>
  </head>

"#;

pub const FOOTER: &str = r#"
      </div>
      <footer id="footer">
        <p>build: {}</p>
        <p>updated: {}</p>
      </footer>
    </div>

</html>
"#;
//
//        github_run_id,
//        github_sha,
//        chrono::offset::Utc::now(),
//    );

pub fn render_body(body: &str) -> String {
    format!(
        r#"  <body>
    <div id="page-container">
      <div id="content-wrap">
        <h1>the page is under construction</h1>
    <nav>
        <a href="/">home</a>
    </nav>
    <br />
    {}
  </body>"#,
        body
    )
}
