package org.foolish.lsp;

import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;

import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.eclipse.lsp4j.Diagnostic;
import org.eclipse.lsp4j.DidChangeTextDocumentParams;
import org.eclipse.lsp4j.DidCloseTextDocumentParams;
import org.eclipse.lsp4j.DidOpenTextDocumentParams;
import org.eclipse.lsp4j.DidSaveTextDocumentParams;
import org.eclipse.lsp4j.PublishDiagnosticsParams;
import org.eclipse.lsp4j.TextDocumentSyncKind;
import org.eclipse.lsp4j.TextEdit;
import org.eclipse.lsp4j.WillSaveTextDocumentParams;
import org.eclipse.lsp4j.services.LanguageClient;
import org.eclipse.lsp4j.services.TextDocumentService;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Text document service with parsing and diagnostics support.
 */
public final class FoolishTextDocumentService implements TextDocumentService {
    private static final Logger LOGGER = LoggerFactory.getLogger(FoolishTextDocumentService.class);

    private final Map<String, String> documents = new ConcurrentHashMap<>();
    private LanguageClient client;

    public TextDocumentSyncKind getSyncKind() {
        return TextDocumentSyncKind.Full;
    }

    public void connect(LanguageClient client) {
        this.client = client;
    }

    @Override
    public void didOpen(DidOpenTextDocumentParams params) {
        String uri = params.getTextDocument().getUri();
        String content = params.getTextDocument().getText();
        LOGGER.info("Opened {}", uri);
        
        documents.put(uri, content);
        validateAndPublishDiagnostics(uri, content);
    }

    @Override
    public void didChange(DidChangeTextDocumentParams params) {
        String uri = params.getTextDocument().getUri();
        String content = params.getContentChanges().get(0).getText();
        LOGGER.debug("Document changed {}", uri);
        
        documents.put(uri, content);
        validateAndPublishDiagnostics(uri, content);
    }

    @Override
    public void didClose(DidCloseTextDocumentParams params) {
        String uri = params.getTextDocument().getUri();
        LOGGER.info("Closed {}", uri);
        
        documents.remove(uri);
        // Clear diagnostics on close
        if (client != null) {
            PublishDiagnosticsParams diagnosticsParams = new PublishDiagnosticsParams();
            diagnosticsParams.setUri(uri);
            diagnosticsParams.setDiagnostics(Collections.emptyList());
            client.publishDiagnostics(diagnosticsParams);
        }
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

    private void validateAndPublishDiagnostics(String uri, String content) {
        List<Diagnostic> diagnostics = parseAndCollectDiagnostics(content);
        
        if (client != null) {
            PublishDiagnosticsParams diagnosticsParams = new PublishDiagnosticsParams();
            diagnosticsParams.setUri(uri);
            diagnosticsParams.setDiagnostics(diagnostics);
            client.publishDiagnostics(diagnosticsParams);
            
            LOGGER.debug("Published {} diagnostics for {}", diagnostics.size(), uri);
        }
    }

    private List<Diagnostic> parseAndCollectDiagnostics(String content) {
        try {
            CharStream input = CharStreams.fromString(content);
            FoolishLexer lexer = new FoolishLexer(input);
            CommonTokenStream tokens = new CommonTokenStream(lexer);
            FoolishParser parser = new FoolishParser(tokens);
            
            // Remove default error listeners and add our collector
            DiagnosticCollector collector = new DiagnosticCollector();
            parser.removeErrorListeners();
            lexer.removeErrorListeners();
            parser.addErrorListener(collector);
            lexer.addErrorListener(collector);
            
            // Parse the program
            parser.program();
            
            return collector.getDiagnostics();
        } catch (Exception e) {
            LOGGER.error("Error parsing document", e);
            return Collections.emptyList();
        }
    }
}
