# Edit Command Implementation Plan

**Created:** 2025-10-13
**Status:** Ready for Implementation
**Complexity:** Medium (estimated 3-4 hours)

---

## 1. Overview

### Feature Description
Add `edit` subcommand to the CLI that allows users to edit markdown entries using their preferred text editor. The command supports:
- Entry specifier format: `5p` (public), `5s` (shadow), or plain `5` (defaults to public)
- Full file paths (relative or absolute)
- Automatic decryption/re-encryption of locked (`.enc`) entries
- Fallback to `vim` if `$EDITOR` not set

### User Stories

**As a user, I want to:**
- Edit entry #5 by typing `cargo run -- edit 5p` instead of manually navigating to the file
- Edit shadow entries with `cargo run -- edit 3s`
- Edit locked entries without manually decrypting/re-encrypting
- Use my preferred editor configured via `$EDITOR`

---

## 2. CLI Interface Design

### 2.1 Argument Structure

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Edit an existing blog entry
    Edit {
        /// Entry specifier: "5p" (public), "5s" (shadow), "5" (defaults to public),
        /// or full path to markdown/encrypted file
        target: String,
    },
}
```

### 2.2 Entry Specifier Format

**Valid formats:**
- `5p` → Public entry 5 (`in/entries/5-*.md` or `5-*.enc`)
- `5s` → Shadow entry 5 (`in/entries/shadow/5-*.md` or `5-*.enc`)
- `5` → Defaults to public entry 5 (same as `5p`)
- `in/entries/5-title.md` → Direct file path (relative)
- `/absolute/path/to/file.md` → Direct file path (absolute)

**Invalid formats (produce clear errors):**
- `5x` → "Invalid visibility modifier 'x'. Use 'p' (public) or 's' (shadow)"
- `p5` → "Invalid entry specifier 'p5'. Format: <number>[p|s]"
- `abc` → "Invalid entry specifier 'abc'"

### 2.3 Usage Examples

```bash
# Edit by entry number (public)
cargo run -- edit 5
cargo run -- edit 5p

# Edit shadow entry
cargo run -- edit 3s

# Edit by full path
cargo run -- edit in/entries/5-title.md
cargo run -- edit in/entries/shadow/3-private.enc
cargo run -- edit /Users/enkron/.../in/entries/7-post.md

# Edit locked entry (auto-decrypts)
cargo run -- edit 5p  # (if 5-title.enc exists, decrypts to temp file)
```

---

## 3. Entry Resolution Logic

### 3.1 Parsing Algorithm

**Priority order:**
1. Check if `target` exists as a file path (absolute or relative)
2. Otherwise, parse as entry specifier: `<number>[p|s]`

**Pseudocode:**
```rust
fn parse_target(target: &str) -> Result<TargetSpec, Error> {
    // 1. Direct path check
    if target.contains('/') || Path::new(target).exists() {
        return Ok(TargetSpec::Path(PathBuf::from(target)));
    }

    // 2. Parse entry specifier
    let (num_str, visibility) = if target.ends_with('p') {
        (&target[..target.len()-1], Visibility::Public)
    } else if target.ends_with('s') {
        (&target[..target.len()-1], Visibility::Shadow)
    } else {
        // Default to public if no suffix
        (target, Visibility::Public)
    };

    let entry_num = num_str.parse::<u32>()
        .map_err(|_| anyhow!("Invalid entry specifier '{}'", target))?;

    Ok(TargetSpec::Entry { num: entry_num, visibility })
}

enum TargetSpec {
    Path(PathBuf),
    Entry { num: u32, visibility: Visibility },
}

