#!/bin/bash
set -e

# 1. Capture the JSON metadata from stdin
INPUT_JSON=$(cat)

# 2. Extract session_id without jq (using grep and sed)
SESSION_ID=$(echo "$INPUT_JSON" | grep -o '"session_id":"[^"]*"' | sed 's/"session_id":"//;s/"//')

# 3. Fallback: If grep failed, try to get it from the transcript_path
if [ -z "$SESSION_ID" ]; then
    TRANSCRIPT_PATH=$(echo "$INPUT_JSON" | grep -o '"transcript_path":"[^"]*"' | sed 's/"transcript_path":"//;s/"//')
    # Extracts the UUID filename from the end of the path
    SESSION_ID=$(basename "$TRANSCRIPT_PATH" .jsonl)
fi

# 3.5 WTF
if [ -z "$CLAUDE_ENV_FILE" ]; then
    CLAUDE_HOME="${CLAUDE_HOME:-$HOME/.claude}"
    mkdir -p "$CLAUDE_HOME/session-env/$SESSION_ID"
    CLAUDE_ENV_FILE="$CLAUDE_HOME/session-env/$SESSION_ID/hook-0.sh"
fi

# 4. Persist to the Claude environment file
if [ -n "$CLAUDE_ENV_FILE" ] && [ -n "$SESSION_ID" ]; then
    echo "export CLAUDE_SESSION_ID=\"$SESSION_ID\"" >> "$CLAUDE_ENV_FILE"
fi

# Modified version of prep_if_ccw.sh for offline environments
# Skips SDKMAN installation, uses existing SDKMAN if available

# Get script directory for accessing repo-vendored files

echo "🔧 Setting up Java 25 (offline mode)..."
echo "⏱️  This setup ensures Java 25 is available for your session..."


export MAVEN_OPTS="-o -Dmaven.repo.local=$CLAUDE_PROJECT_DIR/claude_mvn/repo $MAVEN_OPTS"
echo "MAVEN_OPTS=\"-Dmaven.repo.local=$CLAUDE_PROJECT_DIR/claude_mvn/repo $MAVEN_OPTS\"" >> "$CLAUDE_ENV_FILE"

# Source SDKMAN from repo-vendored copy or home directory (if it exists)
if [ -f "$HOME/.sdkman/bin/sdkman-init.sh" ]; then
    echo "📦 Found SDKMAN in home directory, sourcing..."
    source "$HOME/.sdkman/bin/sdkman-init.sh"
else
    echo "Install SDKMAN"
    curl -s "https://get.sdkman.io" | bash
    source "$HOME/.sdkman/bin/sdkman-init.sh"
fi

# Let's set the path for now, it won't hurt
echo "export PATH=\"$PATH\""  >> "$CLAUDE_ENV_FILE"

# Install Java 25 if needed (latest stable Temurin)
echo "🔍 Checking for Java 25..."
if ! sdk list java | grep -q "25\..*-tem.*installed"; then
    echo "☕ Installing latest stable Java 25 (Temurin)..."
    echo "   (This may take 30-60 seconds on first run)"
    # Get the latest 25.x Temurin version
    JAVA_VERSION=$(sdk list java 2>/dev/null | grep "tem" | grep "25\." | grep -v "fx\|ea" | head -1 | awk '{print $NF}')
    if [ -n "$JAVA_VERSION" ]; then
        echo "   Installing Java $JAVA_VERSION..."
        sdk install java "$JAVA_VERSION" 2>&1 >> /dev/null
        echo "   ✅ Java $JAVA_VERSION installed successfully"
    else
        echo "⚠️  Could not find Java 25 Temurin, trying default..."
        sdk install java 25-tem 2>&1 >> /dev/null
    fi
   sdk install maven 2>&1 >> /dev/null
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
echo "export JAVA_HOME=\"$HOME/.sdkman/candidates/java/current\"" >> "$CLAUDE_ENV_FILE"
export PATH="$JAVA_HOME/bin:$PATH"
echo "export PATH=\"$JAVA_HOME/bin:$PATH\""  >> "$CLAUDE_ENV_FILE"
echo "  "
echo "  "
echo "  "
echo "echo NOTE TO AI: Please set these environment variables:"
cat "$CLAUDE_ENV_FILE"
echo "echo END OF required environment setting lines."
echo "  "
echo "  "
echo "  "
#echo ""
#echo "✨ Java 25 setup complete!"
#echo ""
#echo "📊 Environment status:"
#echo "   Java version:"
#java -version 2>&1 | sed 's/^/   /' | head -1
#echo "   JAVA_HOME: $JAVA_HOME"
#echo ""
