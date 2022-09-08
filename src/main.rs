use std::{fs, path::Path};

const SITE_ENTRY_POINT: &str = "index.html";

fn main() -> std::io::Result<()> {
    if !Path::new(SITE_ENTRY_POINT).exists() {
        fs::write(SITE_ENTRY_POINT, "workflow test v2")?;
    }

    Ok(())
}
