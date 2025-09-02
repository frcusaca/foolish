package org.foolish;

import org.foolish.reference.interpreter.RuntimeFactory;
import org.foolish.reference.interpreter.Evaluator;
import org.foolish.reference.interpreter.Renderer;
import org.foolish.reference.interpreter.ir.RuntimeSourceCodeNode;
import org.foolish.reference.interpreter.ir.RuntimeIntermediateNode;
import org.foolish.ast.AST;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

public class InterpreterTest {
    @Test
    public void testSimpleAssignmentAndArithmetic() {
        String code = "{ x = 2 + 3 * 4; y = x * 2; }";
        RuntimeSourceCodeNode.SourceBranes src = RuntimeFactory.fromStringToSource(code);
        AST.Branes branes = src.ast();
        assertEquals(1, branes.branes().size());
        AST.Brane brane = branes.branes().get(0);
        Evaluator evaluator = new Evaluator();
        RuntimeIntermediateNode.EvaluatedBrane result = evaluator.evaluate(brane);
        Renderer renderer = new Renderer();
        String output = renderer.render(result);
        assertTrue(output.contains("x = 14"));
        assertTrue(output.contains("y = 28"));
    }

    @Test
    public void testIfExpr() {
        String code = "{ x = if 1 then 42 else 0; y = if 0 then 1 else 2; }";
        RuntimeSourceCodeNode.SourceBranes src = RuntimeFactory.fromStringToSource(code);
        AST.Branes branes = src.ast();
        AST.Brane brane = branes.branes().get(0);
        Evaluator evaluator = new Evaluator();
        RuntimeIntermediateNode.EvaluatedBrane result = evaluator.evaluate(brane);
        Renderer renderer = new Renderer();
        String output = renderer.render(result);
        assertTrue(output.contains("x = 42"));
        assertTrue(output.contains("y = 2"));
    }

    @Test
    public void testUnknownAndNestedBrane() {
        String code = "{ a = ???; b = { x = 1; }; }";
        RuntimeSourceCodeNode.SourceBranes src = RuntimeFactory.fromStringToSource(code);
        AST.Branes branes = src.ast();
        AST.Brane brane = branes.branes().get(0);
        Evaluator evaluator = new Evaluator();
        RuntimeIntermediateNode.EvaluatedBrane result = evaluator.evaluate(brane);
        Renderer renderer = new Renderer();
        String output = renderer.render(result);
        assertTrue(output.contains("a = ???"));
        assertTrue(output.contains("b = <brane:1 statements>"));
    }
}

