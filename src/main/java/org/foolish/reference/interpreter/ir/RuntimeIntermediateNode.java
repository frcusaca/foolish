package org.foolish.reference.interpreter.ir;

import org.foolish.ast.AST;

import java.util.List;
import java.util.Map;

/** Intermediate nodes hold partially evaluated state. */
public sealed interface RuntimeIntermediateNode extends RuntimeNode, RuntimeNode.RuntimeExpr permits RuntimeIntermediateNode.Binary, RuntimeIntermediateNode.Unary, RuntimeIntermediateNode.EvaluatedAssignment, RuntimeIntermediateNode.EvaluatedBrane {
    @Override
    default NodeKind kind() { return NodeKind.INTERMEDIATE; }

    record Binary(String op, RuntimeExpr left, RuntimeExpr right) implements RuntimeIntermediateNode { }

    record Unary(String op, RuntimeExpr expr) implements RuntimeIntermediateNode { }

    record EvaluatedAssignment(String id, RuntimeExpr value) implements RuntimeIntermediateNode, RuntimeStatement { }

    /**
     * Holds a sequence of evaluated statements and the final environment snapshot after evaluation.
     */
    record EvaluatedBrane(List<RuntimeNode> statements, Map<AST.Identifier, RuntimeNode> envSnapshot) implements RuntimeIntermediateNode, RuntimeNode.RuntimeBrane { }
}

