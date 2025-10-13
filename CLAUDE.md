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

### Add a shadow entry (private, unlisted)
```bash
cargo run --release -- add --shadow "Private Entry"
```
Creates a private blog entry in `in/entries/shadow/N-private-entry.md` that is NOT added to `junkyard.md`. Shadow entries follow the same numbering and naming rules as regular entries but are kept separate.

**Shadow entry behavior:**
- Stored in `in/entries/shadow/` directory (independent numbering from regular entries)
- Output to `priv/entries/N.html` during build (separate from `pub/`)
- Accessible via `/priv/entries/N.html` URLs
- Navigation links (Previous/Next) only connect to other shadow entries
- NOT listed in `junkyard.md` (private by default)
- Same timestamp and slug generation as regular entries

**Use cases:**
- Draft posts before publishing
- Private notes or documentation
- Content accessible only via direct URL

**Example:**
```bash
cargo run --release -- add --shadow "Private Notes"
# Creates: in/entries/shadow/1-private-notes.md
# Skips: junkyard.md update
# Accessible: https://enkron.org/priv/entries/1.html
```

### Encrypt a blog entry (password-protected)
```bash
# Step 1: Create entry normally
cargo run --release -- add "Encrypted Entry"

# Step 2: Encrypt the entry file
cargo run --release -- lock in/entries/N-encrypted-entry.md
```
Encrypts an existing markdown file using AES-256-GCM and stores it as `.enc` file in the repository. The original `.md` file is removed after encryption.

**Encryption behavior:**
- Uses AES-256-GCM authenticated encryption (NIST-approved, tamper-proof)
- Key derivation with Argon2id (memory-hard, GPU/ASIC resistant)
- Random salt and nonce per encryption (semantic security)
- Source file transformed: `.md` → `.enc` (encrypted on disk)
- Generates locked HTML stub with embedded encrypted content during build
- Decryption happens client-side in browser (no server needed)

**Security properties:**
- Argon2id parameters: 64MB memory, 3 iterations, 4 threads (OWASP 2024)
- Authentication tag prevents tampering with encrypted content
- Passphrase never stored in browser localStorage (re-enter per session)
- NOT compatible with `gpg` command-line tool (uses RustCrypto instead)

**Passphrase management:**
- Environment variable: `export ENKRONIO_LOCK_KEY="your-passphrase"`
- Interactive prompt: Will prompt securely if env var not set
- CLI flag NOT supported (insecure, visible in process list)
- Same passphrase used for all locked entries (global passphrase model)

**Example:**
```bash
# Set passphrase via environment variable
export ENKRONIO_LOCK_KEY="my-secure-passphrase-16-chars"

# Create and encrypt a regular entry
cargo run --release -- add "Private Research"
cargo run --release -- lock in/entries/7-private-research.md
# Creates: in/entries/7-private-research.enc (encrypted source)
# Accessible: https://enkron.org/pub/entries/7.html (shows lock UI)
# Listed in junkyard.md (title visible, content encrypted)

# Create and encrypt a shadow entry (double privacy: unlisted + encrypted)
cargo run --release -- add --shadow "Secret Notes"
cargo run --release -- lock in/entries/shadow/1-secret-notes.md
# Creates: in/entries/shadow/1-secret-notes.enc (encrypted source)
# Accessible: https://enkron.org/priv/entries/1.html (shows lock UI)
# NOT listed in junkyard.md
```

**Building site with locked entries:**
```bash
# No passphrase needed! Build works directly with .enc files
cargo run --release
# .enc files stay encrypted, embedded as-is in HTML stubs
# Browser WASM module handles decryption when user enters passphrase
```

**Unlocking (decrypting) entries locally:**
```bash
# Decrypt entry (.enc -> .md)
cargo run --release -- lock --unlock in/entries/5-title.enc
# Creates: in/entries/5-title.md (removes .enc file)

# Decrypt shadow entry
cargo run --release -- lock --unlock in/entries/shadow/2-title.enc
# Creates: in/entries/shadow/2-title.md (removes .enc file)
```

**Lockfile tracking:**
- File: `.enkronio-locks` (JSON format, gitignored)
- Tracks which entries are encrypted (number, shadow flag, timestamp)
- Used by build system to identify locked entries
- DO NOT commit to repository (contains metadata about encrypted entries)

**Important notes:**
- Encrypted source files (`.enc`) ARE committed to repository
- Use strong passphrases (16+ characters recommended)
- **No passphrase needed to build site** - encrypted files embedded as-is
- Browser-side decryption: users enter passphrase in web UI to view content
- Lost passphrase = permanently encrypted content (no recovery)

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
  - `add --shadow <TITLE>`: creates private entry in shadow directory
  - `lock <PATH>`: encrypts existing markdown file (.md → .enc)
  - `lock --unlock <PATH>`: decrypts encrypted file (.enc → .md)
