#![warn(clippy::all, clippy::pedantic)]
//use chrono;
//use std::{env, fs, path::Path};
//
//const SITE_ENTRY_POINT: &str = "index.html";
//
//fn main() -> std::io::Result<()> {
//    let github_sha = match env::var("GITHUB_SHA") {
//        Ok(v) => v,
//        Err(_) => "no GITHUB_SHA variable is found".into(),
//    };
//
//    let github_run_id = match env::var("GITHUB_RUN_NUMBER") {
//        Ok(v) => v,
//        Err(_) => "no GITHUB_RUN_NUMBER variable is found".into(),
//    };
//
//    let index = format!(
//        "<!DOCTYPE html>\n \
//        <html lang=\"en-US\">\n \
//          <head>\n \
//            <link rel=\"stylesheet\" type=\"text/css\" href=\"css/main.css\" />\n \
//            <meta charset=\"utf-8\">\n \
//            <title>workflow test v{}</title>\n \
//            <style>\n \
//              h1 {{\n \
//                text-align: center;\n \
//              }}\n \
//            </style>\n \
//          </head>\n \
//          <body>\n \
//            <div id=\"page-container\">\n \
//              <div id=\"content-wrap\">\n \
//                <h1>the page is under construction</h1>\n \
//              </div>
//              <footer id=\"footer\">\n \
//                <p>build: {}</p>\n \
//                <p>updated: {}</p>\n \
//              </footer>\n \
//            </div>
//          </body>\n \
//        </html>",
//        github_run_id,
//        github_sha,
//        chrono::offset::Utc::now(),
//    );
//
//    if !Path::new(SITE_ENTRY_POINT).exists() {
//        fs::write(SITE_ENTRY_POINT, &index)?;
//    }
//
//    Ok(())
//}
use pulldown_cmark;
use std::{fs, path::Path};

mod templates;
const CONTENT_DIR: &str = "content";
const PUBLIC_DIR: &str = "public";

fn main() -> Result<(), anyhow::Error> {
    build(CONTENT_DIR, PUBLIC_DIR)?;

    Ok(())
}

fn build(content: &str, public: &str) -> Result<(), anyhow::Error> {
    if !Path::new(public).exists() {
        fs::create_dir_all(public)?;
    }

    let markdown_files: Vec<String> = walkdir::WalkDir::new(content)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().display().to_string().ends_with(".md"))
        .map(|e| e.path().display().to_string())
        .collect();
    let mut html_files = Vec::with_capacity(markdown_files.len());

    for file in &markdown_files {
        let mut html = templates::HEADER.to_owned();
        let markdown = fs::read_to_string(&file)?;
        let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());

        let mut body = String::new();
        pulldown_cmark::html::push_html(&mut body, parser);

        html.push_str(templates::render_body(&body).as_str());
        html.push_str(templates::FOOTER);

        let html_file = file.replace(content, public).replace(".md", ".html");
        let folder = Path::new(&html_file).parent().unwrap();
        let _ = fs::create_dir_all(folder);
        fs::write(&html_file, html)?;

        html_files.push(html_file);
    }

    write_index(html_files, public)?;

    Ok(())
}

fn write_index(files: Vec<String>, public: &str) -> Result<(), anyhow::Error> {
    let mut html = templates::HEADER.to_owned();
    let body = files
        .into_iter()
        .map(|file| {
            let file = file.trim_start_matches(public);
            let title = file.trim_start_matches("/").trim_end_matches(".html");
            format!(r#"<a href="{}">{}</a>"#, file, title)
        })
        .collect::<Vec<String>>()
        .join("<br />\n");

    html.push_str(templates::render_body(&body).as_str());
    html.push_str(templates::FOOTER);

    let index_path = Path::new(&public).join("index.html");
    fs::write(index_path, html)?;

    Ok(())
}
