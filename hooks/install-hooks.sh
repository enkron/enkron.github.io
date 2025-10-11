#!/usr/bin/env bash
#
# Git hooks installation script
#
# PURPOSE:
#   Installs version-controlled git hooks into .git/hooks/ directory using symlinks.
#   Allows git hooks to be shared across team and updated via git pull.
#
# USAGE:
#   Run from repository root:
#     ./hooks/install-hooks.sh
#
# DESIGN:
#   - Creates symlinks from .git/hooks/ to hooks/ directory
#   - Symlinks auto-update when hooks/ content changes via git pull
#   - Preserves executable permissions
#   - Safe to run multiple times (idempotent)
#
# STRUCTURE:
#   hooks/              - Version-controlled hook scripts
#   .git/hooks/         - Git's hook directory (not version-controlled)
#   .git/hooks/pre-push -> ../../hooks/pre-push (symlink)
#
# WHY SYMLINKS:
#   1. Version control: Hooks stored in repository, shared with team
#   2. Auto-update: Changes to hooks/ propagate automatically
#   3. Discoverability: New developers run one command to set up
#   4. Maintenance: Single source of truth for hook logic

set -e

# Color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Git Hooks Installation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Verify we're in repository root
if [ ! -d ".git" ]; then
    echo -e "${YELLOW}Error: Must be run from repository root${NC}"
    echo "  Current directory: $(pwd)"
    echo "  Expected: .git/ directory to exist"
    exit 1
fi

# Verify hooks directory exists
if [ ! -d "hooks" ]; then
    echo -e "${YELLOW}Error: hooks/ directory not found${NC}"
    echo "  Expected location: $(pwd)/hooks"
    exit 1
fi

# Create .git/hooks directory if it doesn't exist
mkdir -p .git/hooks

# List of hooks to install
HOOKS=(
    "pre-push"
)

# Install each hook
for hook in "${HOOKS[@]}"; do
    SOURCE="../../hooks/$hook"
    TARGET=".git/hooks/$hook"

    echo -e "${BLUE}Installing $hook hook...${NC}"

    # Remove existing hook (file or symlink)
    if [ -e "$TARGET" ] || [ -L "$TARGET" ]; then
        echo "  Removing existing hook: $TARGET"
        rm "$TARGET"
    fi

    # Create symlink
    ln -sf "$SOURCE" "$TARGET"

    # Verify symlink was created
    if [ -L "$TARGET" ]; then
        echo -e "  ${GREEN}✓ Created symlink: $TARGET -> $SOURCE${NC}"
    else
        echo -e "  ${YELLOW}✗ Failed to create symlink${NC}"
        exit 1
    fi

    # Ensure source file is executable
    chmod +x "hooks/$hook"
    echo "  Set executable permission on hooks/$hook"
    echo ""
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✓ Git hooks installed successfully${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Installed hooks:"
for hook in "${HOOKS[@]}"; do
    echo "  - $hook: Runs before git push"
done
echo ""
echo "To bypass hooks (not recommended):"
echo "  git push --no-verify"
echo ""
