package org.foolish.lsp;

import org.antlr.v4.runtime.BaseErrorListener;
import org.antlr.v4.runtime.RecognitionException;
import org.antlr.v4.runtime.Recognizer;
import org.eclipse.lsp4j.Diagnostic;
import org.eclipse.lsp4j.DiagnosticSeverity;
import org.eclipse.lsp4j.Position;
import org.eclipse.lsp4j.Range;

import java.util.ArrayList;
import java.util.List;

/**
 * ANTLR error listener that collects parse errors and converts them to LSP diagnostics.
 */
public class DiagnosticCollector extends BaseErrorListener {
    private final List<Diagnostic> diagnostics = new ArrayList<>();

    @Override
    public void syntaxError(
            Recognizer<?, ?> recognizer,
            Object offendingSymbol,
            int line,
            int charPositionInLine,
            String msg,
            RecognitionException e) {
        
        // Convert ANTLR 1-indexed line to LSP 0-indexed
        Position start = new Position(line - 1, charPositionInLine);
        Position end = new Position(line - 1, charPositionInLine + 1);
        Range range = new Range(start, end);

        Diagnostic diagnostic = new Diagnostic();
        diagnostic.setRange(range);
        diagnostic.setSeverity(DiagnosticSeverity.Error);
        diagnostic.setSource("foolish");
        diagnostic.setMessage(msg);

        diagnostics.add(diagnostic);
    }

    public List<Diagnostic> getDiagnostics() {
        return diagnostics;
    }

    public void clear() {
        diagnostics.clear();
    }
}
