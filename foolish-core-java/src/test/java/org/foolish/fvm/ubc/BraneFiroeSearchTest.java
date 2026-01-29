package org.foolish.fvm.ubc;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class BraneFiroeSearchTest {

    @Test
    public void testSearchFunctionality_Arithmetic() {
        // Setup a simple brane: { a = 1; }
        String source = "{ a = 1; }";
        Object result = UbcRepl.eval(source);
        assertTrue(result instanceof BraneFiroe);
        BraneFiroe brane = (BraneFiroe) result;

        // Perform search for "1+2"
        FIR searchResult = brane.search("1+2");
        assertNotNull(searchResult);
        assertTrue(searchResult.isConstant());

        // 1+2 should be a ValueFiroe wrapping 3.
        assertEquals(3, searchResult.getValue());
    }

    @Test
    public void testSearchFunctionality_BackwardSearch() {
        // Setup a brane: { a = 42; }
        String source = "{ a = 42; }";
        BraneFiroe brane = (BraneFiroe) UbcRepl.eval(source);

        // Search for "?a" (backward search for 'a')
        // We support "?a" syntax by stripping the '?' if it causes a parse error,
        // or by relying on parser recovery, to match user expectations.
        FIR searchResult = brane.search("?a");
        assertNotNull(searchResult);
        assertEquals(42, searchResult.getValue());

        if (searchResult instanceof AbstractSearchFiroe asf) {
            assertTrue(asf.isFound());
        }
    }

    @Test
    public void testSearchFunctionality_NestedAccess() {
        // Setup a nested brane: { inner = { x = 100; }; }
        String source = "{ inner = { x = 100; }; }";
        BraneFiroe brane = (BraneFiroe) UbcRepl.eval(source);

        // Search for "inner.x"
        FIR searchResult = brane.search("inner.x");
        assertNotNull(searchResult);
        assertEquals(100, searchResult.getValue());
    }
}
