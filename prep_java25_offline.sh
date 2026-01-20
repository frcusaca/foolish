#!/bin/bash
# Modified version of prep_if_ccw.sh for offline environments
# Skips SDKMAN installation, uses existing SDKMAN if available

set -e

# Get script directory for accessing repo-vendored files
SCRIPT_DIR="/home/user/foolish/support/shared/claude/skills/ccw-maven-setup"

echo "🔧 Setting up Java 25 (offline mode)..."
echo "⏱️  This setup ensures Java 25 is available for your session..."

# Source SDKMAN from repo-vendored copy or home directory (if it exists)
if [ -f "$HOME/.sdkman/bin/sdkman-init.sh" ]; then
    echo "📦 Found SDKMAN in home directory, sourcing..."
    source "$HOME/.sdkman/bin/sdkman-init.sh"
elif [ -f "$SCRIPT_DIR/sdkman-init.sh" ]; then
    echo "📦 Using vendored sdkman-init.sh..."
    source "$SCRIPT_DIR/sdkman-init.sh"
else
    echo "❌ SDKMAN not found in home directory or vendored location"
    echo "   Cannot proceed without SDKMAN"
    exit 1
fi

# Install Java 25 if needed (latest stable Temurin)
echo "🔍 Checking for Java 25..."
if ! sdk list java | grep -q "25\..*-tem.*installed"; then
    echo "☕ Installing latest stable Java 25 (Temurin)..."
    echo "   (This may take 30-60 seconds on first run)"
    # Get the latest 25.x Temurin version
    JAVA_VERSION=$(sdk list java 2>/dev/null | grep "tem" | grep "25\." | grep -v "fx\|ea" | head -1 | awk '{print $NF}')
    if [ -n "$JAVA_VERSION" ]; then
        echo "   Installing Java $JAVA_VERSION..."
        sdk install java "$JAVA_VERSION"
        echo "   ✅ Java $JAVA_VERSION installed successfully"
    else
        echo "⚠️  Could not find Java 25 Temurin, trying default..."
        sdk install java 25-tem
    fi
else
    echo "✅ Java 25 already installed"
    # Use any installed Java 25 Temurin version
    INSTALLED_VERSION=$(sdk list java 2>/dev/null | grep "25\..*-tem.*installed" | head -1 | awk '{print $NF}')
    echo "   Using Java $INSTALLED_VERSION"
    sdk use java "$INSTALLED_VERSION"
fi

# Persist Java 25 environment for the session
JAVA_HOME="$HOME/.sdkman/candidates/java/current"
export JAVA_HOME
export PATH="$JAVA_HOME/bin:$PATH"

echo ""
echo "✨ Java 25 setup complete!"
echo ""
echo "📊 Environment status:"
echo "   Java version:"
java -version 2>&1 | sed 's/^/   /' | head -1
echo "   JAVA_HOME: $JAVA_HOME"
echo ""
