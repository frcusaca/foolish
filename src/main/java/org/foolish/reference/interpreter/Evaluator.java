package org.foolish.reference.interpreter;

import org.foolish.ast.AST;
import org.foolish.reference.interpreter.ir.*;
import java.util.*;

public class Evaluator {
    private final Map<String, RuntimeNode> bindings = new HashMap<>();

    public RuntimeIntermediateNode.EvaluatedBrane evaluate(AST.Brane brane) {
        List<RuntimeNode> evaluatedStatements = new ArrayList<>();
        for (AST.Stmt stmt : brane.statements()) {
            RuntimeNode result = evaluateStatement(stmt);
            evaluatedStatements.add(result);
        }
        return new RuntimeIntermediateNode.EvaluatedBrane(evaluatedStatements);
    }

    private RuntimeNode evaluateStatement(AST.Stmt stmt) {
        // Handle assignment, arithmetic, if, unknown, brane, etc.
        if (stmt instanceof AST.Assignment) {
            AST.Assignment assign = (AST.Assignment) stmt;
            RuntimeNode value = evaluateExpr(assign.expr());
            bindings.put(assign.identifier().name(), value);
            return new RuntimeFinalAssignment(assign.identifier().name(), value);
        } else if (stmt instanceof AST.Brane) {
            // Nested brane
            RuntimeIntermediateNode.EvaluatedBrane nested = evaluate((AST.Brane) stmt);
            return new RuntimeFinalBrane(nested);
        } else if (stmt instanceof AST.UnknownExpr) {
            return new RuntimeFinalUnknown();
        }
        // Add more cases as needed
        return new RuntimeFinalUnknown();
    }

    private RuntimeNode evaluateExpr(AST.Expr expr) {
        if (expr instanceof AST.IntegerLiteral) {
            return new RuntimeFinalInteger(((AST.IntegerLiteral) expr).value());
        } else if (expr instanceof AST.Identifier) {
            String name = ((AST.Identifier) expr).name();
            return bindings.getOrDefault(name, new RuntimeFinalUnknown());
        } else if (expr instanceof AST.BinaryExpr) {
            AST.BinaryExpr bin = (AST.BinaryExpr) expr;
            RuntimeNode left = evaluateExpr(bin.left());
            RuntimeNode right = evaluateExpr(bin.right());
            if (left instanceof RuntimeFinalInteger && right instanceof RuntimeFinalInteger) {
                int l = ((RuntimeFinalInteger) left).getValue();
                int r = ((RuntimeFinalInteger) right).getValue();
                switch (bin.op()) {
                    case "+": return new RuntimeFinalInteger(l + r);
                    case "-": return new RuntimeFinalInteger(l - r);
                    case "*": return new RuntimeFinalInteger(l * r);
                    case "/": return new RuntimeFinalInteger(l / r);
                    case "^": return new RuntimeFinalInteger((int)Math.pow(l, r));
                }
            }
            return new RuntimeFinalUnknown();
        } else if (expr instanceof AST.IfExpr) {
            AST.IfExpr ife = (AST.IfExpr) expr;
            RuntimeNode cond = evaluateExpr(ife.cond());
            if (cond instanceof RuntimeFinalInteger && ((RuntimeFinalInteger) cond).getValue() != 0) {
                return evaluateExpr(ife.thenExpr());
            } else {
                return evaluateExpr(ife.elseExpr());
            }
        }
        return new RuntimeFinalUnknown();
    }

    // Final assignment node
    public static class RuntimeFinalAssignment extends RuntimeFinalNode {
        private final String name;
        private final RuntimeNode value;
        public RuntimeFinalAssignment(String name, RuntimeNode value) {
            this.name = name;
            this.value = value;
        }
        public String getName() { return name; }
        public RuntimeNode getValue() { return value; }
    }

    // Final integer node
    public static class RuntimeFinalInteger extends RuntimeFinalNode {
        private final int value;
        public RuntimeFinalInteger(int value) { this.value = value; }
        public int getValue() { return value; }
    }

    // Final brane node
    public static class RuntimeFinalBrane extends RuntimeFinalNode {
        private final RuntimeIntermediateNode.EvaluatedBrane brane;
        public RuntimeFinalBrane(RuntimeIntermediateNode.EvaluatedBrane brane) { this.brane = brane; }
        public RuntimeIntermediateNode.EvaluatedBrane getBrane() { return brane; }
    }

    // Final unknown node
    public static class RuntimeFinalUnknown extends RuntimeFinalNode {
        @Override
        public String toString() { return "???"; }
    }
}
