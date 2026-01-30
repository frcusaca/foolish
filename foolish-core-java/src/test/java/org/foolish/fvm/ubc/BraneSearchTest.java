package org.foolish.fvm.ubc;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class BraneSearchTest {

    @Test
    public void testBraneSearchFound() {
        // Create a brane with a variable
        Object result = UbcRepl.eval("{ x = 10; }");
        assertTrue(result instanceof BraneFiroe);
        BraneFiroe brane = (BraneFiroe) result;

        // Search for x using backward search ?x
        FIR found = UbcRepl.braneSearch(brane, "?x");

        // Should find the assignment or value
        assertFalse(found instanceof NKFiroe, "Should find x, but got " + found);

        // It should eventually resolve to the value 10
        assertEquals(10, found.getValue());
    }

    @Test
    public void testBraneSearchNotFound() {
        BraneFiroe brane = (BraneFiroe) UbcRepl.eval("{ x = 10; }");
        FIR found = UbcRepl.braneSearch(brane, "?y");
        assertTrue(found instanceof NKFiroe, "Should not find y, but got " + found);
    }

    @Test
    public void testBraneUnmodified() {
        BraneFiroe brane = (BraneFiroe) UbcRepl.eval("{ x = 10; }");
        int initialSize = brane.getExpressionFiroes().size();

        UbcRepl.braneSearch(brane, "?x");

        assertEquals(initialSize, brane.getExpressionFiroes().size(), "Brane size should not change");
    }
}
