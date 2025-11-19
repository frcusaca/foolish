package org.foolish.lsp;

import java.io.IOException;
import java.util.concurrent.ExecutionException;

import org.eclipse.lsp4j.launch.LSPLauncher;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * CLI entry point that wires STDIO streams to the Foolish LSP implementation. Editors such as
 * VS Code and IntelliJ can spawn this main class directly.
 */
public final class FoolishLanguageServerLauncher {
    private static final Logger LOGGER = LoggerFactory.getLogger(FoolishLanguageServerLauncher.class);

    private FoolishLanguageServerLauncher() {
    }

    public static void main(String[] args) throws IOException, InterruptedException, ExecutionException {
        FoolishLanguageServer server = new FoolishLanguageServer();
        LOGGER.info("Starting Foolish LSP server");
        var launcher = LSPLauncher.createServerLauncher(server, System.in, System.out);
        launcher.startListening().get();
    }
}
