package org.foolish;

import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.tree.ParseTree;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

public class ParserTest {

    private AST parse(String code) {
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        ParseTree tree = parser.program();
        return new ASTBuilder().visit(tree);
    }

    @Test
    public void testSimpleAssignment() {
        AST ast = parse("{ x = 5; }");
        assertTrue(ast instanceof AST.Brane);
        AST.Brane brane = (AST.Brane) ast;
        assertEquals(1, brane.statements().size());
        assertTrue(brane.statements().get(0) instanceof AST.Assignment);
        assertEquals("""
{
  x = 5;
}""", ast.toString());

    }

    @Test
    public void testArithmetic() {
        AST ast = parse("{ x = 1+2*3; y = x-4; }");
        assertTrue(ast instanceof AST.Brane);
        AST.Brane brane = (AST.Brane) ast;
        assertEquals(2, brane.statements().size());
        assertTrue(brane.statements().get(0) instanceof AST.Assignment);
        assertEquals("""
{
  x = (1 + (2 * 3));
  y = (x - 4);
}""", ast.toString());

    }

    @Test
    public void testUnary() {
        AST ast = parse("{ x = -3; y = +x; }");
        assertTrue(ast instanceof AST.Brane);
        assertEquals("""
{
  x = -3;
  y = +x;
}""", ast.toString());
    }

    @Test
    public void testNestedExpr() {
        AST ast = parse("{ x = -(2 + (3*4)); }");
        assertTrue(ast instanceof AST.Brane);
        assertEquals("""
{
  x = -(2 + (3 * 4));
}""", ast.toString());
    }
}
