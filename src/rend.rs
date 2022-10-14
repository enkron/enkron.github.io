pub struct Layout;
impl Layout {
    pub fn header() -> String {
        let github_ref_name = match std::env::var("GITHUB_REF_NAME") {
            Ok(v) => v,
            Err(_) => "no GITHUB_REF_NAME variable is found".into(),
        };

        format!(
            r#"
<!DOCTYPE html>
<html lang="en-US">

<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="/css/main.css" type="text/css">
<link rel="stylesheet" href="/web/hack.css">
<title>enkron's junkyard (built from {} branch)</title>
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
    <img class="logo" src="/favicon/favicon-32x32.png" alt="-__-"/>
    <ul>
        <li><a href="/">home</a></li>
        <li><a href="/pub/cv.html">cv</a></li>
    </ul>
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
