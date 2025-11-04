package org.foolish.fvm.ubc;

import org.foolish.fvm.Env;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for the UBC REPL.
 */
class UbcReplTest {

    @Test
    void testParseSimpleInteger() {
        String source = "{1;}";
        var ast = UbcRepl.parse(source);
        assertNotNull(ast);
    }

    @Test
    void testEvalSimpleInteger() {
        String source = "{5;}";
        Env env = new Env();
        Object result = UbcRepl.eval(source, env);
        assertNotNull(result);
        assertTrue(result instanceof BraneFiroe);
        BraneFiroe brane = (BraneFiroe) result;
        assertEquals(1, brane.getExpressionFiroes().size());
        assertEquals(5L, brane.getExpressionFiroes().get(0).getValue());
    }

    @Test
    void testEvalBinaryExpression() {
        String source = "{10 + 20;}";
        Env env = new Env();
        Object result = UbcRepl.eval(source, env);
        assertNotNull(result);
        assertTrue(result instanceof BraneFiroe);
        BraneFiroe brane = (BraneFiroe) result;
        assertEquals(1, brane.getExpressionFiroes().size());
        assertEquals(30L, brane.getExpressionFiroes().get(0).getValue());
    }

    @Test
    void testEvalMultipleExpressions() {
        String source = "{1; 2; 3 + 4;}";
        Env env = new Env();
        Object result = UbcRepl.eval(source, env);
        assertNotNull(result);
        assertTrue(result instanceof BraneFiroe);
        BraneFiroe brane = (BraneFiroe) result;
        assertEquals(3, brane.getExpressionFiroes().size());
        assertEquals(1L, brane.getExpressionFiroes().get(0).getValue());
        assertEquals(2L, brane.getExpressionFiroes().get(1).getValue());
        assertEquals(7L, brane.getExpressionFiroes().get(2).getValue());
    }

    @Test
    void testEvalNestedExpression() {
        String source = "{(5 + 3) * 2;}";
        Env env = new Env();
        Object result = UbcRepl.eval(source, env);
        assertNotNull(result);
        assertTrue(result instanceof BraneFiroe);
        BraneFiroe brane = (BraneFiroe) result;
        assertEquals(1, brane.getExpressionFiroes().size());
        assertEquals(16L, brane.getExpressionFiroes().get(0).getValue());
    }

    @Test
    void testEvalUnaryExpression() {
        String source = "{-42;}";
        Env env = new Env();
        Object result = UbcRepl.eval(source, env);
        assertNotNull(result);
        assertTrue(result instanceof BraneFiroe);
        BraneFiroe brane = (BraneFiroe) result;
        assertEquals(1, brane.getExpressionFiroes().size());
        assertEquals(-42L, brane.getExpressionFiroes().get(0).getValue());
    }

    @Test
    void testEvalDivision() {
        String source = "{100 / 5;}";
        Env env = new Env();
        Object result = UbcRepl.eval(source, env);
        assertNotNull(result);
        assertTrue(result instanceof BraneFiroe);
        BraneFiroe brane = (BraneFiroe) result;
        assertEquals(1, brane.getExpressionFiroes().size());
        assertEquals(20L, brane.getExpressionFiroes().get(0).getValue());
    }

    @Test
    void testEvalModulo() {
        String source = "{17 / 5;}";
        Env env = new Env();
        Object result = UbcRepl.eval(source, env);
        assertNotNull(result);
        assertTrue(result instanceof BraneFiroe);
        BraneFiroe brane = (BraneFiroe) result;
        assertEquals(1, brane.getExpressionFiroes().size());
        assertEquals(3L, brane.getExpressionFiroes().get(0).getValue());
    }

    @Test
    void testEvalComplexExpression() {
        String source = "{(10 + 5) * 2 - 8;}";
        Env env = new Env();
        Object result = UbcRepl.eval(source, env);
        assertNotNull(result);
        assertTrue(result instanceof BraneFiroe);
        BraneFiroe brane = (BraneFiroe) result;
        assertEquals(1, brane.getExpressionFiroes().size());
        assertEquals(22L, brane.getExpressionFiroes().get(0).getValue());
    }

    @Test
    void testEvalMultiplication() {
        String source = "{7 * 8;}";
        Env env = new Env();
        Object result = UbcRepl.eval(source, env);
        assertNotNull(result);
        assertTrue(result instanceof BraneFiroe);
        BraneFiroe brane = (BraneFiroe) result;
        assertEquals(1, brane.getExpressionFiroes().size());
        assertEquals(56L, brane.getExpressionFiroes().get(0).getValue());
    }
}
