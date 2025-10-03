#![warn(clippy::all, clippy::pedantic)]
use pulldown_cmark::{self, Options, Parser};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

mod rend;
use rend::Layout;
mod pdf;

const CONTENT_DIR: &str = "in";
const DOWNLOAD_DIR: &str = "download";
const PUBLIC_DIR: &str = "pub";

fn main() -> Result<(), anyhow::Error> {
    Site::build()?;

    Ok(())
}

struct Site;
impl Site {
    fn build() -> Result<(), anyhow::Error> {
        let mdfiles = WalkDir::new(CONTENT_DIR)
            .min_depth(1)
            .into_iter()
            .filter(|e| e.as_ref().unwrap().clone().into_path().is_file())
            .map(|e| {
                e.unwrap()
                    .into_path()
                    .strip_prefix(CONTENT_DIR)
                    .unwrap()
                    .to_owned()
            })
            .collect::<Vec<_>>();

        for mdfile in &mdfiles {
            let md = fs::read_to_string(PathBuf::from(CONTENT_DIR).join(mdfile))?;
            let parser = Parser::new_ext(&md, Options::all());

            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, parser);

            let mut html = String::new();
            html.push_str(&Layout::header());
            html.push_str(Layout::body(&body).as_str());
            html.push_str(&Layout::footer());

            fs::create_dir_all(PathBuf::from(PUBLIC_DIR).join("entries"))?;

            let mut htmlfile = match mdfile.to_str() {
                Some("index.md" | "cv.md") => PathBuf::from(mdfile),
                _ => {
                    if let Some(v) = mdfile.to_str().unwrap().split_once('-') {
                        PathBuf::from(PUBLIC_DIR).join(v.0)
                    } else {
                        PathBuf::from(PUBLIC_DIR).join(mdfile)
                    }
                }
            };

            htmlfile.set_extension("html");
            fs::write(&htmlfile, html)?;
        }

        fs::create_dir_all(DOWNLOAD_DIR)?;

        Self::export("cv.md", "sbelokon")?;
        Self::export("index.md", "cover")?;

        Ok(())
    }

    fn export<P: AsRef<Path>>(f_in: P, f_out: P) -> Result<(), anyhow::Error> {
        let md = fs::read_to_string(PathBuf::from(CONTENT_DIR).join(f_in))?;
        let mut pdf_path = PathBuf::from(DOWNLOAD_DIR).join(f_out);

        pdf_path.set_extension("pdf");
        let pdf_bytes = pdf::render(&md);
        fs::write(pdf_path, pdf_bytes)?;

        Ok(())
    }
}
