# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A static site generator written in Rust that converts Markdown content to HTML pages and PDF files. The generator reads Markdown from `in/`, produces HTML in `pub/` and root directory, and generates PDFs in `download/`. Deployed to GitHub Pages via GitHub Actions.

## Development Commands

### Build and run the generator
```bash
cargo run --release
```
Generates HTML and PDF files from all Markdown sources in `in/`.

### Add a new blog entry
```bash
cargo run --release -- add "Entry Title"
```
Creates `in/entries/N-entry-title.md` (where N is auto-incremented) and updates `in/junkyard.md` with a new link entry formatted as `DD.MM.YYYY: [Entry Title](/pub/entries/N.html)`. Date uses Roman numerals for month (e.g., `11.X.2025`).

**CLI behavior:**
- Scans `in/entries/` to find highest existing number
- Generates filename slug: lowercase, alphanumeric + dashes, no consecutive dashes
- Creates file with template: `# Entry Title\n\n`
- Inserts link at top of "## recent posts" section in `junkyard.md`

**Example:**
```bash
cargo run --release -- add "Setting up Kubernetes"
# Creates: in/entries/4-setting-up-kubernetes.md
# Updates: in/junkyard.md with "11.X.2025: [Setting up Kubernetes](/pub/entries/4.html)"
```

### Build and serve locally
```bash
make site
```
Builds the WASM module, generates the site, and starts a local HTTP server on port 8080.

### Build WASM module only
```bash
wasm-pack build --target web --out-dir web/pkg
```
Builds the dark mode WASM module to `web/pkg/`.

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

### Git hooks
Install pre-push hook for automated code quality checks:
```bash
./hooks/install-hooks.sh
```

**Hook architecture:**
- Location: `hooks/pre-push` (version-controlled)
- Installation: Symlinked to `.git/hooks/pre-push` via install script
- Trigger: Runs automatically before `git push`
- Validation: Formatting (rustfmt), linting (clippy), tests
- Bypass: `git push --no-verify` (discouraged)

**Design rationale:**
- Version-controlled hooks enable team consistency
- Symlink approach: hooks update automatically with `git pull`
- Pre-push (not pre-commit): Allows local WIP, validates before sharing
- Fast feedback: Catches issues before CI pipeline

**Hook structure:**
```
hooks/
├── pre-push           # Hook script (executable)
└── install-hooks.sh   # Installation script
.git/hooks/
└── pre-push -> ../../hooks/pre-push  # Symlink (created by installer)
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

**`src/main.rs`**: Entry point, CLI, and HTML generation
- **CLI structure**: Uses `clap` derive API with optional subcommand
  - No args: runs `Site::build()` (default behavior)
  - `add <TITLE>`: runs `add_entry()` to create new blog entry
- `Site::build()`: Main build pipeline that processes all Markdown files
- `Site::export()`: PDF generation for specific files
- `add_entry()`: Creates new entry file and updates junkyard index
- `find_next_entry_number()`: Scans `in/entries/` for highest N in `N-*.md` pattern
- `generate_entry_filename()`: Converts title to slug (lowercase, dashes for spaces/special chars)
- `create_entry_file()`: Writes `# {title}\n\n` template to new file
- `update_junkyard()`: Inserts new entry link after "## recent posts" header in `junkyard.md`
- `month_to_roman()`: Converts 1-12 to Roman numerals (I-XII) for date formatting
- Uses constants: `CONTENT_DIR = "in"`, `PUBLIC_DIR = "pub"`, `DOWNLOAD_DIR = "download"`, `ENTRIES_DIR = "in/entries"`, `JUNKYARD_FILE = "in/junkyard.md"`

**`src/rend.rs`**: HTML layout templates
- `Layout::header()`: Navigation, meta tags, CSS links with cache-busting hashes, dark mode toggle button
- `Layout::body()`: Content wrapper
- `Layout::footer()`: Build metadata (GitHub Actions env vars: `GITHUB_RUN_NUMBER`, `GITHUB_SHA`), timestamp, and WASM module loader
- CSS cache-busting: Computes SHA256 hashes of `css/main.css` and `web/hack.css` at compile time using `once_cell::Lazy` and embeds them as query strings

