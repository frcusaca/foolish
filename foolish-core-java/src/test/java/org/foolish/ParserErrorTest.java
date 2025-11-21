package org.foolish;

import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertThrows;

/**
 * Tests for parser errors, specifically validating that unbalanced
 * parentheses in regexp patterns are rejected.
 */
public class ParserErrorTest {

    private void assertParseError(String code) {
        assertThrows(Exception.class, () -> {
            CharStream input = CharStreams.fromString(code);
            FoolishLexer lexer = new FoolishLexer(input);
            CommonTokenStream tokens = new CommonTokenStream(lexer);
            FoolishParser parser = new FoolishParser(tokens);

            // Remove default error listeners to avoid console spam
            parser.removeErrorListeners();

            // Parse the code - this should fail
            parser.program();

            // Check if there were syntax errors
            if (parser.getNumberOfSyntaxErrors() > 0) {
                throw new RuntimeException("Parse error: unbalanced parentheses");
            }
        }, "Expected parse error for unbalanced parentheses");
    }

    @Test
    void unbalancedOpenParenInRegexpThrowsError() {
        assertParseError("{ result = brane.(pattern; }");
    }

    @Test
    void unbalancedCloseParenInRegexpThrowsError() {
        assertParseError("{ result = brane.pattern); }");
    }

    @Test
    void unbalancedOpenBraceInRegexpThrowsError() {
        assertParseError("{ result = brane.{pattern; }");
    }

    @Test
    void unbalancedCloseBraceInRegexpThrowsError() {
        assertParseError("{ result = brane.pattern}; }");
    }

    @Test
    void unbalancedOpenBracketInRegexpThrowsError() {
        assertParseError("{ result = brane.[pattern; }");
    }

    @Test
    void unbalancedCloseBracketInRegexpThrowsError() {
        assertParseError("{ result = brane.pattern]; }");
    }

    @Test
    void mismatchedParenthesesInRegexpThrowsError() {
        assertParseError("{ result = brane.(pattern]; }");
    }

    @Test
    void mismatchedBracesInRegexpThrowsError() {
        assertParseError("{ result = brane.{pattern); }");
    }

    @Test
    void mismatchedBracketsInRegexpThrowsError() {
        assertParseError("{ result = brane.[pattern}; }");
    }

    @Test
    void multipleUnbalancedParenthesesThrowsError() {
        assertParseError("{ result = brane.((pattern); }");
    }

    @Test
    void nestedUnbalancedParenthesesThrowsError() {
        assertParseError("{ result = brane.{outer[inner(deep}; }");
    }
}
