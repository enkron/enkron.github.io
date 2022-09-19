#![warn(clippy::all, clippy::pedantic)]
use pulldown_cmark::{self, Options, Parser};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
use wkhtmltopdf::{Orientation, PdfApplication, Size};

mod rend;
use rend::Layout;

const CONTENT_DIR: &str = "content";
const DOWNLOAD_DIR: &str = "download";
const PUBLIC_DIR: &str = "pub";

fn main() -> Result<(), anyhow::Error> {
    Site::build()?;

    Ok(())
}

struct Site;
impl Site {
    fn build() -> Result<(), anyhow::Error> {
        let mdfiles: Vec<OsString> = WalkDir::new(CONTENT_DIR)
            .min_depth(1)
            .into_iter()
            .filter_map(|entry| Some(entry.ok()?.file_name().to_owned()))
            .collect();

        // WIP
        fs::create_dir_all(DOWNLOAD_DIR)?;
        let pdf_app = PdfApplication::new().expect("Failed to init PDF application");
        // WIP

        for mdfile in &mdfiles {
            let md = fs::read_to_string(Path::new(CONTENT_DIR).join(mdfile))?;
            let parser = Parser::new_ext(&md, Options::all());

            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, parser);

            let mut html = String::new();
            html.push_str(&Layout::header());
            html.push_str(Layout::body(&body).as_str());
            html.push_str(&Layout::footer());

            // WIP
            let mut pdfout = pdf_app
                .builder()
                .orientation(Orientation::Landscape)
                .margin(Size::Inches(2))
                .title("test_pdf_out_1")
                .build_from_html(&html)
                .expect("failed to build pdf");

            let mut pdf_path = PathBuf::from(DOWNLOAD_DIR).join(&mdfile);
            pdf_path.set_extension("pdf");

            pdfout.save(pdf_path).expect("failed to save foo.pdf");
            // WIP

            // the comparison is possible as `OsString` implements `PartialEq<&str>` trait
            if mdfile == "index.md" {
                let mut mdfile = PathBuf::from(mdfile);
                mdfile.set_extension("html");
                fs::write(&mdfile, html)?;
            } else {
                fs::create_dir_all(PUBLIC_DIR)?;

                let mut mdfile = PathBuf::from(PUBLIC_DIR).join(mdfile);
                mdfile.set_extension("html");
                fs::write(&mdfile, html)?;
            }
        }

        Ok(())
    }
}
