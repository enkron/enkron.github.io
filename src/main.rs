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
mod crypto;
mod pdf;
mod work_period;

const CONTENT_DIR: &str = "in";
const DOWNLOAD_DIR: &str = "download";
const PUBLIC_DIR: &str = "pub";
const ENTRIES_DIR: &str = "in/entries";
const SHADOW_ENTRIES_DIR: &str = "in/entries/shadow";
const JUNKYARD_FILE: &str = "in/junkyard.md";
const LOCK_KEY_ENV: &str = "ENKRONIO_LOCK_KEY";
const LOCKFILE_PATH: &str = ".enkronio-locks";

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
        /// Create as shadow entry (private, not listed in junkyard)
        #[arg(long)]
        shadow: bool,
    },
    /// Lock (encrypt) or unlock (decrypt) a markdown file
    Lock {
        /// Path to the markdown file to encrypt/decrypt
        path: String,
        /// Decrypt the file instead of encrypting it
        #[arg(short, long)]
        unlock: bool,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { title, shadow }) => {
            add_entry(&title, shadow)?;
        }
        Some(Commands::Lock { path, unlock }) => {
            lock_file(&path, unlock)?;
        }
        None => {
            // Default behavior: build the site
            Site::build()?;
        }
    }

    Ok(())
}

/// Get encryption passphrase from environment variable or interactive prompt.
///
/// Security priorities:
/// 1. Environment variable `ENKRONIO_LOCK_KEY` (for CI/CD and scripting)
/// 2. Interactive secure prompt (for manual operations, no echo)
///
/// CLI flags are NOT supported for security reasons (visible in process list).
fn get_passphrase(prompt_message: &str) -> Result<String, anyhow::Error> {
    // Try environment variable first
    if let Ok(passphrase) = std::env::var(LOCK_KEY_ENV) {
        if !passphrase.is_empty() {
            return Ok(passphrase);
        }
    }

    // Fall back to interactive prompt (secure input, no terminal echo)
    println!("{prompt_message}");
    let passphrase = rpassword::prompt_password("Passphrase: ")?;

    if passphrase.is_empty() {
        return Err(anyhow::anyhow!("Passphrase cannot be empty"));
    }

    // Optional: passphrase strength validation
    if passphrase.len() < 12 {
        eprintln!("Warning: Passphrase is shorter than recommended minimum (12 characters)");
        eprintln!("For better security, use a longer passphrase (16+ characters recommended)");
    }

    Ok(passphrase)
}

