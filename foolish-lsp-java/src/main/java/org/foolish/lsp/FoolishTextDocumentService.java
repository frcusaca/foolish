package org.foolish.lsp;

import java.util.Collections;
import java.util.concurrent.CompletableFuture;

import org.eclipse.lsp4j.DidChangeTextDocumentParams;
import org.eclipse.lsp4j.DidCloseTextDocumentParams;
import org.eclipse.lsp4j.DidOpenTextDocumentParams;
import org.eclipse.lsp4j.DidSaveTextDocumentParams;
import org.eclipse.lsp4j.TextDocumentSyncKind;
import org.eclipse.lsp4j.TextEdit;
import org.eclipse.lsp4j.WillSaveTextDocumentParams;
import org.eclipse.lsp4j.services.TextDocumentService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Lightweight placeholder implementation while the parser/semantic engine is wired in.
 */
public final class FoolishTextDocumentService implements TextDocumentService {
    private static final Logger LOGGER = LoggerFactory.getLogger(FoolishTextDocumentService.class);

    public TextDocumentSyncKind getSyncKind() {
        return TextDocumentSyncKind.Full;
    }

    @Override
    public void didOpen(DidOpenTextDocumentParams params) {
        LOGGER.info("Opened {}", params.getTextDocument().getUri());
    }

    @Override
    public void didChange(DidChangeTextDocumentParams params) {
        LOGGER.debug("Document changed {}", params.getTextDocument().getUri());
    }

    @Override
    public void didClose(DidCloseTextDocumentParams params) {
        LOGGER.info("Closed {}", params.getTextDocument().getUri());
    }

    @Override
    public void didSave(DidSaveTextDocumentParams params) {
        LOGGER.info("Saved {}", params.getTextDocument().getUri());
    }

    @Override
    public CompletableFuture<java.util.List<TextEdit>> willSaveWaitUntil(WillSaveTextDocumentParams params) {
        return CompletableFuture.completedFuture(Collections.emptyList());
    }

    @Override
    public void willSave(org.eclipse.lsp4j.WillSaveTextDocumentParams params) {
        LOGGER.debug("Will save {}", params.getTextDocument().getUri());
    }
}
