#![warn(clippy::all, clippy::pedantic)]
use chrono;
use std::{env, fs, path::Path};

const SITE_ENTRY_POINT: &str = "index.html";
const WORKFLOW_TEST_VERSION_NUM: u16 = 7;

fn main() -> std::io::Result<()> {
    let github_sha = match env::var("GITHUB_SHA") {
        Ok(v) => v,
        Err(_) => "no SHA variable is found".into(),
    };

    let index = format!(
        "<!DOCTYPE html>\n \
        <html lang=\"en-US\">\n \
          <head>\n \
            <meta charset=\"utf-8\">\n \
            <title>workflow test v{}</title>\n \
            <style>\n \
              h1 {{\n \
                text-align: center;\n \
              }}\n \
            </style>\n \
          </head>\n \
          <body>\n \
            <h1>the page is under construction</h1>\n \
            <p>build: {}</p>\n \
            <p>updated: {}</p>\n \
          </body>\n \
        </html>",
        WORKFLOW_TEST_VERSION_NUM,
        github_sha,
        chrono::offset::Utc::now(),
    );

    if !Path::new(SITE_ENTRY_POINT).exists() {
        fs::write(SITE_ENTRY_POINT, &index)?;
    }

    Ok(())
}
