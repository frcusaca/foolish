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
        assertTrue(ast instanceof AST.Branes);
        AST.Branes branes = (AST.Branes) ast;
        assertEquals(1, branes.branes().size());
        AST.Brane brane = branes.branes().get(0);
        assertEquals(1, brane.statements().size());
        assertTrue(brane.statements().get(0) instanceof AST.Assignment);
        assertEquals("""
                {
                  x = 5;
                }
                """, ast.toString());

    }

    @Test
    public void testArithmetic() {
        AST ast = parse("{ x = 1+2*3; y = x-4; }");
        assertTrue(ast instanceof AST.Branes);
        AST.Branes branes = (AST.Branes) ast;
        assertEquals(1, branes.branes().size());
        AST.Brane brane = branes.branes().get(0);
        assertEquals(2, brane.statements().size());
        assertTrue(brane.statements().get(0) instanceof AST.Assignment);
        assertEquals("""
                {
                  x = (1 + (2 * 3));
                  y = (x - 4);
                }
                """, ast.toString());

    }


    @Test
    public void testPrecedence() {
        AST ast = parse("{ x = (1+2)*3+4; y = -(x/-4); }");
        assertTrue(ast instanceof AST.Branes);
        AST.Branes branes = (AST.Branes) ast;
        assertEquals(1, branes.branes().size());
        AST.Brane brane = branes.branes().get(0);
        assertEquals(2, brane.statements().size());
        assertTrue(brane.statements().get(0) instanceof AST.Assignment);
        assertEquals("""
                {
                  x = (((1 + 2) * 3) + 4);
                  y = -(x / -4);
                }
                """, ast.toString());

    }


    @Test
    public void testUnary() {
        AST ast = parse("{ x = -3; y = +x; }");
        assertTrue(ast instanceof AST.Branes);
        AST.Branes branes = (AST.Branes) ast;
        assertEquals(1, branes.branes().size());
        AST.Brane brane = branes.branes().get(0);
        assertEquals("""
                {
                  x = -3;
                  y = +x;
                }""", brane.toString());
        // AST has an EOL but brane does not


    }

    @Test
    public void testNestedExpr() {
        AST ast = parse("{ x = -(2 + (3*4)); }");
        assertTrue(ast instanceof AST.Branes);
        AST.Branes branes = (AST.Branes) ast;
        assertEquals(1, branes.branes().size());
        AST.Brane brane = branes.branes().get(0);
        assertEquals("""
                {
                  x = -(2 + (3 * 4));
                }
                """, ast.toString());
    }


    @Test
    public void testBraneConcatenation() {
        // In Foolish, consecutive branes with out semicolo separator means to concatenate them
        // in the sequence they're listed.
        AST ast = parse("{ x = 1; y=2 ;} { z=3; } { x = x + y + z; result=42;}");
        assertTrue(ast instanceof AST.Branes);
        assertEquals(3, ((AST.Branes) ast).branes().size());
        assertEquals("""
                {
                  x = 1;
                  y = 2;
                }
                {
                  z = 3;
                }
                {
                  x = ((x + y) + z);
                  result = 42;
                }
                """, ast.toString());
    }
}
