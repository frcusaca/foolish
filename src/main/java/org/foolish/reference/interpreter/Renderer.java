package org.foolish.reference.interpreter;

import org.foolish.ast.AST;
import org.foolish.reference.interpreter.ir.RuntimeFinalNode;
import org.foolish.reference.interpreter.ir.RuntimeIntermediateNode;
import org.foolish.reference.interpreter.ir.RuntimeNode;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;

/** Renders evaluated branes and runtime nodes into a human-readable format. */
public final class Renderer {

    public String render(RuntimeIntermediateNode.EvaluatedBrane brane) {
        StringBuilder sb = new StringBuilder();
        sb.append("{\n");
        // Optionally show normalized statements results
        for (RuntimeNode stmt : brane.statements()) {
            String line = renderStmt(stmt);
            if (line != null && !line.isEmpty()) {
                sb.append("  ").append(line).append(";\n");
            }
        }
        // Also show final environment snapshot
        List<Map.Entry<AST.Identifier, RuntimeNode>> entries = new ArrayList<>(brane.envSnapshot().entrySet());
        entries.sort(Comparator.comparing(e -> e.getKey().cannonicalId()));
        for (Map.Entry<AST.Identifier, RuntimeNode> e : entries) {
            String id = e.getKey().toString();
            String v = renderValue(e.getValue());
            sb.append("  ").append(id).append(" = ").append(v).append(";\n");
        }
        sb.append("}");
        return sb.toString();
    }

    private String renderStmt(RuntimeNode node) {
        if (node instanceof RuntimeIntermediateNode.EvaluatedAssignment asg) {
            return asg.id() + " = " + renderValue(asg.value());
        }
        if (node instanceof RuntimeNode.RuntimeExpr expr) {
            return renderValue(expr);
        }
        return null;
    }

    private String renderValue(RuntimeNode node) {
        if (node instanceof RuntimeFinalNode.IntValue iv) return Long.toString(iv.value());
        if (node instanceof RuntimeFinalNode.BoolValue bv) return Boolean.toString(bv.value());
        if (node instanceof RuntimeFinalNode.UnknownValue) return "???";
        if (node instanceof RuntimeIntermediateNode.Binary b) {
            return "(" + renderValue(b.left()) + " " + b.op() + " " + renderValue(b.right()) + ")";
        }
        if (node instanceof RuntimeIntermediateNode.Unary u) {
            return u.op() + renderValue(u.expr());
        }
        if (node instanceof RuntimeIntermediateNode.EvaluatedBrane eb) {
            return "<brane:" + eb.statements().size() + " statements>";
        }
        // Source nodes or unknown kinds
        return "<node>";
    }
}