enum Visibility {
    Public,  // in/entries/
    Shadow,  // in/entries/shadow/
}
```

### 3.2 Public vs Shadow Directory Resolution

```rust
fn resolve_entry(num: u32, visibility: Visibility) -> Result<PathBuf, Error> {
    let entries_dir = match visibility {
        Visibility::Public => ENTRIES_DIR,       // "in/entries"
        Visibility::Shadow => SHADOW_ENTRIES_DIR, // "in/entries/shadow"
    };

    find_entry_file(entries_dir, num)
}
```

### 3.3 File Extension Priority (.md vs .enc)

**When resolving entry number, if multiple files match:**

```
in/entries/5-old-title.md
in/entries/5-old-title.enc
```

**Resolution logic:**
1. Search for `{num}-*.enc` first
2. If not found, search for `{num}-*.md`
3. If neither found, error: "Entry {num} not found in {directory}"

**Rationale:** Locked (`.enc`) files are the "source of truth" - if a file is encrypted, the plaintext (`.md`) should not exist simultaneously.

**Implementation:**
```rust
fn find_entry_file(dir: &str, num: u32) -> Result<PathBuf, Error> {
    let entries = fs::read_dir(dir)?;

    // Try .enc first
    for entry in entries.filter_map(Result::ok) {
        let filename = entry.file_name().to_string_lossy().to_string();
        if filename.starts_with(&format!("{}-", num)) && filename.ends_with(".enc") {
            return Ok(entry.path());
        }
    }

    // Fall back to .md
    let entries = fs::read_dir(dir)?;
    for entry in entries.filter_map(Result::ok) {
        let filename = entry.file_name().to_string_lossy().to_string();
        if filename.starts_with(&format!("{}-", num)) && filename.ends_with(".md") {
            return Ok(entry.path());
        }
    }

    Err(anyhow!("Entry {} not found in {}", num, dir))
}
```

### 3.4 Error Cases

| Scenario | Error Message | Example |
|----------|--------------|---------|
| Invalid specifier | "Invalid entry specifier 'abc'. Format: \<number\>[p\|s]" | `edit abc` |
| Wrong modifier | "Invalid visibility modifier 'x'. Use 'p' or 's'" | `edit 5x` |
| Entry not found | "Entry 5 not found in public entries" | `edit 5p` (doesn't exist) |
| Path doesn't exist | "File not found: in/entries/missing.md" | `edit in/entries/missing.md` |
| Ambiguous path | (Not applicable - we check file exists first) | N/A |

---

## 4. File Decryption Flow (for .enc files)

### 4.1 Passphrase Acquisition

**Priority:**
1. Environment variable `ENKRONIO_LOCK_KEY`
2. Interactive prompt (no echo)
3. **Fail fast** on incorrect passphrase (no retries)

**Code:**
```rust
fn get_passphrase_for_edit() -> Result<String, Error> {
    // Reuse existing get_passphrase() from lock command
    get_passphrase("Enter passphrase to decrypt for editing:")
}
```

### 4.2 Temporary File Creation

**Location:** System temp directory (`/tmp` on Unix, `%TEMP%` on Windows)

**Naming convention:**
```
/tmp/enkronio-edit-{entry_num}-{random_suffix}.md
```

**Example:**
```
/tmp/enkronio-edit-5-a3f9b2c1.md
```

**Implementation:**
```rust
use std::env;
use rand::Rng;

fn create_temp_file_for_edit(entry_num: u32, content: &str) -> Result<PathBuf, Error> {
    let random_suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join(format!("enkronio-edit-{}-{}.md", entry_num, random_suffix));

    fs::write(&temp_file, content)?;

    eprintln!("Decrypted to temporary file: {}", temp_file.display());

    Ok(temp_file)
}
```

### 4.3 Cleanup Strategy

**When to clean up:**
- After successful re-encryption
- On error (best effort - use `defer` pattern)

**Implementation:**
```rust
struct TempFileGuard(PathBuf);

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        eprintln!("Cleaned up temporary file: {}", self.0.display());
    }
}
```

### 4.4 Full Decryption Flow

```rust
fn handle_edit_locked_file(enc_path: &Path) -> Result<(), Error> {
    // 1. Get passphrase
    let passphrase = get_passphrase_for_edit()?;

    // 2. Decrypt
    let encrypted_bytes = fs::read(enc_path)?;
    let plaintext = crypto::decrypt(&encrypted_bytes, &passphrase)?;

    // 3. Extract entry number from filename
    let entry_num = extract_entry_number(enc_path)?;

    // 4. Create temp file
    let temp_file = create_temp_file_for_edit(entry_num, &plaintext)?;
    let _guard = TempFileGuard(temp_file.clone());

    // 5. Open in editor
    open_in_editor(&temp_file)?;

    // 6. Read edited content
    let edited_content = fs::read_to_string(&temp_file)?;

    // 7. Re-encrypt
    let encrypted_bytes = crypto::encrypt(&edited_content, &passphrase)?;
    fs::write(enc_path, encrypted_bytes)?;

    eprintln!("Re-encrypted: {}", enc_path.display());

    Ok(())
    // Temp file cleaned up by Drop guard
}
```

---

## 5. Editor Invocation

### 5.1 Environment Variable Detection

**Fallback chain:**
1. `$EDITOR` environment variable
2. Hardcoded fallback: `vim`
3. Error if vim not found in PATH

**Rationale:** Keep it simple - `$VISUAL` is less commonly used, and most developers have `$EDITOR` or vim installed.

**Implementation:**
```rust
fn get_editor() -> Result<String, Error> {
    if let Ok(editor) = env::var("EDITOR") {
        if !editor.is_empty() {
            return Ok(editor);
        }
    }

    // Fallback to vim
    Ok("vim".to_string())
}
```

### 5.2 Process Spawning

**Use `std::process::Command` with inherited stdio:**

```rust
use std::process::Command;

