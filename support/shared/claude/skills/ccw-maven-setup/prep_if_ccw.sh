#!/bin/bash
# Prepare Maven environment for Claude Code Web (CCW)
# Does nothing if not in CCW environment

set -e

# Detect CCW environment
if [ -z "$CLAUDECODE" ]; then
    echo "ℹ️  Not in Claude Code Web - no setup needed"
    exit 0
fi

echo "🔧 Claude Code Web detected - setting up Java and Maven proxy..."

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Use local SDKMAN stub (SDKMAN installation blocked by CCW proxy)
SDKMAN_INIT="$SCRIPT_DIR/sdkman-stub/sdkman-init.sh"

if [ -f "$SDKMAN_INIT" ]; then
    echo "📦 Using local SDKMAN stub..."
    source "$SDKMAN_INIT"
else
    echo "⚠️  Warning: SDKMAN stub not found at $SDKMAN_INIT"
    echo "Creating minimal stub..."
    mkdir -p "$SCRIPT_DIR/sdkman-stub"
    cat > "$SDKMAN_INIT" <<'STUBEOF'
#!/bin/bash
sdk() {
    case "$1" in
        "list"|"install"|"use") echo "Using system Java: $(java -version 2>&1 | head -1)";;
        "version") echo "SDKMAN stub v1.0.0";;
    esac
}
export -f sdk
STUBEOF
    source "$SDKMAN_INIT"
fi

# Check Java version
JAVA_VERSION=$(java -version 2>&1 | head -1 | awk -F'"' '{print $2}')
echo "☕ Using system Java: $JAVA_VERSION"

if [[ ! "$JAVA_VERSION" == 21.* ]] && [[ ! "$JAVA_VERSION" == 25.* ]]; then
    echo "⚠️  Warning: Java version is not 21 or 25. Build may fail."
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
java -version
