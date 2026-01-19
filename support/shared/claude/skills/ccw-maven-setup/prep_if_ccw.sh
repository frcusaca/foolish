#!/bin/bash
# Prepare Maven environment for Claude Code Web (CCW)
# Does nothing if not in CCW environment

set -e

# Get script directory for accessing repo-vendored files
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Detect CCW environment
if [ -z "$CLAUDECODE" ]; then
    echo "ℹ️  Not in Claude Code Web - no setup needed"
    exit 0
fi

# Run synchronously to ensure Java 25 and Maven proxy are ready before session starts
# This is critical infrastructure - async mode would create race conditions where
# Maven commands could fail if run before the proxy is ready
echo "🔧 Claude Code Web detected - setting up Java 25 and Maven proxy..."
echo "⏱️  This setup ensures your session is ready for Maven builds..."
echo "    (First run: ~60s, subsequent runs: ~10s due to caching)"

# Install SDKMAN if needed
if [ ! -d "$HOME/.sdkman" ]; then
    echo "📦 Installing SDKMAN..."
    curl -s "https://get.sdkman.io" | bash
fi

# Source SDKMAN from repo-vendored copy (version-controlled)
source "$HOME/.sdkman/bin/sdkman-init.sh" || source "$SCRIPT_DIR/sdkman-init.sh"


# Install Java 25 if needed (latest stable Temurin)
echo "🔍 Checking for Java 25..."
if ! sdk list java 2>/dev/null | grep -q "25\..*-tem.*installed"; then
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

# If CLAUDE_ENV_FILE is available, persist for the entire session
if [ -n "$CLAUDE_ENV_FILE" ]; then
    echo "export JAVA_HOME=\"$JAVA_HOME\"" >> "$CLAUDE_ENV_FILE"
    echo "export PATH=\"\$JAVA_HOME/bin:\$PATH\"" >> "$CLAUDE_ENV_FILE"
    echo "  ✅ Java 25 environment persisted to session"
fi

# Setup Maven proxy for CCW
PROXY_URL="${HTTPS_PROXY:-$https_proxy}"
if [ -z "$PROXY_URL" ]; then
    echo "⚠️  No proxy detected - skipping proxy setup"
    exit 0
fi

echo "🔌 Setting up Maven proxy..."

# Start local proxy if not running
if ! pgrep -f "maven-proxy.py" > /dev/null; then
    echo "🚀 Starting Maven authentication proxy..."

    cat > /tmp/maven-proxy.py <<'PROXYEOF'
#!/usr/bin/env python3
"""Local proxy that adds auth when forwarding to upstream proxy."""
import socket, threading, os, base64, select
from urllib.parse import urlparse

LOCAL_PORT = 3128
UPSTREAM = os.environ.get('https_proxy') or os.environ.get('HTTPS_PROXY')

def get_upstream():
    p = urlparse(UPSTREAM)
    return p.hostname, p.port, p.username or '', p.password or ''

def handle(client):
    try:
        req = b''
        while b'\r\n\r\n' not in req:
            req += client.recv(4096)
        target = req.split(b'\r\n')[0].split()[1].decode()
        host, port = (target.split(':') + ['443'])[:2]
        proxy_host, proxy_port, user, pwd = get_upstream()
        auth = base64.b64encode(f"{user}:{pwd}".encode()).decode()
        upstream = socket.socket()
        upstream.connect((proxy_host, proxy_port))
        upstream.send(f"CONNECT {host}:{port} HTTP/1.1\r\nProxy-Authorization: Basic {auth}\r\n\r\n".encode())
        resp = b''
        while b'\r\n\r\n' not in resp:
            resp += upstream.recv(4096)
        if b'200' in resp.split(b'\r\n')[0]:
            client.send(b'HTTP/1.1 200 Connection Established\r\n\r\n')
            for s in [client, upstream]: s.setblocking(False)
            while True:
                r, _, _ = select.select([client, upstream], [], [], 30)
                if not r: break
                for s in r:
                    data = s.recv(8192)
                    if not data: return
                    (upstream if s is client else client).sendall(data)
    except: pass
    finally: client.close()

if __name__ == '__main__':
    srv = socket.socket()
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(('127.0.0.1', LOCAL_PORT))
    srv.listen(10)
    print(f"Maven proxy listening on 127.0.0.1:{LOCAL_PORT}")
    while True:
        c, _ = srv.accept()
        threading.Thread(target=handle, args=(c,), daemon=True).start()
PROXYEOF

    chmod +x /tmp/maven-proxy.py
    python3 /tmp/maven-proxy.py > /tmp/maven-proxy.log 2>&1 &

    # Wait for proxy to be ready (with timeout)
    echo "  ⏳ Waiting for proxy to start..."
    TIMEOUT=10
    ELAPSED=0
    while ! nc -z 127.0.0.1 3128 2>/dev/null; do
        if [ $ELAPSED -ge $TIMEOUT ]; then
            echo "  ⚠️  WARNING: Proxy did not start within ${TIMEOUT}s"
            echo "  Check /tmp/maven-proxy.log for errors"
            break
        fi
        sleep 0.5
        ELAPSED=$((ELAPSED + 1))
    done

    if nc -z 127.0.0.1 3128 2>/dev/null; then
        echo "  ✅ Proxy started and listening on 127.0.0.1:3128"
    fi
else
    echo "  ✅ Proxy already running"
