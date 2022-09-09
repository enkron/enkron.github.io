use chrono;
use std::{fs, path::Path};

const SITE_ENTRY_POINT: &str = "index.html";
const WORKFLOW_TEST_VERSION_NUM: u16 = 5;

fn main() -> std::io::Result<()> {
    let title = format!("workflow test v{}", WORKFLOW_TEST_VERSION_NUM);
    let body = format!("updated: {}", chrono::offset::Utc::now());
    let index = format!("{}\r\n{}\r\n", title, body);
    if !Path::new(SITE_ENTRY_POINT).exists() {
        fs::write(SITE_ENTRY_POINT, index)?;
    }

    Ok(())
}