fn open_in_editor(file_path: &Path) -> Result<(), Error> {
    let editor = get_editor()?;

    eprintln!("Opening in {}: {}", editor, file_path.display());

    let status = Command::new(&editor)
        .arg(file_path)
        .status()
        .map_err(|e| anyhow!("Failed to launch editor '{}': {}", editor, e))?;

    if !status.success() {
        return Err(anyhow!("Editor exited with non-zero status: {:?}", status.code()));
    }

    Ok(())
}
```

### 5.3 Wait for Exit

**Blocking behavior:**
- The command **waits** for the editor to close
- User edits, saves, and closes editor
- Control returns to the program
- Post-processing (re-encryption) happens after editor closes

**No background watching** - simplicity over complexity.

---

## 6. Post-Edit Processing

### 6.1 Modification Detection (SKIP FOR SIMPLICITY)

**Decision:** Always re-encrypt, even if no changes made.

**Rationale:**
- Simpler implementation
- File hash checking adds complexity
- Re-encryption is fast (Argon2id takes ~1-2s, acceptable)
- Timestamp update is acceptable behavior

### 6.2 Re-encryption Logic

**If original file was `.enc`:**
1. Read edited content from temp file
2. Re-encrypt with same passphrase
3. Write back to original `.enc` path
4. Clean up temp file

**If original file was `.md`:**
1. Read edited content
2. Write back to original `.md` path (no encryption)

**Implementation:**
```rust
fn save_edited_content(original_path: &Path, edited_path: &Path, passphrase: Option<&str>) -> Result<(), Error> {
    let edited_content = fs::read_to_string(edited_path)?;

    if original_path.extension().and_then(|s| s.to_str()) == Some("enc") {
        // Re-encrypt
        let passphrase = passphrase.ok_or(anyhow!("Passphrase required for re-encryption"))?;
        let encrypted_bytes = crypto::encrypt(&edited_content, passphrase)?;
        fs::write(original_path, encrypted_bytes)?;
        eprintln!("Re-encrypted: {}", original_path.display());
    } else {
        // Plain markdown
        fs::write(original_path, edited_content)?;
        eprintln!("Saved: {}", original_path.display());
    }

    Ok(())
}
```

### 6.3 Temporary File Cleanup

**Cleanup handled by `TempFileGuard` Drop implementation (see 4.3)**

**Additional safety:**
- Even if process crashes, OS will eventually clean `/tmp`
- Random suffix prevents conflicts between concurrent edits

### 6.4 Error Recovery

**Scenarios:**

| Error | Handling | Example |
|-------|----------|---------|
| Wrong passphrase | Exit immediately, show error | "Decryption failed: incorrect passphrase" |
| Editor not found | Exit with helpful message | "Editor 'nano' not found. Set $EDITOR or install vim" |
| Editor crashes | Temp file preserved for manual recovery | "Editor exited abnormally. Temp file: /tmp/..." |
| Re-encryption fails | Original `.enc` preserved, temp file kept | "Re-encryption failed. Original file unchanged." |
| File locked by system | Exit with error | "Cannot access file: Permission denied" |

**Principle:** Fail-safe - never delete original file until successful re-encryption.

---

## 7. Implementation Details

### 7.1 Module Structure

**Option A: Inline in main.rs** (simpler for small feature)
```rust
// src/main.rs
fn handle_edit(target: &str) -> Result<(), Error> {
    // Implementation here
}
```

**Option B: Separate module** (cleaner, but adds file)
```rust
// src/edit.rs
pub fn handle_edit(target: &str) -> Result<(), Error> {
    // Implementation here
}

