package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.fvm.Env;
import org.foolish.fvm.v1.Insoe;
import org.foolish.fvm.ubc.BraneFiroe;
import org.foolish.fvm.ubc.Sequencer4Human;
import org.foolish.fvm.ubc.UnicelluarBraneComputer;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for the Sequencer system.
 */
class SequencerTest {

    @Test
    void testSequencer4HumanSimpleInteger() {
        AST.Brane brane = new AST.Brane(List.of(
            new AST.IntegerLiteral(5L)
        ));

        Insoe insoe = new Insoe(brane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);
        ubc.runToCompletion();

        BraneFiroe result = ubc.getRootBrane();
        String output = new Sequencer4Human().sequence(result);

        assertTrue(output.contains("{"));
        assertTrue(output.contains("}"));
        assertTrue(output.contains("5"));
        assertTrue(output.contains("➠")); // Should use the tab character
    }

    @Test
    void testSequencer4HumanMultipleExpressions() {
        AST.Brane brane = new AST.Brane(List.of(
            new AST.IntegerLiteral(1L),
            new AST.IntegerLiteral(2L),
            new AST.IntegerLiteral(3L)
        ));

        Insoe insoe = new Insoe(brane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);
        ubc.runToCompletion();

        BraneFiroe result = ubc.getRootBrane();
        String output = new Sequencer4Human().sequence(result);

        assertTrue(output.contains("1"));
        assertTrue(output.contains("2"));
        assertTrue(output.contains("3"));

        // Count the number of lines
        int lineCount = output.split("\n").length;
        assertEquals(5, lineCount); // { + 3 expressions + }
    }

    @Test
    void testSequencer4HumanCustomTabChar() {
        AST.Brane brane = new AST.Brane(List.of(
            new AST.IntegerLiteral(42L)
        ));

        Insoe insoe = new Insoe(brane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);
        ubc.runToCompletion();

        BraneFiroe result = ubc.getRootBrane();
        Sequencer4Human customSequencer = new Sequencer4Human("  "); // Two spaces
        String output = customSequencer.sequence(result);

        assertTrue(output.contains("{"));
        assertTrue(output.contains("  42")); // Should use two spaces
        assertFalse(output.contains("➠")); // Should NOT use default tab
    }

    @Test
    void testSequencer4HumanBinaryExpression() {
        AST.BinaryExpr expr = new AST.BinaryExpr("+",
            new AST.IntegerLiteral(10L),
            new AST.IntegerLiteral(20L)
        );

        AST.Brane brane = new AST.Brane(List.of(expr));

        Insoe insoe = new Insoe(brane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);
        ubc.runToCompletion();

        BraneFiroe result = ubc.getRootBrane();
        String output = new Sequencer4Human().sequence(result);

        assertTrue(output.contains("30")); // Should show the evaluated result
    }

    @Test
    void testSequencer4HumanNestedBrane() {
        AST.Brane innerBrane = new AST.Brane(List.of(
            new AST.IntegerLiteral(7L)
        ));

        AST.Brane outerBrane = new AST.Brane(List.of(
            new AST.IntegerLiteral(5L),
            innerBrane
        ));

        Insoe insoe = new Insoe(outerBrane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);
        ubc.runToCompletion();

        BraneFiroe result = ubc.getRootBrane();
        String output = new Sequencer4Human().sequence(result);

        // Should have nested braces
        assertTrue(output.contains("{"));
        assertTrue(output.contains("}"));

        // Should show both values
        assertTrue(output.contains("5"));
        assertTrue(output.contains("7"));

        // Should have proper indentation with ➠
        long tabCount = output.chars().filter(ch -> ch == '➠').count();
        assertTrue(tabCount >= 2); // At least 2 tabs for nested structure
    }

    @Test
    void testDefaultTabCharacter() {
        Sequencer4Human sequencer = new Sequencer4Human();
        assertEquals("➠", sequencer.getTabChar());
    }

    @Test
    void testToStringUsesSequencer() {
        AST.Brane brane = new AST.Brane(List.of(
            new AST.IntegerLiteral(99L)
        ));

        Insoe insoe = new Insoe(brane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(insoe);
        ubc.runToCompletion();

        BraneFiroe result = ubc.getRootBrane();
        String toStringOutput = result.toString();

        // toString() should use Sequencer4Human
        assertTrue(toStringOutput.contains("➠"));
        assertTrue(toStringOutput.contains("99"));
    }
}
