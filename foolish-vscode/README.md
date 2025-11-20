# Foolish VS Code Extension

Simple VS Code extension for Foolish language support.

## Features

- Syntax highlighting for `.foo` files
- Real-time syntax error diagnostics via LSP

## Installation

1. Build the LSP server:
   ```bash
   cd /home/user/antigravity
   mvn clean install -pl foolish-lsp-java
   ```

2. Install extension dependencies:
   ```bash
   cd foolish-vscode
   npm install
   ```

3. Open in VS Code:
   ```bash
   code --extensionDevelopmentPath=/home/user/antigravity/foolish-vscode
   ```

## Testing

Create a test file `test.foo`:
```foolish
{
    x = 10;
    y = 20 +;    ! Syntax error - should show red squiggle
    z = x + y;
}
```
