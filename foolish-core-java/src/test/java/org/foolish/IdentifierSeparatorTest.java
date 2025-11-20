package org.foolish;

import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for identifier separator validation.
 * Verifies that ONLY the allowed separators (U+202F and underscore) work in
 * identifiers.
 */
public class IdentifierSeparatorTest {

    private AST parse(String code) {
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);

        // Don't throw on errors - we want to test both valid and invalid cases
        parser.removeErrorListeners();

        return new ASTBuilder().visit(parser.program());
    }

    private boolean parsesSuccessfully(String code) {
        try {
            CharStream input = CharStreams.fromString(code);
            FoolishLexer lexer = new FoolishLexer(input);
            CommonTokenStream tokens = new CommonTokenStream(lexer);
            FoolishParser parser = new FoolishParser(tokens);

            // Count errors
            final int[] errorCount = { 0 };
            parser.removeErrorListeners();
            parser.addErrorListener(new org.antlr.v4.runtime.BaseErrorListener() {
                @Override
                public void syntaxError(org.antlr.v4.runtime.Recognizer<?, ?> recognizer,
                        Object offendingSymbol,
                        int line,
                        int charPositionInLine,
                        String msg,
                        org.antlr.v4.runtime.RecognitionException e) {
                    errorCount[0]++;
                }
            });

            parser.program();
            return errorCount[0] == 0;
        } catch (Exception e) {
            return false;
        }
    }

    // Tests for ALLOWED separators

    @Test
    void underscoreInIdentifierIsAllowed() {
        String code = "{ my_variable = 10; }";
        assertTrue(parsesSuccessfully(code), "Underscore should be allowed in identifiers");

        AST ast = parse(code);
        assertTrue(ast instanceof AST.Program);
    }

    @Test
    void multipleUnderscoresInIdentifierIsAllowed() {
        String code = "{ my_long_variable_name = 10; }";
        assertTrue(parsesSuccessfully(code), "Multiple underscores should be allowed");
    }

    @Test
    void narrowNonBreakingSpaceInIdentifierIsAllowed() {
        // U+202F is narrow non-breaking space
        String code = "{ my\u202Fvariable = 10; }";
        assertTrue(parsesSuccessfully(code), "Narrow non-breaking space (U+202F) should be allowed in identifiers");

        AST ast = parse(code);
        assertTrue(ast instanceof AST.Program);
    }

    @Test
    void multipleNarrowNonBreakingSpacesInIdentifierIsAllowed() {
        String code = "{ my\u202Flong\u202Fvariable\u202Fname = 10; }";
        assertTrue(parsesSuccessfully(code), "Multiple narrow non-breaking spaces should be allowed");
    }

    @Test
    void mixedUnderscoreAndNarrowSpaceIsAllowed() {
        String code = "{ my_variable\u202Fname = 10; }";
        assertTrue(parsesSuccessfully(code), "Mix of underscore and narrow non-breaking space should be allowed");
    }

    // Tests for DISALLOWED separators

    @Test
    void regularSpaceInIdentifierIsNotAllowed() {
        // Regular space (U+0020) should NOT be allowed
        String code = "{ my variable = 10; }";
        assertFalse(parsesSuccessfully(code), "Regular space (U+0020) should NOT be allowed in identifiers");
    }

    @Test
    void tabInIdentifierIsNotAllowed() {
        String code = "{ my\tvariable = 10; }";
        assertFalse(parsesSuccessfully(code), "Tab should NOT be allowed in identifiers");
    }

    // Edge cases

    @Test
    void identifierCanStartWithLetter() {
        String code = "{ a = 10; }";
        assertTrue(parsesSuccessfully(code), "Identifier can start with letter");
    }

    @Test
    void identifierCanStartWithUnderscore() {
        // Actually, identifiers CAN start with underscore in Foolish
        String code = "{ _variable = 10; }";
        assertTrue(parsesSuccessfully(code), "Identifier can start with underscore");
    }

    @Test
    void identifierCanStartWithNarrowSpace() {
        // Narrow space at the start gets skipped as whitespace, so this becomes just
        // "variable"
        String code = "{ \u202Fvariable = 10; }";
        assertTrue(parsesSuccessfully(code), "Leading narrow space is treated as whitespace");
    }

    @Test
    void identifierCanContainDigitsAfterLetter() {
        String code = "{ var123 = 10; }";
        assertTrue(parsesSuccessfully(code), "Identifier can contain digits after letter");
    }

    @Test
    void identifierCanContainUnderscoreAndDigits() {
        String code = "{ var_123 = 10; }";
        assertTrue(parsesSuccessfully(code), "Identifier can contain underscore and digits");
    }

    @Test
    void identifierCanContainNarrowSpaceAndDigits() {
        String code = "{ var\u202F123 = 10; }";
        assertTrue(parsesSuccessfully(code), "Identifier can contain narrow space and digits");
    }
}
