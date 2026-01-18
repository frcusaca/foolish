#!/bin/bash
# Install ccw-maven-setup skill to ~/.claude/skills/
# This makes the skill available globally for Claude Code

set -e

echo "📦 Installing ccw-maven-setup skill to ~/.claude/skills/"
echo ""

# Get project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Source directory (in project repo)
SOURCE_DIR="$PROJECT_ROOT/support/shared/claude/skills/ccw-maven-setup"

# Destination directory (user's home .claude)
DEST_DIR="$HOME/.claude/skills/ccw-maven-setup"

# Check if source exists
if [ ! -d "$SOURCE_DIR" ]; then
    echo "❌ Error: Source directory not found: $SOURCE_DIR"
    exit 1
fi

# Create destination directory
mkdir -p "$HOME/.claude/skills"

# Copy skill files
echo "📋 Copying skill files..."
cp -r "$SOURCE_DIR" "$DEST_DIR"

# Make prep script executable
chmod +x "$DEST_DIR/prep_if_ccw.sh"

echo "✅ Installation complete!"
echo ""
echo "Skill installed to: $DEST_DIR"
echo ""
echo "You can now use the skill from any project:"
echo "  /skill ccw-maven-setup"
echo ""
echo "Or run the script directly:"
echo "  ~/.claude/skills/ccw-maven-setup/prep_if_ccw.sh"
echo ""
