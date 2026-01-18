#!/bin/bash
# Verify Foolish project setup for both local and CCW environments

set -e

echo "🔍 Foolish Project Setup Verification"
echo "======================================"
echo ""

# Get project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

ERRORS=0
WARNINGS=0

# Check 1: mvn_cmd exists and is executable
echo "✓ Checking mvn_cmd wrapper..."
if [ ! -f "./mvn_cmd" ]; then
    echo "  ❌ ERROR: mvn_cmd not found in project root"
    ERRORS=$((ERRORS + 1))
elif [ ! -x "./mvn_cmd" ]; then
    echo "  ❌ ERROR: mvn_cmd is not executable"
    ERRORS=$((ERRORS + 1))
else
    echo "  ✅ mvn_cmd found and executable"
fi

# Check 2: SessionStart hook exists
echo ""
echo "✓ Checking SessionStart hook..."
if [ ! -f ".claude/hooks/SessionStart.sh" ]; then
    echo "  ⚠️  WARNING: SessionStart hook not found"
    echo "     This hook auto-runs CCW setup and enforces mvn_cmd usage"
    WARNINGS=$((WARNINGS + 1))
elif [ ! -x ".claude/hooks/SessionStart.sh" ]; then
    echo "  ⚠️  WARNING: SessionStart hook is not executable"
    WARNINGS=$((WARNINGS + 1))
else
    echo "  ✅ SessionStart hook found and executable"
fi

# Check 3: CCW setup skill exists
echo ""
echo "✓ Checking ccw-maven-setup skill..."
if [ -f ".claude/skills/ccw-maven-setup/prep_if_ccw.sh" ]; then
    echo "  ✅ ccw-maven-setup skill found (via symlink)"
elif [ -f "support/shared/claude/skills/ccw-maven-setup/prep_if_ccw.sh" ]; then
    echo "  ✅ ccw-maven-setup skill found (in support/)"
else
    echo "  ❌ ERROR: ccw-maven-setup skill not found"
    ERRORS=$((ERRORS + 1))
fi

# Check 4: Environment detection
echo ""
echo "✓ Detecting environment..."
if [ -n "$CLAUDECODE" ]; then
    echo "  📍 Running in Claude Code Web (CCW)"

    # CCW-specific checks
    echo ""
    echo "✓ CCW-specific checks..."

    # Check Java version
    if command -v java >/dev/null 2>&1; then
        JAVA_VERSION=$(java -version 2>&1 | head -1 | cut -d'"' -f2 | cut -d'.' -f1)
        if [ "$JAVA_VERSION" = "25" ]; then
            echo "  ✅ Java 25 installed"
        else
            echo "  ⚠️  WARNING: Java version is $JAVA_VERSION (expected 25)"
            echo "     Run: /skill ccw-maven-setup"
            WARNINGS=$((WARNINGS + 1))
        fi
    else
        echo "  ❌ ERROR: Java not found"
        echo "     Run: /skill ccw-maven-setup"
        ERRORS=$((ERRORS + 1))
    fi

    # Check Maven proxy
    if pgrep -f "maven-proxy.py" > /dev/null; then
        echo "  ✅ Maven proxy running on 127.0.0.1:3128"
    else
        echo "  ⚠️  WARNING: Maven proxy not running"
        echo "     Run: /skill ccw-maven-setup"
        WARNINGS=$((WARNINGS + 1))
    fi

    # Check Maven settings.xml
    if [ -f "$HOME/.m2/settings.xml" ]; then
        if grep -q "127.0.0.1" "$HOME/.m2/settings.xml" 2>/dev/null; then
            echo "  ✅ Maven settings.xml configured with proxy"
        else
            echo "  ⚠️  WARNING: Maven settings.xml exists but may not have proxy config"
            WARNINGS=$((WARNINGS + 1))
        fi
    else
        echo "  ⚠️  WARNING: Maven settings.xml not found"
        echo "     Run: /skill ccw-maven-setup"
        WARNINGS=$((WARNINGS + 1))
    fi
else
    echo "  💻 Running in local environment"

    # Local-specific checks
    echo ""
    echo "✓ Local environment checks..."

    # Check Java version
    if command -v java >/dev/null 2>&1; then
        JAVA_VERSION=$(java -version 2>&1 | head -1 | cut -d'"' -f2 | cut -d'.' -f1)
        if [ "$JAVA_VERSION" = "25" ]; then
            echo "  ✅ Java 25 installed"
        else
            echo "  ⚠️  WARNING: Java version is $JAVA_VERSION (expected 25)"
            WARNINGS=$((WARNINGS + 1))
        fi
    else
        echo "  ❌ ERROR: Java not found"
        ERRORS=$((ERRORS + 1))
    fi
fi

# Check 5: AGENTS.md and CLAUDE.md exist
echo ""
echo "✓ Checking documentation..."
if [ -f "AGENTS.md" ]; then
    echo "  ✅ AGENTS.md found"
else
    echo "  ❌ ERROR: AGENTS.md not found"
    ERRORS=$((ERRORS + 1))
fi

if [ -f ".claude/CLAUDE.md" ]; then
    echo "  ✅ CLAUDE.md found"
else
    echo "  ⚠️  WARNING: .claude/CLAUDE.md not found"
    WARNINGS=$((WARNINGS + 1))
fi

# Summary
echo ""
echo "======================================"
echo "Summary:"
echo ""

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo "✨ All checks passed! Setup is correct."
    echo ""
    echo "You can now run:"
    echo "  ./mvn_cmd clean test"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo "⚠️  Setup has $WARNINGS warning(s) but should work."
    echo ""
    echo "Consider addressing warnings for optimal experience."
    exit 0
else
    echo "❌ Setup has $ERRORS error(s) and $WARNINGS warning(s)."
    echo ""
    echo "Please fix errors before building."
    exit 1
fi
