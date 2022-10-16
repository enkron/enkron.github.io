#![warn(clippy::all, clippy::pedantic)]
use pulldown_cmark::{self, Options, Parser};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
use wkhtmltopdf::{Orientation, PageSize, PdfApplication, Size};

mod rend;
use rend::Layout;

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
        let mdfiles: Vec<OsString> = WalkDir::new(CONTENT_DIR)
            .min_depth(1)
            .into_iter()
            .filter_map(|entry| Some(entry.ok()?.file_name().to_owned()))
            .collect();

        for mdfile in &mdfiles {
            let md = fs::read_to_string(Path::new(CONTENT_DIR).join(mdfile))?;
            let parser = Parser::new_ext(&md, Options::all());

            let mut body = String::new();
            pulldown_cmark::html::push_html(&mut body, parser);

            let mut html = String::new();
            html.push_str(&Layout::header());
            html.push_str(Layout::body(&body).as_str());
            html.push_str(&Layout::footer());

            let mdfile = mdfile.to_str().unwrap(); // Try to convert OsString to &str
            match mdfile {
                "index.md" => {
                    let mut mdfile = PathBuf::from(mdfile);
                    mdfile.set_extension("html");
                    fs::write(&mdfile, html)?;
                }

                _ => {
                    fs::create_dir_all(PUBLIC_DIR)?;
                    let mut mdfile = PathBuf::from(PUBLIC_DIR).join(mdfile);
                    mdfile.set_extension("html");
                    fs::write(&mdfile, html)?;
                }
            }
        }

        fs::create_dir_all(DOWNLOAD_DIR)?;
        let pdf_app = PdfApplication::new()?;

        Self::export("cv.md", "sbelokon", &pdf_app)?;
        Self::export("index.md", "cover", &pdf_app)?;

        Ok(())
    }

    fn export<P: AsRef<Path>>(
        f_in: P,
        f_out: P,
        pdf_app: &PdfApplication,
    ) -> Result<(), anyhow::Error> {
        let md = fs::read_to_string(Path::new(CONTENT_DIR).join(f_in))?;
        let parser = Parser::new_ext(&md, Options::all());

        let mut body = String::new();
        pulldown_cmark::html::push_html(&mut body, parser);
        let mut html = String::new();
        html.push_str(Layout::body(&body).as_str());

        let mut pdf_builder = pdf_app.builder();
        pdf_builder
            .page_size(PageSize::A4)
            .orientation(Orientation::Portrait)
            .margin(Size::Millimeters(10))
            .title("sbelokon");

        let mut pdf = pdf_builder.build_from_html(&html)?;
        let mut pdf_path = PathBuf::from(DOWNLOAD_DIR).join(f_out);

        pdf_path.set_extension("pdf");
        pdf.save(pdf_path)?;

        Ok(())
    }
}
