#![warn(clippy::all, clippy::pedantic)]
use chrono::Datelike;
use clap::{Parser, Subcommand};
use pulldown_cmark::{self, Options, Parser as MdParser};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

mod rend;
use rend::Layout;
mod pdf;
mod work_period;

const CONTENT_DIR: &str = "in";
const DOWNLOAD_DIR: &str = "download";
const PUBLIC_DIR: &str = "pub";
const ENTRIES_DIR: &str = "in/entries";
const JUNKYARD_FILE: &str = "in/junkyard.md";

#[derive(Parser)]
#[command(name = "enkronio")]
#[command(about = "Static site generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new blog entry
    Add {
        /// Title of the new entry
        title: String,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { title }) => {
            add_entry(&title)?;
        }
        None => {
            // Default behavior: build the site
            Site::build()?;
        }
    }

    Ok(())
}

/// Add a new blog entry
fn add_entry(title: &str) -> Result<(), anyhow::Error> {
    // Find the next entry number
    let next_number = find_next_entry_number()?;

    // Generate filename from title
    let filename = generate_entry_filename(next_number, title);

    // Create the new entry file
    let entry_path = PathBuf::from(ENTRIES_DIR).join(&filename);
    create_entry_file(&entry_path, title)?;

    // Update junkyard.md with the new entry link
    update_junkyard(next_number, title)?;

    println!("Created new entry: {}", entry_path.display());
    println!("Updated {JUNKYARD_FILE}");

    Ok(())
}

/// Find the next entry number by scanning existing entries
fn find_next_entry_number() -> Result<u32, anyhow::Error> {
    let entries = fs::read_dir(ENTRIES_DIR)?;
    let mut max_number = 0;

    for entry in entries {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        // Parse number from filename like "3-ipv6-local-networking.md"
        if let Some(dash_pos) = filename_str.find('-') {
            if let Ok(num) = filename_str[..dash_pos].parse::<u32>() {
                max_number = max_number.max(num);
            }
        }
    }

    Ok(max_number + 1)
}

/// Generate filename from title: convert to lowercase, replace spaces with dashes
fn generate_entry_filename(number: u32, title: &str) -> String {
    let slug = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_whitespace() { '-' } else { c })
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    // Remove consecutive dashes
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    format!("{number}-{slug}.md")
}

/// Create a new entry file with a basic template
fn create_entry_file(path: &Path, title: &str) -> Result<(), anyhow::Error> {
    let content = format!("# {title}\n\n");
    fs::write(path, content)?;
    Ok(())
}

/// Update junkyard.md with a new entry link
fn update_junkyard(entry_number: u32, title: &str) -> Result<(), anyhow::Error> {
    let junkyard_content = fs::read_to_string(JUNKYARD_FILE)?;

    // Generate date in Roman numeral format (like "24.V.2024")
    let now = chrono::Local::now();
    let day = now.day();
    let month_roman = month_to_roman(now.month());
    let year = now.year();
    let date_str = format!("{day}.{month_roman}.{year}");

    // Generate the new entry line
    let new_entry = format!(
        "- {date_str}: [{title}](/pub/entries/{entry_number}.html)\n"
    );

    // Find the "## recent posts" section and insert after it
    let lines: Vec<&str> = junkyard_content.lines().collect();
    let mut new_content = String::new();
    let mut inserted = false;

    for (i, line) in lines.iter().enumerate() {
        new_content.push_str(line);
        new_content.push('\n');

        // Insert after "## recent posts" header
        if !inserted && line.trim() == "## recent posts" {
            // Skip empty line if present
            if i + 1 < lines.len() && lines[i + 1].trim().is_empty() {
                new_content.push('\n');
                new_content.push_str(&new_entry);
                inserted = true;
            } else {
                new_content.push_str(&new_entry);
                inserted = true;
            }
        }
    }

    // If we didn't find the section, append to the end
    if !inserted {
        new_content.push_str("\n## recent posts\n\n");
        new_content.push_str(&new_entry);
    }

    fs::write(JUNKYARD_FILE, new_content)?;
    Ok(())
}

/// Convert month number to Roman numeral
fn month_to_roman(month: u32) -> &'static str {
    match month {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        6 => "VI",
        7 => "VII",
        8 => "VIII",
        9 => "IX",
        10 => "X",
        11 => "XI",
        12 => "XII",
        _ => "?",
    }
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
            let md = work_period::process(&md);
            let parser = MdParser::new_ext(&md, Options::all());

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
        let md = work_period::process(&md);
        let mut pdf_path = PathBuf::from(DOWNLOAD_DIR).join(f_out);

        pdf_path.set_extension("pdf");
        let pdf_bytes = pdf::render(&md);
        fs::write(pdf_path, pdf_bytes)?;

        Ok(())
    }
}
