#!/bin/bash
# SessionStart hook for Claude Code Web (CCW) environments
# This script sets up the Java environment and Maven proxy for CCW sessions

# Check if we're in Claude Code Web environment
if [ -n "$CLAUDECODE" ] || [ "$CLAUDE_CODE_REMOTE_ENVIRONMENT_TYPE" = "cloud_default" ]; then
    echo "🔧 Detected Claude Code Web environment - setting up Java and Maven..."

    # Install sdkman if not already installed
    if [ ! -d "$HOME/.sdkman" ]; then
        echo "📦 Installing SDKMAN..."
        curl -s "https://get.sdkman.io" | bash
    fi

    # Source sdkman
    source "$HOME/.sdkman/bin/sdkman-init.sh"

    # Install Java 25 if not already installed
    if ! sdk list java | grep -q "25.0.1-tem.*installed"; then
        echo "☕ Installing Java 25 (Temurin)..."
        sdk install java 25.0.1-tem
    else
        echo "✅ Java 25 already installed"
        sdk use java 25.0.1-tem
    fi

    # Set up Maven proxy for CCW
    echo "🔌 Setting up Maven proxy..."

    # Extract proxy info from environment
    PROXY_URL="${HTTPS_PROXY:-$https_proxy}"
    if [ -n "$PROXY_URL" ]; then
        # Parse proxy URL: http://user:pass@host:port
        PROXY_HOST=$(echo "$PROXY_URL" | sed -E 's|.*@([^:]+):.*|\1|')
        PROXY_PORT=$(echo "$PROXY_URL" | sed -E 's|.*:([0-9]+)$|\1|')

        echo "  Proxy host: $PROXY_HOST"
        echo "  Proxy port: $PROXY_PORT"

        # Start local Maven proxy if not already running
        if ! pgrep -f "maven-proxy.py" > /dev/null; then
            echo "🚀 Starting local Maven authentication proxy..."

            # Create the proxy script
            cat > /tmp/maven-proxy.py <<'PROXY_EOF'
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
    print(f"Local proxy on 127.0.0.1:{LOCAL_PORT}")
    while True:
        c, _ = srv.accept()
        threading.Thread(target=handle, args=(c,), daemon=True).start()
PROXY_EOF

            chmod +x /tmp/maven-proxy.py
            python3 /tmp/maven-proxy.py > /tmp/maven-proxy.log 2>&1 &
            sleep 2
            echo "  ✅ Maven proxy started at 127.0.0.1:3128"
        else
            echo "  ✅ Maven proxy already running"
        fi

        # Configure Maven settings.xml
        mkdir -p "$HOME/.m2"
        cat > "$HOME/.m2/settings.xml" <<'SETTINGS_EOF'
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
SETTINGS_EOF
        echo "  ✅ Maven settings.xml configured"
    fi

    echo "✨ Claude Code Web setup complete!"
    echo ""
    echo "Java version:"
    java -version
    echo ""
else
    echo "ℹ️  Not in Claude Code Web environment - skipping CCW-specific setup"
fi
