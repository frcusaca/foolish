package org.foolish;

import org.antlr.v4.runtime.*;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.foolish.interpreter.BraneValue;
import org.foolish.interpreter.Environment;
import org.foolish.interpreter.IntValue;
import org.foolish.interpreter.Interpreter;
import org.foolish.interpreter.Value;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

public class InterpreterTest {
    private AST.Branes parse(String code) {
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        return (AST.Branes) new ASTBuilder().visit(parser.program());
    }

    @Test
    public void testSimpleAssignmentAndArithmetic() {
        String code = "{ x = 2 + 3 * 4; y = x * 2; }";
        AST.Branes branes = parse(code);
        Interpreter interpreter = new Interpreter();
        interpreter.evaluate(branes);
        Environment env = interpreter.global();
        assertEquals(14L, ((IntValue) env.get("x")).value());
        assertEquals(28L, ((IntValue) env.get("y")).value());
    }

    @Test
    public void testIfExpr() {
        String code = "{ x = if 1 then 42 else 0; y = if 0 then 1 else 2; }";
        AST.Branes branes = parse(code);
        Interpreter interpreter = new Interpreter();
        interpreter.evaluate(branes);
        Environment env = interpreter.global();
        assertEquals(42L, ((IntValue) env.get("x")).value());
        assertEquals(2L, ((IntValue) env.get("y")).value());
    }

    @Test
    public void testUnknownAndNestedBrane() {
        String code = "{ a = ???; b = { x = 1; }; }";
        AST.Branes branes = parse(code);
        Interpreter interpreter = new Interpreter();
        interpreter.evaluate(branes);
        Environment env = interpreter.global();
        assertSame(Value.UNKNOWN, env.get("a"));
        Value b = env.get("b");
        assertTrue(b instanceof BraneValue);
        assertEquals(1, ((BraneValue) b).brane().statements().size());
    }
}