// src/main.rs
mod edit;
use edit::handle_edit;
```

**Recommendation:** Start with Option A, refactor to Option B if `main.rs` gets too large.

### 7.2 Function Signatures

```rust
// Main entry point
fn handle_edit(target: &str) -> Result<(), Error>

// Target parsing
fn parse_target(target: &str) -> Result<TargetSpec, Error>
enum TargetSpec {
    Path(PathBuf),
    Entry { num: u32, visibility: Visibility },
}

// Entry resolution
fn resolve_entry(num: u32, visibility: Visibility) -> Result<PathBuf, Error>
fn find_entry_file(dir: &str, num: u32) -> Result<PathBuf, Error>

// File handling
fn handle_edit_locked_file(enc_path: &Path) -> Result<(), Error>
fn handle_edit_plain_file(md_path: &Path) -> Result<(), Error>
fn create_temp_file_for_edit(entry_num: u32, content: &str) -> Result<PathBuf, Error>

// Editor
fn get_editor() -> Result<String, Error>
fn open_in_editor(file_path: &Path) -> Result<(), Error>

// Utility
fn extract_entry_number(path: &Path) -> Result<u32, Error>
struct TempFileGuard(PathBuf);
```

### 7.3 Error Types

**Reuse existing `anyhow::Error` for simplicity.**

**Error messages (user-facing):**
```rust
// Clear, actionable error messages
anyhow!("Invalid entry specifier '{}'. Format: <number>[p|s]. Examples: 5p, 12s", target)
anyhow!("Entry {} not found in {} directory", num, visibility_str)
anyhow!("Editor '{}' not found. Set $EDITOR environment variable or install vim", editor)
anyhow!("Decryption failed: {}", e)
anyhow!("Re-encryption failed: {}. Original file unchanged.", e)
```

---

## 8. Testing Strategy

### 8.1 Unit Tests (Entry Parsing)

**Test file:** `src/main.rs` (inline) or `src/edit.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_target_public_explicit() {
        let spec = parse_target("5p").unwrap();
        assert!(matches!(spec, TargetSpec::Entry { num: 5, visibility: Visibility::Public }));
    }

    #[test]
    fn test_parse_target_shadow() {
        let spec = parse_target("12s").unwrap();
        assert!(matches!(spec, TargetSpec::Entry { num: 12, visibility: Visibility::Shadow }));
    }

    #[test]
    fn test_parse_target_defaults_to_public() {
        let spec = parse_target("7").unwrap();
        assert!(matches!(spec, TargetSpec::Entry { num: 7, visibility: Visibility::Public }));
    }

    #[test]
    fn test_parse_target_invalid_modifier() {
        let err = parse_target("5x").unwrap_err();
        assert!(err.to_string().contains("Invalid"));
    }

    #[test]
    fn test_parse_target_file_path() {
        // Create temp file for testing
        let temp = tempfile::NamedTempFile::new().unwrap();
        let path = temp.path().to_str().unwrap();

        let spec = parse_target(path).unwrap();
        assert!(matches!(spec, TargetSpec::Path(_)));
    }

    #[test]
    fn test_parse_target_relative_path() {
        let spec = parse_target("in/entries/5-title.md").unwrap();
        assert!(matches!(spec, TargetSpec::Path(_)));
    }

    #[test]
    fn test_extract_entry_number() {
        let path = PathBuf::from("in/entries/5-title.md");
        assert_eq!(extract_entry_number(&path).unwrap(), 5);

        let path = PathBuf::from("in/entries/shadow/12-private.enc");
        assert_eq!(extract_entry_number(&path).unwrap(), 12);
    }
}
```

### 8.2 Integration Tests (Full Flow)

**Test file:** `tests/edit_integration.rs`

```rust
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_edit_public_entry() {
    // Setup: Create test entry
    let temp_dir = TempDir::new().unwrap();
    let entries_dir = temp_dir.path().join("in/entries");
    fs::create_dir_all(&entries_dir).unwrap();

    let entry_file = entries_dir.join("5-test.md");
    fs::write(&entry_file, "# Test Entry\n\nOriginal content").unwrap();

    // TODO: Mock editor (tricky - might need to skip this test)

    // Test: Run edit command
    // Verify: File opened in editor
}

