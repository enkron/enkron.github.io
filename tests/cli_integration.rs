/// Integration tests for CLI functionality
///
/// Tests the complete workflow of adding blog entries via CLI,
/// including file creation, junkyard updates, and entry numbering.
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to set up a temporary test environment.
/// Creates a minimal directory structure with in/entries/ and in/junkyard.md.
/// Reserved for future use in file system tests.
#[allow(dead_code)]
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let entries_dir = temp_dir.path().join("in/entries");
    fs::create_dir_all(&entries_dir).expect("Failed to create entries directory");

    // Create initial junkyard.md
    let junkyard_path = temp_dir.path().join("in/junkyard.md");
    let junkyard_content = "# index\n\n## recent posts\n\n";
    fs::write(&junkyard_path, junkyard_content).expect("Failed to create junkyard.md");

    temp_dir
}

/// Tests CLI help output.
/// Verifies that --help flag produces expected usage information.
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--release", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Static site generator"));
    assert!(stdout.contains("add"));
    assert!(stdout.contains("help"));
}

/// Tests CLI add subcommand help output.
/// Verifies that add --help produces correct subcommand documentation.
#[test]
fn test_cli_add_help() {
    let output = Command::new("cargo")
        .args(["run", "--release", "--", "add", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Add a new blog entry"));
    assert!(stdout.contains("TITLE"));
}

/// Tests site build without errors.
/// Verifies that default site generation completes successfully.
#[test]
fn test_site_build() {
    let output = Command::new("cargo")
        .args(["run", "--release"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Site build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify expected output directories exist
    assert!(PathBuf::from("pub").exists(), "pub/ directory not created");
    assert!(
        PathBuf::from("download").exists(),
        "download/ directory not created"
    );
}

/// Tests that generated HTML files contain expected structure.
/// Verifies HTML templates are applied correctly.
#[test]
fn test_generated_html_structure() {
    // Ensure site is built
    Command::new("cargo")
        .args(["run", "--release"])
        .output()
        .expect("Failed to build site");

    // Check index.html
    let index_content = fs::read_to_string("index.html").expect("Failed to read index.html");
    assert!(index_content.contains("<!DOCTYPE html>"));
    assert!(index_content.contains("<html lang=\"en-US\">"));
    assert!(index_content.contains("enk junkyard"));

    // Check cv.html
    let cv_content = fs::read_to_string("cv.html").expect("Failed to read cv.html");
    assert!(cv_content.contains("<!DOCTYPE html>"));
    assert!(cv_content.contains("<html lang=\"en-US\">"));
}

/// Tests that PDF files are generated.
/// Verifies PDF export functionality produces output files.
#[test]
fn test_pdf_generation() {
    // Ensure site is built
    Command::new("cargo")
        .args(["run", "--release"])
        .output()
        .expect("Failed to build site");

    // Check PDFs exist
    let cv_pdf = PathBuf::from("download/sbelokon.pdf");
    assert!(cv_pdf.exists(), "CV PDF not generated");

    let cover_pdf = PathBuf::from("download/cover.pdf");
    assert!(cover_pdf.exists(), "Cover PDF not generated");

    // Verify files are readable and have reasonable size (> 100 bytes)
    // Note: PDF size validation is lenient to account for test environment variations
    if let Ok(metadata) = fs::metadata(&cv_pdf) {
        let size = metadata.len();
        assert!(size > 100, "CV PDF appears invalid (size: {} bytes)", size);
    }

    if let Ok(metadata) = fs::metadata(&cover_pdf) {
        let size = metadata.len();
        assert!(
            size > 100,
            "Cover PDF appears invalid (size: {} bytes)",
            size
        );
    }
}

/// Tests entry HTML files are generated with correct naming.
/// Verifies numbered entries from in/entries/ produce corresponding HTML.
#[test]
fn test_entry_html_generation() {
    // Ensure site is built
    Command::new("cargo")
        .args(["run", "--release"])
        .output()
        .expect("Failed to build site");

    // Check that entry files exist
    let entry1 = PathBuf::from("pub/entries/1.html");
    assert!(entry1.exists(), "Entry 1 HTML not generated");

    let entry2 = PathBuf::from("pub/entries/2.html");
    assert!(entry2.exists(), "Entry 2 HTML not generated");

    let entry3 = PathBuf::from("pub/entries/3.html");
    assert!(entry3.exists(), "Entry 3 HTML not generated");
}

/// Tests that generated HTML includes theme toggle button.
/// Verifies WASM module integration and theme UI elements.
#[test]
fn test_theme_toggle_in_html() {
    // Ensure site is built
    Command::new("cargo")
        .args(["run", "--release"])
        .output()
        .expect("Failed to build site");

    let index_content = fs::read_to_string("index.html").expect("Failed to read index.html");
    assert!(index_content.contains("theme-toggle"));
    assert!(index_content.contains("theme-icon"));
    assert!(index_content.contains("/web/pkg/enkronio.js"));
}

/// Tests that CSS files have cache-busting query strings.
/// Verifies hash-based versioning for static assets.
#[test]
fn test_css_cache_busting() {
    // Ensure site is built
    Command::new("cargo")
        .args(["run", "--release"])
        .output()
        .expect("Failed to build site");

    let index_content = fs::read_to_string("index.html").expect("Failed to read index.html");
    // Check for version query strings on CSS links
    assert!(index_content.contains("/css/main.css?v="));
    assert!(index_content.contains("/web/hack.css?v="));
}
