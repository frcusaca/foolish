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

# Enable async mode - session starts immediately while this runs in background
echo '{"async": true, "asyncTimeout": 300000}'

echo "🔧 Claude Code Web detected - setting up Java 25 and Maven proxy..."
echo "⏱️  This will run in the background while your session starts..."

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
    sleep 2
    echo "  ✅ Proxy started at 127.0.0.1:3128"
else
    echo "  ✅ Proxy already running"
fi

# Configure Maven settings
mkdir -p "$HOME/.m2"
cat > "$HOME/.m2/settings.xml" <<'SETTINGSEOF'
<?xml version="1.0" encoding="UTF-8"?>
<settings xmlns="http://maven.apache.org/SETTINGS/1.0.0"
          xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
          xsi:schemaLocation="http://maven.apache.org/SETTINGS/1.0.0
                              http://maven.apache.org/xsd/settings-1.0.0.xsd">
  <proxies>
    <proxy>
      <id>local</id>
      <active>true</active>
      <protocol>https</protocol>
      <host>127.0.0.1</host>
      <port>3128</port>
    </proxy>
  </proxies>
</settings>
SETTINGSEOF
echo "  ✅ Maven settings.xml configured"

echo "✨ CCW setup complete!"
echo ""
echo "📊 Environment status:"
echo "   Java version:"
java -version 2>&1 | sed 's/^/   /'
echo ""
echo "   Maven proxy: 127.0.0.1:3128"
echo "   Maven settings: $HOME/.m2/settings.xml"
echo ""
echo "✅ Your session is ready for Maven builds!"
echo "   You can now run: mvn clean test"
