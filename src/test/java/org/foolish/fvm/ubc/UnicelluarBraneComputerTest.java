package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.fvm.v1.Insoe;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for the Unicellular Brane Computer.
 */
class UnicelluarBraneComputerTest {

    @Test
    void testSimpleIntegerBrane() {
        // Create a simple brane with integer literals: {1; 2; 3;}
        AST.Brane brane = new AST.Brane(List.of(
            new AST.IntegerLiteral(1L),
            new AST.IntegerLiteral(2L),
            new AST.IntegerLiteral(3L)
        ));

        Insoe insoe = new Insoe(brane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);

        // Run to completion
        int steps = ubc.runToCompletion();

        // Should complete successfully
        assertTrue(ubc.isComplete());
        assertTrue(steps >= 0);
    }

    @Test
    void testBinaryExpression() {
        // Create a brane with a binary expression: {1 + 2;}
        AST.BinaryExpr expr = new AST.BinaryExpr("+",
            new AST.IntegerLiteral(1L),
            new AST.IntegerLiteral(2L)
        );

        AST.Brane brane = new AST.Brane(List.of(expr));

        Insoe insoe = new Insoe(brane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);

        // Run to completion
        ubc.runToCompletion();

        // Should complete successfully
        assertTrue(ubc.isComplete());

        // Get the result from the root brane
        BraneFiroe rootBrane = ubc.getRootBrane();
        List<FIR> expressions = rootBrane.getExpressionFiroes();

        assertEquals(1, expressions.size());
        FIR resultFiroe = expressions.get(0);
        assertTrue(resultFiroe instanceof BinaryFiroe);

        BinaryFiroe binaryFiroe = (BinaryFiroe) resultFiroe;
        assertEquals(3L, binaryFiroe.getValue());
    }

    @Test
    void testUnaryExpression() {
        // Create a brane with a unary expression: {-5;}
        AST.UnaryExpr expr = new AST.UnaryExpr("-",
            new AST.IntegerLiteral(5L)
        );

        AST.Brane brane = new AST.Brane(List.of(expr));

        Insoe insoe = new Insoe(brane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);

        // Run to completion
        ubc.runToCompletion();

        // Should complete successfully
        assertTrue(ubc.isComplete());

        // Get the result
        BraneFiroe rootBrane = ubc.getRootBrane();
        List<FIR> expressions = rootBrane.getExpressionFiroes();

        assertEquals(1, expressions.size());
        FIR resultFiroe = expressions.get(0);
        assertTrue(resultFiroe instanceof UnaryFiroe);

        UnaryFiroe unaryFiroe = (UnaryFiroe) resultFiroe;
        assertEquals(-5L, unaryFiroe.getValue());
    }

    @Test
    void testNestedBinaryExpression() {
        // Create a brane with nested binary expression: {(1 + 2) * 3;}
        AST.BinaryExpr innerExpr = new AST.BinaryExpr("+",
            new AST.IntegerLiteral(1L),
            new AST.IntegerLiteral(2L)
        );

        AST.BinaryExpr outerExpr = new AST.BinaryExpr("*",
            innerExpr,
            new AST.IntegerLiteral(3L)
        );

        AST.Brane brane = new AST.Brane(List.of(outerExpr));

        Insoe insoe = new Insoe(brane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);

        // Run to completion
        ubc.runToCompletion();

        // Should complete successfully
        assertTrue(ubc.isComplete());

        // Get the result
        BraneFiroe rootBrane = ubc.getRootBrane();
        List<FIR> expressions = rootBrane.getExpressionFiroes();

        assertEquals(1, expressions.size());
        FIR resultFiroe = expressions.get(0);
        assertTrue(resultFiroe instanceof BinaryFiroe);

        BinaryFiroe binaryFiroe = (BinaryFiroe) resultFiroe;
        assertEquals(9L, binaryFiroe.getValue());
    }

    @Test
    void testStepByStep() {
        // Create a simple brane with a binary expression
        AST.BinaryExpr expr = new AST.BinaryExpr("+",
            new AST.IntegerLiteral(10L),
            new AST.IntegerLiteral(20L)
        );

        AST.Brane brane = new AST.Brane(List.of(expr));

        Insoe insoe = new Insoe(brane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);

        // Initially not complete
        assertFalse(ubc.isComplete());

        // Step until complete
        int stepCount = 0;
        while (ubc.step()) {
            stepCount++;
            if (stepCount > 100) {
                fail("Too many steps, possible infinite loop");
            }
        }

        // Should be complete now
        assertTrue(ubc.isComplete());
        assertTrue(stepCount > 0);
    }
}