fi

# Configure Maven settings
echo "  📝 Configuring Maven settings.xml..."
mkdir -p "$HOME/.m2"
cat > "$HOME/.m2/settings.xml" <<'SETTINGSEOF'
<?xml version="1.0" encoding="UTF-8"?>
<settings xmlns="http://maven.apache.org/SETTINGS/1.0.0"
          xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
          xsi:schemaLocation="http://maven.apache.org/SETTINGS/1.0.0
                              http://maven.apache.org/xsd/settings-1.0.0.xsd">
  <proxies>
    <!-- HTTP proxy for Maven Central -->
    <proxy>
      <id>local-http</id>
      <active>true</active>
      <protocol>http</protocol>
      <host>127.0.0.1</host>
      <port>3128</port>
      <nonProxyHosts>localhost|127.0.0.1</nonProxyHosts>
    </proxy>
    <!-- HTTPS proxy for Maven Central -->
    <proxy>
      <id>local-https</id>
      <active>true</active>
      <protocol>https</protocol>
      <host>127.0.0.1</host>
      <port>3128</port>
      <nonProxyHosts>localhost|127.0.0.1</nonProxyHosts>
    </proxy>
  </proxies>
</settings>
SETTINGSEOF
echo "  ✅ Maven settings.xml configured (HTTP + HTTPS)"

# Configure Java truststore for CCW proxy CA certificate
echo "  🔐 Setting up Java truststore for CCW proxy..."
TRUSTSTORE="$HOME/.m2/ccw-truststore.jks"
STOREPASS="${STOREPASS:-changeit}"

# Try to extract the CA certificate from the CCW proxy
# The proxy might use a self-signed cert or corporate CA
CA_PEM="$HOME/.m2/ccw-proxy-ca.pem"

# Attempt to extract the CA cert from the proxy connection
# This uses openssl to connect and extract the certificate chain
if command -v openssl >/dev/null 2>&1; then
    # Extract proxy host from HTTPS_PROXY
    PROXY_HOST=$(echo "$HTTPS_PROXY" | sed -E 's|https?://([^:@]*:)?([^:@]*)@||' | sed -E 's|https?://||' | cut -d':' -f1)
    PROXY_PORT=$(echo "$HTTPS_PROXY" | sed -E 's|https?://([^:@]*:)?([^:@]*)@||' | sed -E 's|https?://||' | cut -d':' -f2 | cut -d'/' -f1)

    # Try to get certificate from a known HTTPS endpoint through the proxy
    if timeout 5 openssl s_client -connect repo.maven.apache.org:443 -proxy "$PROXY_HOST:$PROXY_PORT" -showcerts </dev/null 2>/dev/null | \
       openssl x509 -outform PEM > "$CA_PEM" 2>/dev/null; then
        echo "  📜 Extracted CA certificate from proxy"
    else
        echo "  ℹ️  Could not extract CA certificate automatically"
        rm -f "$CA_PEM"
    fi
fi

# If we have a CA certificate (either extracted or provided), import it
if [ -f "$CA_PEM" ]; then
    echo "  🔧 Importing CA certificate into Java truststore..."

    # Remove old truststore if it exists
    rm -f "$TRUSTSTORE"

    # Import the CA certificate
    if keytool -importcert -noprompt \
        -alias ccw-proxy-ca \
        -file "$CA_PEM" \
        -keystore "$TRUSTSTORE" \
        -storepass "$STOREPASS" >/dev/null 2>&1; then

        echo "  ✅ CA certificate imported successfully"

        # Set MAVEN_OPTS to use the custom truststore
        export MAVEN_OPTS="${MAVEN_OPTS:-} -Djavax.net.ssl.trustStore=$TRUSTSTORE -Djavax.net.ssl.trustStorePassword=$STOREPASS"

        # Persist to session if CLAUDE_ENV_FILE is available
        if [ -n "$CLAUDE_ENV_FILE" ]; then
            echo "export MAVEN_OPTS=\"\${MAVEN_OPTS:-} -Djavax.net.ssl.trustStore=$TRUSTSTORE -Djavax.net.ssl.trustStorePassword=$STOREPASS\"" >> "$CLAUDE_ENV_FILE"
        fi
    else
        echo "  ⚠️  Failed to import CA certificate"
    fi
else
    echo "  ℹ️  No CA certificate found - PKIX errors may occur for HTTPS downloads"
    echo "  📝 To fix PKIX errors, place CA cert at: $SCRIPT_DIR/ccw-proxy-ca.pem"
fi

echo ""
echo "✨ CCW setup complete!"
echo ""
echo "📊 Environment status:"
echo "   Java version:"
java -version 2>&1 | sed 's/^/   /' | head -1
echo ""
echo "   Maven proxy: 127.0.0.1:3128 (HTTP + HTTPS)"
echo "   Maven settings: $HOME/.m2/settings.xml"
if [ -f "$TRUSTSTORE" ]; then
    echo "   Java truststore: $TRUSTSTORE"
fi
echo "   Proxy log: /tmp/maven-proxy.log"
echo ""
echo "✅ Your session is ready for Maven builds!"
echo "   You can now run: mvn clean test"
echo ""
echo "📚 This setup implements the fix from:"
echo "   - GitHub Issue #13372 (Maven/Gradle proxy authentication)"
echo "   - LinkedIn article by Tarun Lalwani"
