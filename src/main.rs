#![warn(clippy::all, clippy::pedantic)]
use chrono::{Datelike, Timelike};
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

/// Create a new entry file with a basic template including timestamp
fn create_entry_file(path: &Path, title: &str) -> Result<(), anyhow::Error> {
    // Generate timestamp in format: DD.ROMAN_MONTH.YYYY HH.MM UTC+OFFSET
    let now = chrono::Local::now();
    let day = now.day();
    let month_roman = month_to_roman(now.month());
    let year = now.year();
    let hour = now.hour();
    let minute = now.minute();

    // Get timezone offset in hours
    let offset = now.offset().local_minus_utc() / 3600;
    let offset_sign = if offset >= 0 { "+" } else { "" };

    let timestamp =
        format!("{day}.{month_roman}.{year} {hour:02}.{minute:02} UTC{offset_sign}{offset}");

    // Wrap timestamp in HTML span with CSS class for styling
    let content = format!("# {title}\n\n<span class=\"entry-timestamp\">{timestamp}</span>\n\n");

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
    let new_entry = format!("- {date_str}: [{title}](/pub/entries/{entry_number}.html)\n");

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

/// Generate navigation HTML for blog entry pagination
/// Returns HTML with links to previous/next entries if they exist
fn generate_entry_navigation(entry_number: u32) -> String {
    let prev_exists = PathBuf::from(ENTRIES_DIR)
        .join(format!("{}-", entry_number - 1))
        .parent()
        .and_then(|parent| {
            fs::read_dir(parent).ok().and_then(|entries| {
                entries
                    .filter_map(Result::ok)
                    .find(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .starts_with(&format!("{}-", entry_number - 1))
                    })
                    .map(|_| true)
            })
        })
        .unwrap_or(false);

    let next_exists = PathBuf::from(ENTRIES_DIR)
        .join(format!("{}-", entry_number + 1))
        .parent()
        .and_then(|parent| {
            fs::read_dir(parent).ok().and_then(|entries| {
                entries
                    .filter_map(Result::ok)
                    .find(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .starts_with(&format!("{}-", entry_number + 1))
                    })
                    .map(|_| true)
            })
        })
        .unwrap_or(false);

    let prev_link = if prev_exists {
        format!(
            "  <a href=\"/pub/entries/{}.html\" class=\"entry-nav-prev\">← Previous</a>\n",
            entry_number - 1
        )
    } else {
        String::from("  <span class=\"entry-nav-disabled\">← Previous</span>\n")
    };

    let next_link = if next_exists {
        format!(
            "  <a href=\"/pub/entries/{}.html\" class=\"entry-nav-next\">Next →</a>\n",
            entry_number + 1
        )
    } else {
        String::from("  <span class=\"entry-nav-disabled\">Next →</span>\n")
    };

    format!("<nav class=\"entry-nav\">\n{prev_link}{next_link}</nav>\n\n")
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

            // Add navigation for entry files
            if let Some(filename) = mdfile.file_name().and_then(|f| f.to_str()) {
                if let Some(dash_pos) = filename.find('-') {
                    if let Ok(entry_num) = filename[..dash_pos].parse::<u32>() {
                        // This is an entry file, prepend navigation
                        let navigation = generate_entry_navigation(entry_num);
                        body = navigation + &body;
                    }
                }
            }

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests `month_to_roman` conversion for all valid months (1-12).
    /// Verifies correct Roman numeral output for standard calendar months.
    #[test]
    fn test_month_to_roman_all_months() {
        assert_eq!(month_to_roman(1), "I");
        assert_eq!(month_to_roman(2), "II");
        assert_eq!(month_to_roman(3), "III");
        assert_eq!(month_to_roman(4), "IV");
        assert_eq!(month_to_roman(5), "V");
        assert_eq!(month_to_roman(6), "VI");
        assert_eq!(month_to_roman(7), "VII");
        assert_eq!(month_to_roman(8), "VIII");
        assert_eq!(month_to_roman(9), "IX");
        assert_eq!(month_to_roman(10), "X");
        assert_eq!(month_to_roman(11), "XI");
        assert_eq!(month_to_roman(12), "XII");
    }

    /// Tests `month_to_roman` with invalid month values.
    /// Verifies fallback to "?" for out-of-range inputs.
    #[test]
    fn test_month_to_roman_invalid() {
        assert_eq!(month_to_roman(0), "?");
        assert_eq!(month_to_roman(13), "?");
        assert_eq!(month_to_roman(100), "?");
    }

    /// Tests `generate_entry_filename` with simple alphanumeric title.
    /// Verifies basic slug generation: lowercase conversion and numbering.
    #[test]
    fn test_generate_entry_filename_simple() {
        let filename = generate_entry_filename(1, "Hello World");
        assert_eq!(filename, "1-hello-world.md");
    }

    /// Tests `generate_entry_filename` with special characters.
    /// Verifies non-alphanumeric characters are filtered out except dashes.
    #[test]
    fn test_generate_entry_filename_special_chars() {
        let filename = generate_entry_filename(5, "Hello, World! How's it going?");
        assert_eq!(filename, "5-hello-world-hows-it-going.md");
    }

    /// Tests `generate_entry_filename` with multiple consecutive spaces.
    /// Verifies consecutive dashes are collapsed into single dash.
    #[test]
    fn test_generate_entry_filename_multiple_spaces() {
        let filename = generate_entry_filename(10, "Multiple   Spaces    Here");
        assert_eq!(filename, "10-multiple-spaces-here.md");
    }

    /// Tests `generate_entry_filename` with mixed case and numbers.
    /// Verifies alphanumeric preservation and case normalization.
    #[test]
    fn test_generate_entry_filename_with_numbers() {
        let filename = generate_entry_filename(42, "IPv6 Setup 2024");
        assert_eq!(filename, "42-ipv6-setup-2024.md");
    }

    /// Tests `generate_entry_filename` with leading/trailing spaces.
    /// Verifies trimming behavior through dash filtering.
    #[test]
    fn test_generate_entry_filename_trim() {
        let filename = generate_entry_filename(3, "  Leading and Trailing  ");
        assert_eq!(filename, "3-leading-and-trailing.md");
    }

    /// Tests `generate_entry_filename` with only special characters.
    /// Verifies edge case handling when all characters are filtered.
    #[test]
    fn test_generate_entry_filename_only_special() {
        let filename = generate_entry_filename(7, "!@#$%^&*()");
        assert_eq!(filename, "7-.md");
    }

    /// Tests `generate_entry_filename` with dashes in title.
    /// Verifies existing dashes are preserved in slug.
    #[test]
    fn test_generate_entry_filename_with_dashes() {
        let filename = generate_entry_filename(8, "Pre-existing-dashes");
        assert_eq!(filename, "8-pre-existing-dashes.md");
    }

    /// Tests `generate_entry_filename` with Unicode characters.
    /// Verifies that Unicode alphanumeric characters are preserved.
    #[test]
    fn test_generate_entry_filename_unicode() {
        let filename = generate_entry_filename(9, "Café münchen");
        assert_eq!(filename, "9-café-münchen.md");
    }

    /// Tests `generate_entry_filename` with empty title.
    /// Verifies handling of edge case with no valid characters.
    #[test]
    fn test_generate_entry_filename_empty() {
        let filename = generate_entry_filename(1, "");
        assert_eq!(filename, "1-.md");
    }

    /// Tests `generate_entry_filename` with large entry number.
    /// Verifies no overflow or formatting issues with large numbers.
    #[test]
    fn test_generate_entry_filename_large_number() {
        let filename = generate_entry_filename(999_999, "Test Entry");
        assert_eq!(filename, "999999-test-entry.md");
    }
}
