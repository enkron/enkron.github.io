# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A static site generator written in Rust that converts Markdown content to HTML pages and PDF files. The generator reads Markdown from `in/`, produces HTML in `pub/` and root directory, and generates PDFs in `download/`. Deployed to GitHub Pages via GitHub Actions.

## Development Commands

### Build and run the generator
```bash
cargo run --release
```

### Build and serve locally
```bash
make site
```
Builds the site and starts a local HTTP server on port 8080.

### Clean generated files
```bash
make clean
```
Removes `pub/`, `download/`, `index.html`, and `cv.html`.

### Check code
```bash
cargo check
cargo clippy
```

## Architecture

### Data Flow
1. **Input**: Markdown files in `in/` directory
   - Special files: `cv.md`, `index.md` (go to root as HTML + PDF)
   - Other files: processed into `pub/` directory
   - Date-prefixed files (e.g., `YYYY-MM-DD-title.md`) → `pub/YYYY-MM-DD.html`

2. **Processing Pipeline** (`src/main.rs`):
   - `Site::build()` walks `in/` directory using `walkdir`
   - Converts Markdown to HTML via `pulldown-cmark`
   - Wraps HTML with layout templates from `rend.rs`
   - Calls `Site::export()` to generate PDFs for `cv.md` and `index.md`

3. **Output**:
   - HTML files in `pub/` (for blog entries) and root (for index/cv)
   - PDF files in `download/` (cv and cover pages)

### Module Structure

**`src/main.rs`**: Entry point and HTML generation
- `Site::build()`: Main build pipeline that processes all Markdown files
- `Site::export()`: PDF generation for specific files
- Uses constants: `CONTENT_DIR = "in"`, `PUBLIC_DIR = "pub"`, `DOWNLOAD_DIR = "download"`

**`src/rend.rs`**: HTML layout templates
- `Layout::header()`: Navigation, meta tags, CSS links with cache-busting hashes
- `Layout::body()`: Content wrapper
- `Layout::footer()`: Build metadata (GitHub Actions env vars: `GITHUB_RUN_NUMBER`, `GITHUB_SHA`) and timestamp
- CSS cache-busting: Computes SHA256 hashes of `css/main.css` and `web/hack.css` at compile time using `once_cell::Lazy` and embeds them as query strings

**`src/pdf.rs`**: Custom PDF renderer (replaces `wkhtmltopdf`)
- `render()`: Main entry point that converts Markdown to PDF bytes
- `parse_markdown()`: Parses Markdown into structured blocks (headings, paragraphs, lists, tables)
- `PdfComposer`: Handles layout, pagination, and text positioning
- Writes raw PDF 1.4 format with Courier and Courier-Bold fonts
- Page dimensions: A4-like (595 x 842 points) with configurable margins

### Key Dependencies
- `pulldown-cmark`: Markdown parsing (both HTML and PDF generation)
- `walkdir`: Recursive directory traversal
- `chrono`: Timestamp generation for footer
- `once_cell`: Lazy static initialization for CSS hashes
- `sha2`: SHA256 hashing for cache-busting
- `anyhow`: Error handling

### File Naming Logic
- `index.md` and `cv.md` → root directory as `index.html` and `cv.html`
- Date-prefixed files: Split on first `-`, use date portion only (e.g., `2024-01-15-title.md` → `pub/2024-01-15.html`)
- Other files → `pub/filename.html`

## CI/CD

GitHub Actions workflow (`.github/workflows/build_site.yml`):
1. Installs Rust toolchain
2. Installs `wkhtmltopdf` (Ubuntu Jammy package) - note: this is no longer used in the code but still referenced in CI
3. Builds and runs: `cargo run --release`
4. Archives and deploys to GitHub Pages

The workflow runs on pushes to `main` and includes three jobs: `build`, `test` (stub), and `deploy`.
