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
    public void testExprStatement() {
        AST ast = parse("{ 1; }");
        assertTrue(ast instanceof AST.Branes);
        AST.Branes branes = (AST.Branes) ast;
        assertEquals(1, branes.branes().size());
        AST.Brane brane = branes.branes().get(0);
        assertEquals("""
                {
                  1;
                }
                """, ast.toString());
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
        AST ast = parse("{ x = (1+2)*3+4; y = -(x/-4); z = 1*2^3^4*4^5+1; k=1*2^3^4^5^6}");
        assertTrue(ast instanceof AST.Branes);
        AST.Branes branes = (AST.Branes) ast;
        assertEquals(1, branes.branes().size());
        AST.Brane brane = branes.branes().get(0);
        assertEquals(4, brane.statements().size());
        assertTrue(brane.statements().get(0) instanceof AST.Assignment);
        assertEquals("""
                {
                  x = (((1 + 2) * 3) + 4);
                  y = -(x / -4);
                  z = (((1 * (2 ^ (3 ^ 4))) * (4 ^ 5)) + 1);
                  k = (1 * (2 ^ (3 ^ (4 ^ (5 ^ 6)))));
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


    @Test
    public void testUnknown() {
        AST ast = parse("{ x = ???; y = ??? ;}");
        assertEquals("""
                {
                  x = ???;
                  y = ???;
                }
                """, ast.toString());
    }

    @Test
    public void testComments() {
        AST ast = parse("""
                    { 
                    !!
                        This block of text is known as a block comment.
                        It can contain code such as:
                            {
                                a=2;
                                b=t'3;
                                etc=???;
                            }
                        But it will be ignored by the parser.
                     !!
                        x = ???;
                        y = ???;
                        ! {} this is a comment?
                        ! how about this?
                        y = 10; ! !!now is another comment
                        z = 11;  ! ! staggered commenting ;;;;
                    }
                """);
        assertEquals("""
                {
                  x = ???;
                  y = ???;
                  y = 10;
                  z = 11;
                }
                """, ast.toString());
    }


    @Test
    public void testIf() {
        AST ast = parse("{ x = if a then 1; }");
        assertEquals("""
                {
                  x = if a then 1;
                }
                """, ast.toString());

        AST ast2 = parse("{ x = if a then 1 else ???; }");
        assertEquals("""
                {
                  x = if a then 1;
                }
                """, ast2.toString());

        AST ast3 = parse("{ x = if a then 1 elif b then 3 elif d then 100 else 4; }");
        assertEquals("""
                {
                  x = if a then 1 elif b then 3 elif d then 100 else 4;
                }
                """, ast3.toString());


        AST ast4 = parse("""
                {
                    if a then 
                        if z then 10 else 2 
                    elif b then 
                        if x then
                            30 
                        else 
                            if y then 20 else 3
                    elif d then 
                        if asdf then 
                            if qwer then 300 else 200
                        elif zxcv then 
                            100
                        elif qwe then 
                            if 2 then 50 elif 45 then 40 else 0
                    else 
                        4;
                }""");
        assertEquals("""
                {
                  if a then if z then 10 else 2 elif b then if x then 30 else if y then 20 else 3 elif d then if asdf then if qwer then 300 else 200 elif zxcv then 100 elif qwe then if 2 then 50 elif 45 then 40 else 0 else 4;
                }
                """, ast4.toString());
        AST.Branes ast4Brane = (AST.Branes) ast4;
        assertEquals(1, ast4Brane.branes().size());
        AST.Brane brane = ast4Brane.branes().get(0);
        assertEquals(1, brane.statements().size());
        assertTrue(brane.statements().get(0) instanceof AST.IfExpr);
        AST.IfExpr ifExpr = (AST.IfExpr) brane.statements().get(0);
        // Check the nested structure of the if expression
        assertTrue(ifExpr.thenExpr() instanceof AST.IfExpr);

        // The outer if has two elseifs
        assertEquals(2, ifExpr.elseIfs().size());

        AST.IfExpr elif2 = (AST.IfExpr) ifExpr.elseIfs().get(1);
        assertTrue(elif2.thenExpr() instanceof AST.IfExpr);

        assertEquals(2, ((AST.IfExpr) elif2.thenExpr()).elseIfs().size());
        AST.IfExpr elif21 = ((AST.IfExpr) elif2.thenExpr()).elseIfs().get(0);
        assertEquals(new AST.Identifier("zxcv"), elif21.condition());
        assertEquals(new AST.IntegerLiteral(100), elif21.thenExpr());

    }

    @Test
    public void testUnaryStarOperator() {
        AST ast = parse("{ *1; }");
        assertTrue(ast instanceof AST.Branes);
        AST.Branes branes = (AST.Branes) ast;
        assertEquals(1, branes.branes().size());
        AST.Brane brane = branes.branes().get(0);
        assertEquals(1, brane.statements().size());
        AST.Expr expr = brane.statements().get(0);
        assertTrue(expr instanceof AST.UnaryExpr);
        AST.UnaryExpr unary = (AST.UnaryExpr) expr;
        assertEquals("*", unary.op());
        assertTrue(unary.expr() instanceof AST.IntegerLiteral);
        assertEquals(1L, ((AST.IntegerLiteral) unary.expr()).value());
        assertEquals("*1", unary.toString());
    }

    @Test
    public void testCharacterization() {
        AST ast = parse("{ x = 5; n'42; b = x'{true; false; result=10;}; c= t'1+u'var*z'zz^a'a^b'b+1}");
        assertTrue(ast instanceof AST.Branes);
        AST.Branes branes = (AST.Branes) ast;
        assertEquals(1, branes.branes().size());
        AST.Brane brane = branes.branes().get(0);
        assertEquals(4, brane.statements().size());

        // Check t'x = 5
        assertTrue(brane.statements().get(0) instanceof AST.Assignment);
        AST.Assignment assignment = (AST.Assignment) brane.statements().get(0);
        assertEquals("x", assignment.id());
        assertTrue(assignment.expr() instanceof AST.IntegerLiteral);
        assertEquals(5L, ((AST.IntegerLiteral) assignment.expr()).value());

        // Check n'42
        assertTrue(brane.statements().get(1) instanceof AST.Characterizable);
        AST.Characterizable characterizable = (AST.Characterizable) brane.statements().get(1);
        assertEquals("n", characterizable.characterization().id());
        assertTrue(characterizable instanceof AST.IntegerLiteral);
        assertEquals(42L, ((AST.IntegerLiteral) characterizable).value());

        assertEquals("""
                {
                  x = 5;
                  n'42;
                  b = x'{
                  true;
                  false;
                  result = 10;
                };
                  c = ((t'1 + (u'var * (z'zz ^ (a'a ^ b'b)))) + 1);
                }
                """, ast.toString());
    }


    @Test
    public void testParsingIds(){
        AST ast = parse("""
                {
                   r = a b c d;
                   e f g
                }""");
        assertEquals("""
                {
                  r = a b c d;
                  e f g;
                }
                """, ast.toString());
    }
    @Test
    public void testLibratedBrane(){
        AST ast = parse("""
                {
                    parameter1=1;
                    parameter2 = 2;
                    lb = f'{parameter1;parameter2;}{result = parameter1 + parameter2;}; 
                    result = {1,2} lb;
                }""");
        assertEquals("""
                Math'{
                  pi = 3.14159;
                  e = 2.71828;
                }
                """, ast.toString());
    }
}
