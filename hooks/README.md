# Git Hooks

This directory contains version-controlled git hooks shared across the development team.

## Installation

Run from repository root:

```bash
./hooks/install-hooks.sh
```

This creates symlinks from `.git/hooks/` to version-controlled hooks in this directory.

## Available Hooks

### pre-push

**Purpose:** Validates code quality before pushing to remote repository.

**Checks performed:**
1. Code formatting (`cargo fmt --check`)
2. Clippy lints with pedantic mode (`cargo clippy -- -D warnings`)
3. Test suite execution (`cargo test --all-features`)

**When it runs:** Automatically before `git push` executes.

**Exit behavior:**
- Success (exit 0): All checks pass, push proceeds
- Failure (exit 1): At least one check fails, push blocked

**Bypass:**
```bash
git push --no-verify
```
Not recommended - use only when necessary (e.g., emergency hotfix).

## Architecture

### Version-Controlled Hooks

**Problem:** Git hooks in `.git/hooks/` are not version-controlled by default.

**Solution:** Store hooks in `hooks/` directory and symlink them to `.git/hooks/`.

**Benefits:**
- Team shares same validation rules
- Hooks update automatically via `git pull`
- Single source of truth for hook logic
- Easy onboarding for new developers

### Symlink Structure

```
Repository root
├── hooks/                      # Version-controlled
│   ├── pre-push               # Hook script
│   ├── install-hooks.sh       # Installation script
│   └── README.md              # This file
└── .git/
    └── hooks/                 # Not version-controlled
        └── pre-push -> ../../hooks/pre-push  # Symlink
```

### Why Pre-Push (Not Pre-Commit)?

**Pre-commit:**
- Runs on every commit
- Can be disruptive for WIP commits
- Slows down local development flow

**Pre-push:**
- Runs only before sharing code
- Allows rapid local iteration
- Validates before CI pipeline
- Catches issues earlier than CI

## Development

### Adding New Hooks

1. Create hook script in `hooks/` directory:
```bash
touch hooks/pre-commit
chmod +x hooks/pre-commit
```

2. Add comprehensive documentation in script comments:
   - Purpose and rationale
   - Checks performed
   - Exit codes
   - Bypass instructions

3. Update `install-hooks.sh` to include new hook:
```bash
HOOKS=(
    "pre-push"
    "pre-commit"  # Add here
)
```

4. Document in this README

5. Test locally before committing:
```bash
./hooks/install-hooks.sh
.git/hooks/pre-commit  # Test directly
```

### Testing Hooks

**Direct execution:**
```bash
.git/hooks/pre-push
```

**Simulate git push:**
```bash
git push --dry-run origin main
```

**Test failure scenarios:**
```bash
# Break formatting
echo "fn broken( ){}" >> src/main.rs
git add src/main.rs
git commit -m "test"
git push  # Should block

# Restore
git reset --hard HEAD~1
```

## Troubleshooting

### Hook not running

**Check symlink exists:**
```bash
ls -la .git/hooks/pre-push
# Should show: lrwxr-xr-x ... pre-push -> ../../hooks/pre-push
```

**Re-install:**
```bash
./hooks/install-hooks.sh
```

### Hook fails incorrectly

**Run checks manually:**
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

**Check hook script output:**
```bash
bash -x .git/hooks/pre-push  # Debug mode
```

### Permissions issue

**Make scripts executable:**
```bash
chmod +x hooks/pre-push hooks/install-hooks.sh
```

## Best Practices

1. **Keep hooks fast:** Slow hooks frustrate developers
2. **Provide clear output:** Show what's being checked and why it failed
3. **Allow bypass:** Emergency situations require flexibility
4. **Document thoroughly:** Explain purpose and usage in comments
5. **Test before committing:** Validate hook changes locally

## References

- [Git Hooks Documentation](https://git-scm.com/docs/githooks)
- [Rust Formatting](https://github.com/rust-lang/rustfmt)
- [Clippy Lints](https://github.com/rust-lang/rust-clippy)
