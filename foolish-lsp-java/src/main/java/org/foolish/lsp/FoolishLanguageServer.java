package org.foolish.lsp;

import java.util.concurrent.CompletableFuture;

import org.eclipse.lsp4j.InitializeParams;
import org.eclipse.lsp4j.InitializeResult;
import org.eclipse.lsp4j.ServerCapabilities;
import org.eclipse.lsp4j.TextDocumentSyncKind;
import org.eclipse.lsp4j.WorkspaceFoldersOptions;
import org.eclipse.lsp4j.WorkspaceServerCapabilities;
import org.eclipse.lsp4j.services.LanguageServer;
import org.eclipse.lsp4j.services.TextDocumentService;
import org.eclipse.lsp4j.services.WorkspaceService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Minimal Language Server stub used for the Foolish LSP MVP. This class wires the protocol
 * entry points and exposes a synchronous document sync capability so editors can connect while the
 * remaining features from the roadmap are implemented incrementally.
 */
public final class FoolishLanguageServer implements LanguageServer {
    private static final Logger LOGGER = LoggerFactory.getLogger(FoolishLanguageServer.class);

    private final FoolishTextDocumentService textDocumentService;
    private final FoolishWorkspaceService workspaceService;

    public FoolishLanguageServer() {
        this.textDocumentService = new FoolishTextDocumentService();
        this.workspaceService = new FoolishWorkspaceService();
    }

    @Override
    public CompletableFuture<InitializeResult> initialize(InitializeParams params) {
        String clientName = params.getClientInfo() != null ? params.getClientInfo().getName() : "unknown";
        LOGGER.info("Initializing Foolish LSP for client {}", clientName);
        ServerCapabilities capabilities = new ServerCapabilities();
        capabilities.setTextDocumentSync(TextDocumentSyncKind.Full);

        WorkspaceServerCapabilities workspaceCapabilities = new WorkspaceServerCapabilities();
        WorkspaceFoldersOptions workspaceFoldersOptions = new WorkspaceFoldersOptions();
        workspaceFoldersOptions.setSupported(true);
        workspaceFoldersOptions.setChangeNotifications(Boolean.TRUE);
        workspaceCapabilities.setWorkspaceFolders(workspaceFoldersOptions);
        capabilities.setWorkspace(workspaceCapabilities);

        InitializeResult result = new InitializeResult(capabilities);
        return CompletableFuture.completedFuture(result);
    }

    @Override
    public void initialized(org.eclipse.lsp4j.InitializedParams params) {
        LOGGER.info("Client initialized: {}", params);
    }

    @Override
    public CompletableFuture<Object> shutdown() {
        LOGGER.info("Shutdown requested.");
        return CompletableFuture.completedFuture(null);
    }

    @Override
    public void exit() {
        LOGGER.info("Server exiting");
    }

    @Override
    public TextDocumentService getTextDocumentService() {
        return textDocumentService;
    }

    @Override
    public WorkspaceService getWorkspaceService() {
        return workspaceService;
    }
}
