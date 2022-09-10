#![warn(clippy::all, clippy::pedantic)]
use pulldown_cmark;
use std::{fs, path::Path};

mod render;
use render::Layout;

const CONTENT_DIR: &str = "content";
const PUBLIC_DIR: &str = "public";

fn main() -> Result<(), anyhow::Error> {
    Site::build(CONTENT_DIR, PUBLIC_DIR)?;

    Ok(())
}

struct Site;

impl Site {
    fn build(content: &str, public: &str) -> Result<Self, anyhow::Error> {
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
            let mut html = Layout::header();
            let markdown = fs::read_to_string(&file)?;
            let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());

            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, parser);

            html.push_str(Layout::body(&body).as_str());
            html.push_str(&Layout::footer());

            let html_file = file.replace(content, public).replace(".md", ".html");
            fs::write(&html_file, html)?;

            html_files.push(html_file);
        }

        Self::index(html_files, public)?;

        Ok(Self)
    }

    fn index(files: Vec<String>, public: &str) -> Result<(), anyhow::Error> {
        let mut idx = Layout::header();
        let body = files
            .into_iter()
            .map(|file| {
                let file = file.trim_start_matches(public);
                let title = file.trim_start_matches("/").trim_end_matches(".html");
                format!(r#"<a href="{}{}">{}</a>"#, public, file, title)
            })
            .collect::<Vec<String>>()
            .join("<br />\n");

        idx.push_str(Layout::body(&body).as_str());
        idx.push_str(&Layout::footer());
        fs::write(Path::new("index.html"), idx)?;

        Ok(())
    }
}
