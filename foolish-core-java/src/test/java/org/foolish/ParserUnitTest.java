package org.foolish;

import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.tree.ParseTree;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

public class ParserUnitTest {

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
        var ast = parse("{ 1; }");
        assertTrue(ast instanceof AST.Program program && program.branes() instanceof AST.Branes branes);
        var branes = ((AST.Program) ast).branes();
        assertEquals(1, branes.branes().size());
        var brane = (AST.Brane) branes.branes().get(0);
        assertEquals("""
                {
                  1;
                }
                """, ast.toString());
    }

    @Test
    public void testSimpleAssignment() {
        var ast = parse("{ x = 5; }");
        assertTrue(ast instanceof AST.Program);
        var branes = ((AST.Program) ast).branes();
        assertEquals(1, branes.branes().size());
        var brane = (AST.Brane) branes.branes().get(0);
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
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(1, branes.branes().size());
        AST.Brane brane = (AST.Brane) branes.branes().get(0);
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
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(1, branes.branes().size());
        AST.Brane brane = (AST.Brane) branes.branes().get(0);
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
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(1, branes.branes().size());
        AST.Brane brane = (AST.Brane) branes.branes().get(0);
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
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(1, branes.branes().size());
        AST.Brane brane = (AST.Brane) branes.branes().get(0);
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
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(3, branes.size());
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
    public void testDetachmentBrane() {
        AST ast = parse("""
                [
                    x = ???;
                    y;
                ]
                {
                    result = x;
                }
                """);
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(2, branes.size());
        assertTrue(branes.branes().get(0) instanceof AST.DetachmentBrane);
        AST.DetachmentBrane detachment = (AST.DetachmentBrane) branes.branes().get(0);
        assertEquals(2, detachment.statements().size());
        AST.DetachmentStatement firstAssignment = detachment.statements().get(0);
        assertEquals("x", firstAssignment.identifier().id());
        assertTrue(firstAssignment.expr() instanceof AST.UnknownExpr);
        AST.DetachmentStatement secondAssignment = detachment.statements().get(1);
        assertEquals("y", secondAssignment.identifier().id());
        assertTrue(secondAssignment.expr() instanceof AST.UnknownExpr);
        assertEquals("""
                [
                  x = ???;
                  y = ???;
                ]
                {
                  result = x;
                }
                """, ast.toString());
    }

    @Test
    public void testDetachmentBraneWithDefaults() {
        AST ast = parse("""
                [
                    r = ???;
                    pi = 3;
                    circumference;
                ]
                """);
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(1, branes.size());
        assertTrue(branes.branes().get(0) instanceof AST.DetachmentBrane);
        AST.DetachmentBrane detachment = (AST.DetachmentBrane) branes.branes().get(0);
        assertEquals(3, detachment.statements().size());
        AST.DetachmentStatement radius = detachment.statements().get(0);
        assertEquals("r", radius.identifier().id());
        assertTrue(radius.expr() instanceof AST.UnknownExpr);
        AST.DetachmentStatement pi = detachment.statements().get(1);
        assertEquals("pi", pi.identifier().id());
        assertTrue(pi.expr() instanceof AST.IntegerLiteral);
        AST.DetachmentStatement circumference = detachment.statements().get(2);
        assertEquals("circumference", circumference.identifier().id());
        assertTrue(circumference.expr() instanceof AST.UnknownExpr);
        assertEquals("""
                [
                  r = ???;
                  pi = 3;
                  circumference = ???;
                ]
                """, ast.toString());
    }

    @Test
    public void testDetachmentBraneCharacterizedIdentifiers() {
        AST ast = parse("""
                [
                    det'x = 1;
                    det'y;
                ]
                """);
        AST.DetachmentBrane detachment = (AST.DetachmentBrane) ((AST.Program) ast).branes().branes().get(0);
        AST.DetachmentStatement first = detachment.statements().get(0);
        assertNotNull(first.identifier().characterizations());
        assertEquals(List.of("det"), first.identifier().characterizations());
        assertEquals("x", first.identifier().id());
        AST.DetachmentStatement second = detachment.statements().get(1);
        assertNotNull(second.identifier().characterizations());
        assertEquals(List.of("det"), second.identifier().characterizations());
        assertEquals("y", second.identifier().id());
        assertTrue(second.expr() instanceof AST.UnknownExpr);
        assertEquals("""
                [
                  det'x = 1;
                  det'y = ???;
                ]
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
                    !
                        This block of text is known as a block comment.
                        It can contain code such as:
                            {
                                a=2;
                                b=t'3;
                                etc=???;
                            }
                        But it will be ignored by the parser.
                     !
                        x = ???;
                        y = ???;
                        !! {} this is a comment?
                        !! how about this?
                        y = 10; !! !!now is another comment
                        z = 11;  !! ! staggered commenting ;;;;
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
        AST.Branes ast4Brane = ((AST.Program) ast4).branes();
        assertEquals(1, ast4Brane.branes().size());
        AST.Brane brane = (AST.Brane) ast4Brane.branes().get(0);
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
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(1, branes.branes().size());
        AST.Brane brane = (AST.Brane) branes.branes().get(0);
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
        AST ast = parse("{ x = 5; n'42; b = x'{true; false; result=10;};}");
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(1, branes.branes().size());
        AST.Brane brane = (AST.Brane) branes.branes().get(0);
        assertEquals(3, brane.statements().size());

        // Check t'x = 5
        assertTrue(brane.statements().get(0) instanceof AST.Assignment);
        AST.Assignment assignment = (AST.Assignment) brane.statements().get(0);
        assertEquals("x", assignment.id());
        assertTrue(assignment.expr() instanceof AST.IntegerLiteral);
        assertEquals(5L, ((AST.IntegerLiteral) assignment.expr()).value());

        // Check n'42
        assertTrue(brane.statements().get(1) instanceof AST.Characterizable);
        AST.Characterizable characterizable = (AST.Characterizable) brane.statements().get(1);
        assertEquals(List.of("n"), characterizable.characterizations());
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
                }
                """, ast.toString());
    }

    @Test
    public void testAllOperatorPrecedences() {
        AST ast = parse("{ x = -1 + +2 * 3 / *4 - +5; }");
        assertEquals("""
                {
                  x = ((-1 + ((+2 * 3) / *4)) - +5);
                }
                """, ast.toString());
    }

    @Test
    public void testDeepBraneNesting() {
        AST ast = parse(
                """
                        {
                         a = 2;
                         b = { a = 3; };
                         c = {
                                 a = 4;
                                    b = { a = 5; b = { a = 6; }; };
                             };
                         { uhoh = ??? ; };
                        }
                        {
                            {
                                { z = 3; };
                                y = 2;
                                { w = 4; };
                            };
                            x = 1;
                            {
                                p = 5;
                                { q = 6;{};{{{};};};};
                            };
                         }
                """
         );
        assertEquals("""
{
  a = 2;
  b = {
  a = 3;
};
  c = {
  a = 4;
  b = {
  a = 5;
  b = {
  a = 6;
};
};
};
  {
  uhoh = ???;
};
}
{
  {
  {
  z = 3;
};
  y = 2;
  {
  w = 4;
};
};
  x = 1;
  {
  p = 5;
  {
  q = 6;
  {
};
  {
  {
  {
};
};
};
};
};
}
""", ast.toString());
    }

    @Test
    public void testSimpleSearchUp() {
        AST ast = parse("↑");
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(1, branes.branes().size());
        assertTrue(branes.branes().get(0) instanceof AST.SearchUP);
        AST.SearchUP searchUp = (AST.SearchUP) branes.branes().get(0);
        assertTrue(searchUp.characterizations().isEmpty());
        assertEquals("↑\n", ast.toString());
    }

    @Test
    public void testMultipleSearchUps() {
        AST ast = parse("↑ ↑");
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(2, branes.branes().size());

        assertTrue(branes.branes().get(0) instanceof AST.SearchUP);
        assertTrue(((AST.SearchUP) branes.branes().get(0)).characterizations().isEmpty());

        assertTrue(branes.branes().get(1) instanceof AST.SearchUP);
        assertTrue(((AST.SearchUP) branes.branes().get(1)).characterizations().isEmpty());

        assertEquals("↑\n↑\n", ast.toString());
    }

    @Test
    public void testSearchUpMixedWithBranes() {
        AST ast = parse("{ x = 1; } ↑ { y = 2; }");
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(3, branes.branes().size());

        assertTrue(branes.branes().get(0) instanceof AST.Brane);
        AST.Brane brane1 = (AST.Brane) branes.branes().get(0);
        assertEquals(1, brane1.statements().size());

        assertTrue(branes.branes().get(1) instanceof AST.SearchUP);

        assertTrue(branes.branes().get(2) instanceof AST.Brane);
        AST.Brane brane2 = (AST.Brane) branes.branes().get(2);
        assertEquals(1, brane2.statements().size());

        assertEquals("""
                {
                  x = 1;
                }
                ↑
                {
                  y = 2;
                }
                """, ast.toString());
    }

    @Test
    public void testSearchUpEquality() {
        AST.SearchUP searchUp1 = new AST.SearchUP();
        AST.SearchUP searchUp2 = new AST.SearchUP();
        AST.SearchUP searchUp3 = new AST.SearchUP(List.of("n"));
        AST.SearchUP searchUp4 = new AST.SearchUP(List.of("n"));
        AST.SearchUP searchUp5 = new AST.SearchUP(List.of("m"));

        assertEquals(searchUp1, searchUp2);
        assertEquals(searchUp3, searchUp4);
        assertNotEquals(searchUp1, searchUp3);
        assertNotEquals(searchUp3, searchUp5);
    }

    @Test
    public void testSearchUpToString() {
        AST.SearchUP searchUp1 = new AST.SearchUP();
        assertEquals("↑", searchUp1.toString());

        AST.SearchUP searchUp2 = new AST.SearchUP(List.of("type"));
        assertEquals("type'↑", searchUp2.toString());

        AST.SearchUP searchUp3 = new AST.SearchUP(List.of());
        assertEquals("↑", searchUp3.toString());
    }

    @Test
    public void testSetCharacterizationOnSearchUp() {
        AST.SearchUP searchUp = new AST.SearchUP();
        assertTrue(searchUp.characterizations().isEmpty());

        AST.SearchUP characterized = AST.setCharacterization(List.of("type"), searchUp);
        assertNotNull(characterized.characterizations());
        assertEquals(List.of("type"), characterized.characterizations());
        assertEquals("type'↑", characterized.toString());

        // Test with empty list characterization
        AST.SearchUP emptyChar = AST.setCharacterization(List.of(), searchUp);
        assertNotNull(emptyChar.characterizations());
        assertTrue(emptyChar.characterizations().isEmpty());
        assertEquals("↑", emptyChar.toString());

        // Test with null characterization
        AST.SearchUP nullChar = AST.setCharacterization(null, searchUp);
        // null should be handled as empty list
        assertEquals("↑", nullChar.toString());
    }

    @Test
    public void testSearchUpWithEmptyCharacterization() {
        AST.SearchUP searchUp = new AST.SearchUP(List.of());
        assertEquals("", searchUp.canonicalCharacterization());
        assertEquals("↑", searchUp.toString());
    }

    @Test
    public void testComplexSearchUpScenario() {
        // Test a complex scenario with SearchUp in a brane with assignments
        AST ast = parse("{ x = 5; } ↑ { y = 10; }");
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(3, branes.branes().size());

        // First brane
        assertTrue(branes.branes().get(0) instanceof AST.Brane);

        // SearchUp (uncharacterized)
        assertTrue(branes.branes().get(1) instanceof AST.SearchUP);
        assertTrue(((AST.SearchUP) branes.branes().get(1)).characterizations().isEmpty());

        // Last brane
        assertTrue(branes.branes().get(2) instanceof AST.Brane);
    }

    @Test
    public void testProgrammaticCharacterization() {
        // Test characterization set programmatically (not parsed)
        AST.SearchUP searchUp = new AST.SearchUP();
        AST.SearchUP characterized = AST.setCharacterization(List.of("type"), searchUp);

        assertNotNull(characterized.characterizations());
        assertEquals(List.of("type"), characterized.characterizations());
        assertEquals("type'↑", characterized.toString());
    }
}