**`src/lib.rs`**: WASM module for dark mode functionality
- `main()`: Initializes theme from localStorage on page load
- `toggle_theme()`: Switches between light and dark themes
- `init_theme()`: Sets up theme state and event listeners for toggle button
- Compiled to WebAssembly and loaded as ES6 module in footer

**`src/pdf.rs`**: Custom PDF renderer (replaces `wkhtmltopdf`)
- `render()`: Main entry point that converts Markdown to PDF bytes
- `parse_markdown()`: Parses Markdown into structured blocks (headings, paragraphs, lists, tables)
- `PdfComposer`: Handles layout, pagination, and text positioning
- Writes raw PDF 1.4 format with Courier and Courier-Bold fonts
- Page dimensions: A4-like (595 x 842 points) with configurable margins

### Key Dependencies
- `pulldown-cmark`: Markdown parsing (both HTML and PDF generation)
- `walkdir`: Recursive directory traversal
- `chrono`: Timestamp generation (footer, blog entry dates) and `Datelike` trait for date components
- `clap`: CLI argument parsing with derive API for subcommands
- `once_cell`: Lazy static initialization for CSS hashes
- `sha2`: SHA256 hashing for cache-busting
- `anyhow`: Error handling
- `wasm-bindgen`: Rust-WASM interop for dark mode toggle
- `web-sys`: Web API bindings for DOM manipulation, localStorage, and media queries

### File Naming Logic
- `index.md` and `cv.md` → root directory as `index.html` and `cv.html`
- Entry files in `in/entries/`: Numbered format `N-slug.md` → `pub/entries/N.html`
  - Number portion extracted by splitting on first `-`
  - `add` command auto-generates: scans entries for max N, creates `(N+1)-title-slug.md`
  - Slug generation: lowercase, spaces→dashes, alphanumeric+dashes only, no consecutive dashes
- Other files → `pub/filename.html`

**Entry numbering examples:**
- `in/entries/1-initial.md` → `pub/entries/1.html`
- `in/entries/4-setting-up-kubernetes.md` → `pub/entries/4.html`
- Manual renumbering: possible but requires updating `junkyard.md` links manually

## Dark Mode Feature

The site includes a three-state theme system implemented in WebAssembly:

**Theme States**:
- **Light** (✸): Force light theme
- **Dark** (☽): Force dark theme
- **Auto** (◐): Follow system preference (default)

**CSS Variables** (`css/main.css`):
- `:root` defines light theme colors
- `[data-theme="dark"]` defines dark theme colors
- All color values use CSS variables for seamless theme switching

**WASM Module** (`src/lib.rs`):
- `ThemePreference` enum: Light | Dark | Auto
- Loads preference from `localStorage` key `"theme-preference"` (defaults to Auto)
- Auto mode: queries `prefers-color-scheme` media feature via `window.matchMedia()`
- Real-time system theme tracking: event listener on media query updates theme when OS theme changes
- Toggle button cycles: Light → Dark → Auto → Light
- Icon updates to reflect current preference (not actual theme)
- Applies theme on page load (before first paint to prevent flash)
- Preference persists across sessions

**System Theme Detection**:
- Uses `MediaQueryList` from `web-sys` to detect OS theme
- `is_system_dark_mode()`: checks `(prefers-color-scheme: dark)` match status
- `setup_system_theme_listener()`: registers callback for OS theme changes
- Only applies system theme when preference is Auto

**Benefits of WASM approach**:
- Type-safe Rust code compiled to efficient WASM binary (~20KB)
- No runtime JavaScript parsing overhead
- Near-native performance
- Shared codebase with main site generator
- OS-level theme integration with opt-out capability

## CI/CD

GitHub Actions workflow (`.github/workflows/build_site.yml`):
1. Installs Rust toolchain and wasm-pack
2. Builds WASM module: `wasm-pack build --target web --out-dir web/pkg`
3. Builds and runs site generator: `cargo run --release`
4. Archives and deploys to GitHub Pages (includes `web/pkg/` directory)

The workflow runs on pushes to `main` and includes three jobs: `build`, `test` (stub), and `deploy`.

**Note**: The CI workflow must install `wasm-pack` and build the WASM module before running the site generator.
- to memorize