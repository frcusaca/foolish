package org.foolish.reference.interpreter;

import org.foolish.ast.AST;
import org.foolish.reference.interpreter.ir.RuntimeFinalNode;
import org.foolish.reference.interpreter.ir.RuntimeIntermediateNode;
import org.foolish.reference.interpreter.ir.RuntimeNode;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

/** Quick-and-dirty evaluator over the AST producing runtime nodes. */
public final class Evaluator {

    public RuntimeIntermediateNode.EvaluatedBrane evaluate(AST.Brane brane) {
        return evaluate(brane, new Environment());
    }

    public RuntimeIntermediateNode.EvaluatedBrane evaluate(AST.Brane brane, Environment env) {
        List<RuntimeNode> out = new ArrayList<>();
        for (AST.Expr stmt : brane.statements()) {
            RuntimeNode node = evaluateStmtOrExpr(stmt, env);
            out.add(node);
        }
        Map<AST.Identifier, RuntimeNode> snap = env.snapshot();
        return new RuntimeIntermediateNode.EvaluatedBrane(out, snap);
    }

    private RuntimeNode evaluateStmtOrExpr(AST.Expr stmt, Environment env) {
        if (stmt instanceof AST.Assignment asg) {
            RuntimeNode.RuntimeExpr value = evalExpr(asg.expr(), env);
            // Bind by plain identifier (no characterization on LHS by grammar)
            env.put(new AST.Identifier(asg.id()), value);
            return new RuntimeIntermediateNode.EvaluatedAssignment(asg.id(), value);
        }
        // Expression statement
        return evalExpr(stmt, env);
    }

    public RuntimeNode.RuntimeExpr evalExpr(AST.Expr e, Environment env) {
        if (e instanceof AST.IntegerLiteral lit) {
            return new RuntimeFinalNode.IntValue(lit.value());
        }
        if (e instanceof AST.UnknownExpr) {
            return RuntimeFinalNode.UnknownValue.INSTANCE;
        }
        if (e instanceof AST.Identifier id) {
            RuntimeNode v = env.get(id);
            if (v == null) return RuntimeFinalNode.UnknownValue.INSTANCE;
            if (v instanceof RuntimeNode.RuntimeExpr re) return re;
            // If it's a statement or brane somehow, no meaningful value; mark unknown
            return RuntimeFinalNode.UnknownValue.INSTANCE;
        }
        if (e instanceof AST.UnaryExpr ue) {
            RuntimeNode.RuntimeExpr inner = evalExpr(ue.expr(), env);
            return evalUnary(ue.op(), inner);
        }
        if (e instanceof AST.BinaryExpr be) {
            RuntimeNode.RuntimeExpr l = evalExpr(be.left(), env);
            RuntimeNode.RuntimeExpr r = evalExpr(be.right(), env);
            return evalBinary(be.op(), l, r);
        }
        if (e instanceof AST.Branes brs) {
            // Evaluate each brane; value of an expression branes is unknown; but return last env snapshot as a brane node
            Environment child = env.copy();
            RuntimeIntermediateNode.EvaluatedBrane last = null;
            for (AST.Brane b : brs.branes()) {
                last = evaluate(b, child);
            }
            return last == null ? RuntimeFinalNode.UnknownValue.INSTANCE : last;
        }
        if (e instanceof AST.Brane b) {
            // Embedded brane: evaluate in child env, return the evaluated brane node
            Environment child = env.copy();
            return evaluate(b, child);
        }
        if (e instanceof AST.IfExpr ife) {
            return evalIf(ife, env);
        }
        // Fallback: unknown
        return RuntimeFinalNode.UnknownValue.INSTANCE;
    }

    private RuntimeNode.RuntimeExpr evalIf(AST.IfExpr ife, Environment env) {
        RuntimeNode.RuntimeExpr cond = evalExpr(ife.condition(), env);
        Boolean condVal = toBoolean(cond);
        if (condVal == null) return RuntimeFinalNode.UnknownValue.INSTANCE;
        if (condVal) return evalExpr(ife.thenExpr(), env);
        // else-ifs
        for (AST.IfExpr elif : ife.elseIfs()) {
            RuntimeNode.RuntimeExpr c = evalExpr(elif.condition(), env);
            Boolean v = toBoolean(c);
            if (v == null) return RuntimeFinalNode.UnknownValue.INSTANCE;
            if (v) return evalExpr(elif.thenExpr(), env);
        }
        return evalExpr(ife.elseExpr(), env);
    }

    private Boolean toBoolean(RuntimeNode.RuntimeExpr expr) {
        if (expr instanceof RuntimeFinalNode.IntValue iv) return iv.value() != 0;
        if (expr instanceof RuntimeFinalNode.BoolValue bv) return bv.value();
        return null; // unknown
    }

    private RuntimeNode.RuntimeExpr evalUnary(String op, RuntimeNode.RuntimeExpr inner) {
        if (inner instanceof RuntimeFinalNode.IntValue iv) {
            switch (op) {
                case "+": return new RuntimeFinalNode.IntValue(+iv.value());
                case "-": return new RuntimeFinalNode.IntValue(-iv.value());
                case "*": return new RuntimeFinalNode.IntValue(iv.value()); // unary * is identity here
            }
        }
        return new RuntimeIntermediateNode.Unary(op, inner);
    }

    private RuntimeNode.RuntimeExpr evalBinary(String op, RuntimeNode.RuntimeExpr l, RuntimeNode.RuntimeExpr r) {
        if (l instanceof RuntimeFinalNode.IntValue li && r instanceof RuntimeFinalNode.IntValue ri) {
            return switch (op) {
                case "+" -> new RuntimeFinalNode.IntValue(li.value() + ri.value());
                case "-" -> new RuntimeFinalNode.IntValue(li.value() - ri.value());
                case "*" -> new RuntimeFinalNode.IntValue(li.value() * ri.value());
                case "/" -> ri.value() == 0 ? RuntimeFinalNode.UnknownValue.INSTANCE : new RuntimeFinalNode.IntValue(li.value() / ri.value());
                case "^" -> powInt(li.value(), ri.value());
                default -> new RuntimeIntermediateNode.Binary(op, l, r);
            };
        }
        return new RuntimeIntermediateNode.Binary(op, l, r);
    }

    private RuntimeNode.RuntimeExpr powInt(long base, long exp) {
        if (exp < 0) return RuntimeFinalNode.UnknownValue.INSTANCE;
        long result = 1;
        long b = base;
        long e = exp;
        while (e > 0) {
            if ((e & 1) == 1) result *= b;
            e >>= 1;
            if (e != 0) b *= b;
        }
        return new RuntimeFinalNode.IntValue(result);
    }
}