- `Site::build()`: Main build pipeline that processes all Markdown files (.md and .enc only), handles shadow entry routing, embeds encrypted entries as-is (no decryption needed)
- `Site::export()`: PDF generation for specific files
- `add_entry(title, shadow)`: Creates new entry file with template, conditionally updates junkyard based on shadow flag
- `lock_file(path, unlock)`: Encrypts (.md → .enc) or decrypts (.enc → .md) existing file, removes original
- `get_passphrase(prompt_message)`: Gets passphrase from ENKRONIO_LOCK_KEY env var or secure interactive prompt
- `generate_locked_stub_from_encrypted(encrypted_b64)`: Creates locked HTML interface with embedded encrypted content for browser decryption (no passphrase needed at build time)
- `find_next_entry_number(dir)`: Scans specified directory for highest N in `N-*.md` or `N-*.enc` pattern
- `generate_entry_filename()`: Converts title to slug (lowercase, dashes for spaces/special chars)
- `create_entry_file()`: Writes entry template with title and timestamp
- `update_junkyard()`: Inserts new entry link after "## recent posts" header in `junkyard.md`
- `generate_entry_navigation(entry_num, is_shadow)`: Generates prev/next navigation with appropriate URL prefix
- `month_to_roman()`: Converts 1-12 to Roman numerals (I-XII) for date formatting
- `track_locked_entry()`, `read_lockfile()`, `write_lockfile()`, `is_entry_locked()`: Lockfile management for tracking encrypted entries
- Uses constants: `CONTENT_DIR = "in"`, `PUBLIC_DIR = "pub"`, `DOWNLOAD_DIR = "download"`, `ENTRIES_DIR = "in/entries"`, `SHADOW_ENTRIES_DIR = "in/entries/shadow"`, `JUNKYARD_FILE = "in/junkyard.md"`, `LOCK_KEY_ENV = "ENKRONIO_LOCK_KEY"`, `LOCKFILE_PATH = ".enkronio-locks"`

**`src/rend.rs`**: HTML layout templates
- `Layout::header()`: Navigation, meta tags, CSS links with cache-busting hashes, dark mode toggle button
- `Layout::body()`: Content wrapper
- `Layout::footer()`: Build metadata (GitHub Actions env vars: `GITHUB_RUN_NUMBER`, `GITHUB_SHA`), timestamp, and WASM module loader
- CSS cache-busting: Computes SHA256 hashes of `css/main.css` and `web/hack.css` at compile time using `once_cell::Lazy` and embeds them as query strings

**`src/lib.rs`**: WASM module for dark mode and browser-side decryption
- `main()`: Initializes theme and locked entry UI on page load
- `toggle_theme()`: Switches between light and dark themes
- `init_theme()`: Sets up theme state and event listeners for toggle button
- `init_locked_entry()`: Sets up decryption UI if page has locked entry
- `handle_decrypt()`: Handles decrypt button click, validates passphrase, decrypts content
- `decrypt_content()`: AES-256-GCM + Argon2id decryption (matches src/crypto.rs)
- `markdown_to_html()`: Simple markdown parser for decrypted content
- `process_inline_html()`: Allows safe HTML tags (span, strong, em, code) while escaping others
- Compiled to WebAssembly and loaded as ES6 module in footer

**`src/crypto.rs`**: Encryption/decryption module for locked entries
- `encrypt(plaintext, passphrase)`: Encrypts content with AES-256-GCM + Argon2id
- `decrypt(ciphertext, passphrase)`: Decrypts and verifies authenticated encryption
- `to_base64(bytes)`: Encodes encrypted data for HTML embedding
- `from_base64(encoded)`: Decodes base64 data for decryption
- Uses RustCrypto ecosystem (NOT GPG-compatible)
- Format: `salt|nonce|ciphertext+auth_tag` (delimited for parsing)
- Security: Argon2id (64MB, 3 iter, 4 threads), AES-256-GCM with random nonce

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
- Shadow entry files in `in/entries/shadow/`: Numbered format `N-slug.md` → `priv/entries/N.html`
  - Independent numbering sequence from regular entries
  - Same slug generation rules
  - Output to `priv/` directory (separate from `pub/`)
  - Navigation links only connect to other shadow entries
- Other files → `pub/filename.html`

**Entry numbering examples:**
- `in/entries/1-initial.md` → `pub/entries/1.html` (accessible at `/pub/entries/1.html`)
- `in/entries/4-setting-up-kubernetes.md` → `pub/entries/4.html` (accessible at `/pub/entries/4.html`)
- `in/entries/shadow/1-private-notes.md` → `priv/entries/1.html` (accessible at `/priv/entries/1.html`)
- Manual renumbering: possible but requires updating `junkyard.md` links manually for regular entries

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