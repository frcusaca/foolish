package org.foolish.interpreter;

import org.foolish.ast.AST;

/** Simple interpreter for the Foolish language. */
public class Interpreter {
    private final Environment global = new Environment();

    public Environment global() {
        return global;
    }

    /** Evaluate any AST node. */
    public Value evaluate(AST ast) {
        if (ast instanceof AST.Program p) {
            return evalBranes(p.brane());
        } else if (ast instanceof AST.Branes branes) {
            return evalBranes(branes);
        } else if (ast instanceof AST.Expr expr) {
            return evalExpr(expr, global);
        }
        return Value.UNKNOWN;
    }

    private Value evalBranes(AST.Branes branes) {
        Value result = Value.UNKNOWN;
        for (AST.Brane brane : branes.branes()) {
            result = execBrane(brane, global);
        }
        return result;
    }

    /** Execute the statements of a brane in the given environment. */
    private Value execBrane(AST.Brane brane, Environment env) {
        Value result = Value.UNKNOWN;
        for (AST.Expr stmt : brane.statements()) {
            result = evalExpr(stmt, env);
        }
        return result;
    }

    private Value evalExpr(AST.Expr expr, Environment env) {
        if (expr instanceof AST.Assignment assign) {
            Value value = evalExpr(assign.expr(), env);
            env.set(assign.id(), value);
            return value;
        } else if (expr instanceof AST.IntegerLiteral lit) {
            return new IntValue(lit.value());
        } else if (expr instanceof AST.Identifier id) {
            String key = keyFor(id);
            return env.get(key);
        } else if (expr instanceof AST.BinaryExpr bin) {
            Value left = evalExpr(bin.left(), env);
            Value right = evalExpr(bin.right(), env);
            if (left instanceof IntValue l && right instanceof IntValue r) {
                return switch (bin.op()) {
                    case "+" -> new IntValue(l.value() + r.value());
                    case "-" -> new IntValue(l.value() - r.value());
                    case "*" -> new IntValue(l.value() * r.value());
                    case "/" -> new IntValue(l.value() / r.value());
                    case "^" -> new IntValue((long) Math.pow(l.value(), r.value()));
                    default -> Value.UNKNOWN;
                };
            }
            return Value.UNKNOWN;
        } else if (expr instanceof AST.UnaryExpr un) {
            Value v = evalExpr(un.expr(), env);
            if (v instanceof IntValue i) {
                return switch (un.op()) {
                    case "+" -> i;
                    case "-" -> new IntValue(-i.value());
                    case "*" -> i; // treat unary * as identity
                    default -> Value.UNKNOWN;
                };
            }
            return Value.UNKNOWN;
        } else if (expr instanceof AST.IfExpr iff) {
            if (truthy(evalExpr(iff.condition(), env))) {
                return evalExpr(iff.thenExpr(), env);
            }
            for (AST.IfExpr elif : iff.elseIfs()) {
                if (truthy(evalExpr(elif.condition(), env))) {
                    return evalExpr(elif.thenExpr(), env);
                }
            }
            return evalExpr(iff.elseExpr(), env);
        } else if (expr instanceof AST.Brane brane) {
            // Brane used as value, not executed
            return new BraneValue(brane);
        } else if (expr instanceof AST.Branes branes) {
            return evalBranes(branes);
        } else if (expr instanceof AST.UnknownExpr) {
            return Value.UNKNOWN;
        }
        return Value.UNKNOWN;
    }

    private boolean truthy(Value v) {
        return v instanceof IntValue i && i.value() != 0;
    }

    private String keyFor(AST.Identifier id) {
        String chr = id.cannoicalCharacterization();
        return (chr.isEmpty() ? "" : chr + ":") + id.cannonicalId();
    }
}
