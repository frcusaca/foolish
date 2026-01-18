#!/bin/bash
# SessionStart hook for Foolish project
# Automatically runs on every Claude Code session start

set -e

echo "🚀 Foolish Project SessionStart Hook"
echo "========================================"

# Get project root (where .claude directory is)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# Detect if running in Claude Code Web
if [ -n "$CLAUDECODE" ]; then
    echo ""
    echo "📍 Claude Code Web detected"
    echo ""

    # Run CCW setup automatically
    echo "🔧 Running CCW Maven setup..."
    if [ -f ".claude/skills/ccw-maven-setup/prep_if_ccw.sh" ]; then
        bash .claude/skills/ccw-maven-setup/prep_if_ccw.sh
    elif [ -f "support/shared/claude/skills/ccw-maven-setup/prep_if_ccw.sh" ]; then
        bash support/shared/claude/skills/ccw-maven-setup/prep_if_ccw.sh
    else
        echo "⚠️  Warning: ccw-maven-setup script not found!"
    fi

    echo ""
    echo "✅ CCW setup complete"
fi

echo ""
echo "⚠️  CRITICAL REMINDER FOR AI AGENTS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  🚫 NEVER use 'mvn' directly"
echo "  ✅ ALWAYS use './mvn_cmd' instead"
echo ""
echo "  Examples:"
echo "    ✅ ./mvn_cmd clean test"
echo "    ✅ ./mvn_cmd compile -DskipTests"
echo "    ❌ mvn clean test"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create mvn wrapper function to catch direct usage
# This creates a shell function that intercepts 'mvn' calls
cat > /tmp/foolish-mvn-wrapper.sh <<'WRAPPEREOF'
# Override mvn command to redirect to mvn_cmd
mvn() {
    if [ -f "./mvn_cmd" ]; then
        echo "⚠️  Intercepted 'mvn' call - redirecting to './mvn_cmd'"
        echo "   Please use './mvn_cmd' directly in future"
        echo ""
        ./mvn_cmd "$@"
    else
        echo "❌ ERROR: mvn_cmd not found in current directory"
        echo "   You must use './mvn_cmd' from project root"
        return 1
    fi
}
export -f mvn
WRAPPEREOF

echo "💡 Shell wrapper installed: 'mvn' will auto-redirect to './mvn_cmd'"
echo "   (This is a safety net - please use './mvn_cmd' directly)"
echo ""

# Source the wrapper in current shell (if interactive)
if [ -n "$BASH_VERSION" ]; then
    source /tmp/foolish-mvn-wrapper.sh 2>/dev/null || true
fi

echo "✨ SessionStart complete - ready to build!"
echo ""