/// Lock (encrypt) or unlock (decrypt) a markdown file.
///
/// When encrypting: reads .md file, encrypts it, saves as .enc, removes .md
/// When decrypting: reads .enc file, decrypts it, saves as .md, removes .enc
fn lock_file(path: &str, unlock: bool) -> Result<(), anyhow::Error> {
    let file_path = PathBuf::from(path);

    if !file_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path));
    }

    if unlock {
        // Decrypt: .enc -> .md
        if !path.to_lowercase().ends_with(".enc") {
            return Err(anyhow::anyhow!("Unlock requires .enc file, got: {}", path));
        }

        eprintln!("Unlocking: {path}");

        // Read encrypted content
        let encrypted_bytes = fs::read(&file_path)?;

        // Get passphrase
        let passphrase = get_passphrase("Enter passphrase to decrypt file:")?;

        // Decrypt
        let plaintext = crypto::decrypt(&encrypted_bytes, &passphrase)?;

        // Write decrypted content (.enc -> .md)
        let output_path = file_path.with_extension("md");
        fs::write(&output_path, plaintext)?;

        // Remove encrypted file
        fs::remove_file(&file_path)?;

        println!("Unlocked: {} -> {}", path, output_path.display());
        println!("File decrypted successfully!");
    } else {
        // Encrypt: .md -> .enc
        if !path.to_lowercase().ends_with(".md") {
            return Err(anyhow::anyhow!("Lock requires .md file, got: {}", path));
        }

        eprintln!("Locking: {path}");

        // Read plaintext content
        let plaintext = fs::read_to_string(&file_path)?;

        // Get passphrase
        let passphrase = get_passphrase("Enter passphrase to encrypt file:")?;

        // Encrypt
        let encrypted_bytes = crypto::encrypt(&plaintext, &passphrase)?;

        // Write encrypted content (.md -> .enc)
        let mut output_path = file_path.clone();
        output_path.set_extension("enc");
        fs::write(&output_path, encrypted_bytes)?;

        // Remove plaintext file
        fs::remove_file(&file_path)?;

        println!("Locked: {} -> {}", path, output_path.display());
        println!("File encrypted successfully!");

        // Track in lockfile if it's an entry
        if path.contains("/entries/") {
            let is_shadow = path.contains("/entries/shadow/");
            // Try to extract entry number
            if let Some(filename) = file_path.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    if let Some(dash_pos) = filename_str.find('-') {
                        if let Ok(entry_num) = filename_str[..dash_pos].parse::<u32>() {
                            track_locked_entry(entry_num, is_shadow)?;
                            eprintln!("Tracked in lockfile: entry {entry_num}");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Add a new blog entry
fn add_entry(title: &str, shadow: bool) -> Result<(), anyhow::Error> {
    // Determine directory based on shadow flag
    let entries_dir = if shadow {
        SHADOW_ENTRIES_DIR
    } else {
        ENTRIES_DIR
    };

    // Find the next entry number in the appropriate directory
    let next_number = find_next_entry_number(entries_dir)?;

    // Generate filename from title
    let filename = generate_entry_filename(next_number, title);
    let entry_path = PathBuf::from(entries_dir).join(&filename);

    // Create the entry file with template content
    create_entry_file(&entry_path, title)?;

    println!("Created new entry: {}", entry_path.display());

    // Update junkyard for non-shadow entries
    if shadow {
        println!("Shadow entry created (private, not listed in junkyard)");
        println!("To encrypt: cargo run -- lock {}", entry_path.display());
    } else {
        update_junkyard(next_number, title)?;
        println!("Updated {JUNKYARD_FILE}");
        println!("To encrypt: cargo run -- lock {}", entry_path.display());
    }

    Ok(())
}

/// Lockfile format for tracking encrypted entries
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Lockfile {
    version: String,
    locked_entries: Vec<LockedEntry>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LockedEntry {
    number: u32,
    shadow: bool,
    created: String,
}

/// Read the lockfile (.enkronio-locks) or return empty default
fn read_lockfile() -> Result<Lockfile, anyhow::Error> {
    if !Path::new(LOCKFILE_PATH).exists() {
        return Ok(Lockfile {
            version: "1.0".to_string(),
            locked_entries: vec![],
        });
    }

    let content = fs::read_to_string(LOCKFILE_PATH)?;
    let lockfile: Lockfile = serde_json::from_str(&content)?;
    Ok(lockfile)
}

/// Write the lockfile (.enkronio-locks)
fn write_lockfile(lockfile: &Lockfile) -> Result<(), anyhow::Error> {
    let content = serde_json::to_string_pretty(lockfile)?;
    fs::write(LOCKFILE_PATH, content)?;
    Ok(())
}

/// Track a locked entry in the lockfile
fn track_locked_entry(entry_number: u32, shadow: bool) -> Result<(), anyhow::Error> {
    let mut lockfile = read_lockfile()?;

    // Add new locked entry
    lockfile.locked_entries.push(LockedEntry {
        number: entry_number,
        shadow,
        created: chrono::Utc::now().to_rfc3339(),
    });

    write_lockfile(&lockfile)?;
    Ok(())
}

/// Check if an entry is locked
#[allow(dead_code)] // Reserved for future navigation features
fn is_entry_locked(entry_number: u32, shadow: bool) -> bool {
    let lockfile = read_lockfile().ok();
    if let Some(lockfile) = lockfile {
        lockfile
            .locked_entries
            .iter()
            .any(|e| e.number == entry_number && e.shadow == shadow)
    } else {
        false
    }
}

/// Find the next entry number by scanning existing entries in specified directory
fn find_next_entry_number(entries_dir: &str) -> Result<u32, anyhow::Error> {
    // Create directory if it doesn't exist
    fs::create_dir_all(entries_dir)?;

    let entries = fs::read_dir(entries_dir)?;
    let mut max_number = 0;

    for entry in entries {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        // Parse number from filename like "3-ipv6-local-networking.md" or "5-title.enc"
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
/// For shadow entries, uses /priv/entries/ URL prefix and checks shadow directory
fn generate_entry_navigation(entry_number: u32, is_shadow: bool) -> String {
    let entries_dir = if is_shadow {
        SHADOW_ENTRIES_DIR
    } else {
        ENTRIES_DIR
    };
    let url_prefix = if is_shadow {
        "/priv/entries/"
    } else {
        "/pub/entries/"
    };

    let prev_exists = PathBuf::from(entries_dir)
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

    let next_exists = PathBuf::from(entries_dir)
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
            "  <a href=\"{}{}.html\" class=\"entry-nav-prev\">← Previous</a>\n",
            url_prefix,
            entry_number - 1
        )
    } else {
        String::new()
    };

    let next_link = if next_exists {
        format!(
            "  <a href=\"{}{}.html\" class=\"entry-nav-next\">Next →</a>\n",
            url_prefix,
            entry_number + 1
        )
    } else {
        String::new()
    };

    // Only render nav if at least one link exists
    if prev_exists || next_exists {
        format!("<nav class=\"entry-nav\">\n{prev_link}{next_link}</nav>\n\n")
    } else {
        String::new()
    }
}

/// Generate a locked HTML stub with embedded encrypted content for browser decryption.
///
/// This function creates the "🔒 Locked Entry" interface that users see before decryption.
/// The encrypted markdown is embedded as base64 in a data attribute for WASM decryption.
///
/// This version directly embeds already-encrypted bytes without requiring the passphrase.
/// The browser WASM module will handle decryption when the user enters their passphrase.
fn generate_locked_stub_from_encrypted(encrypted_b64: &str) -> String {
    // Generate the locked stub HTML
    let stub = format!(
        r#"
<div id="locked-entry-container" class="locked-entry" data-encrypted="{encrypted_b64}">
  <div id="lock-banner" class="lock-banner">
    <span class="lock-icon">🔒</span>
    <h2>This entry is encrypted</h2>
    <p>Enter the passphrase to decrypt and view the content.</p>
  </div>

  <div id="unlock-interface" class="unlock-interface">
    <input type="password"
           id="passphrase-input"
           placeholder="Enter passphrase"
           autocomplete="off"
           aria-label="Passphrase"
           class="passphrase-input">
    <button id="decrypt-button" class="decrypt-button">🔓 Unlock</button>

    <div id="error-message" class="error-message hidden" role="alert"></div>
    <div id="decrypt-status" class="decrypt-status hidden" aria-live="polite">
      Decrypting... (this may take a few seconds)
    </div>
  </div>

  <div id="decrypted-content" class="decrypted-content hidden"></div>
</div>
"#,
    );

    stub
}

/// Generate a 404 error page with full layout
fn generate_404_html() -> String {
    let body = r#"
<div class="error-page">
    <img src="/favicon/android-chrome-192x192.png" alt="Logo" class="error-logo"/>
    <h1>404</h1>
    <p>Page not found</p>
    <nav class="error-nav">
        <a href="/">Home</a>
        <a href="/pub/junkyard.html">Junkyard</a>
    </nav>
</div>
"#;

    let mut html = String::new();
    html.push_str(&Layout::header());
    html.push_str(&Layout::body(body));
    html.push_str(&Layout::footer());
    html
}

/// Generate a directory index stub that redirects to a target URL
/// If `redirect_to` is None, displays a 404-style message
fn generate_directory_index_html(redirect_to: Option<&str>) -> String {
    if let Some(url) = redirect_to {
        // Generate redirect stub
        format!(
            r#"<!DOCTYPE html>
<html lang="en-US">
<head>
    <meta charset="utf-8">
    <meta http-equiv="refresh" content="0;url={url}">
    <link rel="canonical" href="{url}">
    <title>Redirecting...</title>
</head>
<body>
    <p>Redirecting to <a href="{url}">{url}</a>...</p>
</body>
</html>"#
        )
    } else {
        // Generate 404-style stub
        let body = r#"
<div class="error-page">
    <img src="/favicon/android-chrome-192x192.png" alt="Logo" class="error-logo"/>
    <h1>404</h1>
    <p>This directory is not browsable</p>
    <nav class="error-nav">
        <a href="/">Home</a>
        <a href="/pub/junkyard.html">Junkyard</a>
    </nav>
</div>
"#;

        let mut html = String::new();
        html.push_str(&Layout::header());
        html.push_str(&Layout::body(body));
        html.push_str(&Layout::footer());
        html
    }
}

/// Generate 404 page and directory index stubs to prevent directory listings
fn generate_error_pages() -> Result<(), anyhow::Error> {
    // Generate 404 page at root
    let html_404 = generate_404_html();
    fs::write("404.html", html_404)?;
    eprintln!("Generated: 404.html");

    // Generate directory index stubs to prevent directory listings
    let pub_index = generate_directory_index_html(Some("/pub/junkyard.html"));
    fs::write("pub/index.html", pub_index)?;
    eprintln!("Generated: pub/index.html (redirects to junkyard)");

    let pub_entries_index = generate_directory_index_html(Some("/pub/junkyard.html"));
    fs::write("pub/entries/index.html", pub_entries_index)?;
    eprintln!("Generated: pub/entries/index.html (redirects to junkyard)");

    let priv_entries_index = generate_directory_index_html(None);
    fs::write("priv/entries/index.html", priv_entries_index)?;
    eprintln!("Generated: priv/entries/index.html (not browsable)");

    let download_index = generate_directory_index_html(Some("/"));
    fs::write("download/index.html", download_index)?;
    eprintln!("Generated: download/index.html (redirects to home)");

    Ok(())
}

struct Site;
impl Site {
    fn build() -> Result<(), anyhow::Error> {
        // Collect all files from content directory (.md and .enc only)
        let all_files = WalkDir::new(CONTENT_DIR)
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
            .filter(|path| {
                // Only process .md and .enc files
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| {
                        ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("enc")
                    })
            })
            .collect::<Vec<_>>();

        fs::create_dir_all(PathBuf::from(PUBLIC_DIR).join("entries"))?;
        fs::create_dir_all("priv/entries")?;

        for mdfile in &all_files {
            let file_path = PathBuf::from(CONTENT_DIR).join(mdfile);
            let filename = mdfile.to_str().unwrap();

            // Check if this is an encrypted file (.enc)
            let is_locked = mdfile
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("enc"));

            // For locked entries, we keep them encrypted and generate a stub
            // For regular entries, we process markdown normally
            let encrypted_bytes = if is_locked {
                Some(fs::read(&file_path)?)
            } else {
                None
            };

            // Read markdown content (skip for locked entries, will generate stub)
            let md = if is_locked {
                String::new() // Placeholder, we'll use encrypted_bytes directly
            } else {
                fs::read_to_string(&file_path)?
            };

            let md = work_period::process(&md);

            // Determine if this is a shadow entry
            let is_shadow = filename.contains("entries/shadow/");

            // Extract entry number if this is an entry file
            let entry_num: Option<u32> =
                mdfile
                    .file_name()
                    .and_then(|f| f.to_str())
                    .and_then(|fname| {
                        // Remove .enc extension if present for parsing
                        let fname_clean = fname.strip_suffix(".enc").unwrap_or(fname);
                        fname_clean
                            .find('-')
                            .and_then(|dash_pos| fname_clean[..dash_pos].parse::<u32>().ok())
                    });

            // Generate HTML body
            let body = if is_locked {
                // For locked entries: generate stub with embedded encrypted bytes (no decryption needed!)
                let encrypted_b64 = crypto::to_base64(encrypted_bytes.as_ref().unwrap());
                generate_locked_stub_from_encrypted(&encrypted_b64)
            } else {
                // For regular entries: normal markdown to HTML
                let parser = MdParser::new_ext(&md, Options::all());
                let mut body = String::new();
                pulldown_cmark::html::push_html(&mut body, parser);

                // Add navigation for entry files
                if let Some(entry_num) = entry_num {
                    let navigation = generate_entry_navigation(entry_num, is_shadow);
                    body = navigation + &body;
                }

                body
            };

            // Wrap in layout
            let mut html = String::new();
            html.push_str(&Layout::header());
            html.push_str(Layout::body(&body).as_str());
            html.push_str(&Layout::footer());

            // Determine output file path
            let mut htmlfile = if let Some("index.md" | "cv.md") = mdfile.to_str() {
                PathBuf::from(mdfile)
            } else {
                let mdfile_str = mdfile.to_str().unwrap();
                // Remove .enc extension for path calculation
                let mdfile_clean = mdfile_str.strip_suffix(".enc").unwrap_or(mdfile_str);

                if mdfile_clean.contains("entries/shadow/") {
                    // Shadow entry: write to priv/entries/N.html
                    if let Some(entry_num) = entry_num {
                        PathBuf::from("priv/entries").join(entry_num.to_string())
                    } else {
                        PathBuf::from("priv").join(mdfile)
                    }
                } else if let Some(v) = mdfile_clean.split_once('-') {
                    // Regular numbered entry: write to pub/entries/N.html
                    PathBuf::from(PUBLIC_DIR).join(v.0)
                } else {
                    // Other files: write to pub/
                    PathBuf::from(PUBLIC_DIR).join(mdfile)
                }
            };

            htmlfile.set_extension("html");
            fs::write(&htmlfile, html)?;

            if is_locked {
                eprintln!("Generated locked HTML: {}", htmlfile.display());
            }
        }

        fs::create_dir_all(DOWNLOAD_DIR)?;

        Self::export("cv.md", "sbelokon")?;
        Self::export("index.md", "cover")?;

        // Generate 404 page and directory index stubs
        generate_error_pages()?;

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
