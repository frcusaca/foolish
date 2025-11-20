const path = require('path');
const { workspace, ExtensionContext } = require('vscode');
const {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions
} = require('vscode-languageclient/node');

let client;

function activate(context) {
    // Path to the LSP server JAR
    const serverJar = path.join(
        context.extensionPath,
        '..',
        'foolish-lsp-java',
        'target',
        'foolish-lsp-java-1.0-SNAPSHOT.jar'
    );

    // Server options - launch the Java LSP server
    const serverOptions = {
        run: {
            command: 'java',
            args: ['-jar', serverJar]
        },
        debug: {
            command: 'java',
            args: ['-jar', serverJar]
        }
    };

    // Client options - configure which files to monitor
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'foolish' }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.foo')
        }
    };

    // Create and start the language client
    client = new LanguageClient(
        'foolish',
        'Foolish Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

module.exports = {
    activate,
    deactivate
};