#[test]
fn test_edit_locked_entry_with_env_key() {
    // Setup: Create encrypted entry
    // Set ENKRONIO_LOCK_KEY
    // Test: Run edit command
    // Verify: Decrypts, edits, re-encrypts
    // Verify: Temp file cleaned up
}

#[test]
fn test_edit_nonexistent_entry() {
    // Test: edit 999p
    // Verify: Error "Entry 999 not found"
}
```

**Note:** Full integration tests are challenging due to editor interaction. Consider:
- Mock editor with a script that modifies the file
- Manual testing for editor flow
- Focus unit tests on parsing/resolution logic

### 8.3 Edge Cases

| Test Case | Expected Behavior |
|-----------|-------------------|
| `edit 5` (only .enc exists) | Decrypt, edit, re-encrypt |
| `edit 5` (only .md exists) | Edit plaintext directly |
| `edit 5` (both .md and .enc exist) | Prefer .enc, warn about duplicate |
| `edit 999` (doesn't exist) | Error: "Entry 999 not found" |
| `edit 5x` (invalid modifier) | Error: "Invalid visibility modifier" |
| `edit /absolute/path.md` | Edit file at absolute path |
| Wrong passphrase | Error: "Decryption failed", exit immediately |
| $EDITOR not set | Fall back to vim |
| vim not installed | Error: "Editor not found" |
| Editor crashes mid-edit | Temp file preserved for recovery |

---

## 9. Implementation Checklist

### Phase 1: CLI Setup
- [ ] Add `Edit` variant to `Commands` enum
- [ ] Add target parsing to `match cli.command`
- [ ] Add `handle_edit()` function skeleton

### Phase 2: Entry Resolution
- [ ] Implement `parse_target()` function
  - [ ] Check if path exists
  - [ ] Parse entry specifier (number + p/s modifier)
  - [ ] Default to public if no modifier
- [ ] Implement `resolve_entry()` function
  - [ ] Map visibility to directory
  - [ ] Call `find_entry_file()`
- [ ] Implement `find_entry_file()` function
  - [ ] Search for .enc files first
  - [ ] Fall back to .md files
  - [ ] Error if not found
- [ ] Add unit tests for parsing logic

### Phase 3: Plain File Editing
- [ ] Implement `handle_edit_plain_file()`
  - [ ] Get editor from environment
  - [ ] Open file in editor
  - [ ] Wait for exit
- [ ] Implement `get_editor()` function
  - [ ] Check $EDITOR
  - [ ] Fall back to vim
- [ ] Implement `open_in_editor()` function
  - [ ] Spawn editor process
  - [ ] Inherit stdio
  - [ ] Check exit status

### Phase 4: Locked File Editing
- [ ] Implement `handle_edit_locked_file()`
  - [ ] Get passphrase (reuse `get_passphrase()`)
  - [ ] Decrypt file
  - [ ] Create temp file
  - [ ] Open in editor
  - [ ] Re-encrypt after edit
  - [ ] Clean up temp file
- [ ] Implement `create_temp_file_for_edit()`
  - [ ] Generate random suffix
  - [ ] Write to system temp dir
  - [ ] Return path
- [ ] Implement `TempFileGuard`
  - [ ] Drop trait to clean up file
- [ ] Add error handling for decryption failure

### Phase 5: Error Handling
- [ ] Add clear error messages for all failure modes
- [ ] Test wrong passphrase handling
- [ ] Test missing entry handling
- [ ] Test invalid specifier handling

### Phase 6: Testing
- [ ] Write unit tests for `parse_target()`
- [ ] Write unit tests for `extract_entry_number()`
- [ ] Manual testing with real entries
- [ ] Test with $EDITOR set to different editors
- [ ] Test with locked entries

### Phase 7: Documentation
- [ ] Update CLAUDE.md with edit command examples
- [ ] Add CLI help text
- [ ] Document entry specifier format

---

## 10. Future Enhancements

**Not in scope for initial implementation, but consider for future:**

### 10.1 Auto-save Watching
Watch temp file for changes, re-encrypt on each save (not just on editor exit).

**Use case:** User saves multiple times while editing.

**Complexity:** Medium (requires file watching library like `notify`)

### 10.2 Edit History Tracking
Track edit timestamps in lockfile or separate metadata file.

**Use case:** "When was this entry last edited?"

**Complexity:** Low

### 10.3 Multiple Entry Editing
Support editing multiple entries at once: `cargo run -- edit 5p 7s 12p`

**Use case:** Batch editing.

**Complexity:** Medium (need to handle multiple temp files)

### 10.4 Preview Mode (Read-Only)
`cargo run -- preview 5p` - decrypt and open in read-only mode.

**Use case:** View locked entry without risk of accidental edits.

**Complexity:** Low (pass `--readonly` flag to vim)

### 10.5 Editor Configuration File
Allow users to specify editor in `.enkronio-config`:
```toml
[editor]
command = "code"
args = ["--wait"]
```

**Use case:** Users who prefer VS Code or other GUI editors.

**Complexity:** Medium (need config file parsing)

---

## 11. Implementation Notes

### Dependencies
**No new dependencies required!** All functionality uses existing:
- `std::process::Command` (editor invocation)
- `std::env::temp_dir()` (temp file creation)
- `rand` (already in `Cargo.toml` for crypto) - random suffix
- Existing `crypto` module (decrypt/encrypt)
- Existing `get_passphrase()` function

### Estimated Time
- **Phase 1-2 (CLI + Resolution):** 1 hour
- **Phase 3 (Plain editing):** 30 minutes
- **Phase 4 (Locked editing):** 1.5 hours
- **Phase 5 (Error handling):** 30 minutes
- **Phase 6 (Testing):** 1 hour
- **Total:** ~4.5 hours

### Key Design Decisions Summary
1. **Entry specifier:** Number defaults to public (`5` = `5p`)
2. **File priority:** Prefer `.enc` over `.md`
3. **Temp location:** System temp dir with random suffix
4. **Re-encryption:** Automatic after edit (secure by default)
5. **Passphrase:** Single attempt, fail-fast
6. **Editor fallback:** `$EDITOR` → `vim` → error

---

## 12. Example Usage Scenarios

### Scenario 1: Edit Public Entry (Plaintext)
```bash
$ cargo run -- edit 5
Opening in vim: in/entries/5-title.md
# User edits, saves, closes vim
Saved: in/entries/5-title.md
```

### Scenario 2: Edit Locked Public Entry
```bash
$ cargo run -- edit 5p
Entry 5 is encrypted. Decrypting for editing...
Enter passphrase to decrypt for editing: ****
Decrypted to temporary file: /tmp/enkronio-edit-5-a3f9b2c1.md
Opening in vim: /tmp/enkronio-edit-5-a3f9b2c1.md
# User edits, saves, closes vim
Re-encrypted: in/entries/5-title.enc
Cleaned up temporary file: /tmp/enkronio-edit-5-a3f9b2c1.md
```

### Scenario 3: Edit Shadow Entry
```bash
$ cargo run -- edit 3s
Opening in vim: in/entries/shadow/3-private.md
Saved: in/entries/shadow/3-private.md
```

### Scenario 4: Edit by Full Path
```bash
$ cargo run -- edit in/entries/7-post.md
Opening in vim: in/entries/7-post.md
Saved: in/entries/7-post.md
```

### Scenario 5: Wrong Passphrase
```bash
$ cargo run -- edit 5p
Entry 5 is encrypted. Decrypting for editing...
Enter passphrase to decrypt for editing: ****
Error: Decryption failed: incorrect passphrase or corrupted data
```

### Scenario 6: Entry Not Found
```bash
$ cargo run -- edit 999p
Error: Entry 999 not found in public entries
```

---

## 13. Security Considerations

### 13.1 Passphrase Handling
- Never log or print passphrase
- Use `rpassword` for no-echo input
- Clear passphrase from memory after use (Rust does this automatically on drop)

### 13.2 Temporary File Security
- Use system temp dir with restrictive permissions (OS default)
- Random suffix prevents conflicts and guessing
- Clean up immediately after use
- Even if process crashes, temp file is in `/tmp` (eventually cleaned by OS)

### 13.3 File Integrity
- Never delete original `.enc` file until successful re-encryption
- If re-encryption fails, preserve temp file for manual recovery
- Atomic write operations where possible

### 13.4 Race Conditions
- Random suffix prevents concurrent edit conflicts
- No complex locking needed (single-user tool)

---

**End of Implementation Plan**

---

**Status:** ✅ Ready for implementation
**Next Steps:** Begin with Phase 1 (CLI Setup) when ready to implement
