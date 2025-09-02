package org.foolish.reference.interpreter;

import org.foolish.reference.interpreter.ir.*;
import java.util.*;

public class Renderer {
    public String render(RuntimeIntermediateNode.EvaluatedBrane brane) {
        StringBuilder sb = new StringBuilder();
        for (RuntimeNode node : brane.getStatements()) {
            sb.append(renderNode(node)).append("\n");
        }
        return sb.toString();
    }

    private String renderNode(RuntimeNode node) {
        if (node instanceof Evaluator.RuntimeFinalAssignment) {
            Evaluator.RuntimeFinalAssignment assign = (Evaluator.RuntimeFinalAssignment) node;
            return assign.getName() + " = " + renderNode(assign.getValue());
        } else if (node instanceof Evaluator.RuntimeFinalInteger) {
            return Integer.toString(((Evaluator.RuntimeFinalInteger) node).getValue());
        } else if (node instanceof Evaluator.RuntimeFinalBrane) {
            Evaluator.RuntimeFinalBrane brane = (Evaluator.RuntimeFinalBrane) node;
            return "<brane:" + brane.getBrane().getStatements().size() + " statements>";
        } else if (node instanceof Evaluator.RuntimeFinalUnknown) {
            return node.toString();
        }
        return "<unknown>";
    }
}
