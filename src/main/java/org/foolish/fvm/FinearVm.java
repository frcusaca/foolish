package org.foolish.fvm;

/**
 * Evaluation utilities that operate on {@link Midoe} trees and push them
 * forward toward {@link Finear} results.
 */
public final class FinearVm {
    private FinearVm() {}

    /** Evaluates the given midoe within the provided environment. */
    public static Finear evaluate(Midoe midoe, Environment env) {
        if (midoe.progress_heap().isEmpty()) {
            return Finear.UNKNOWN;
        }
        Targoe top = midoe.progress_heap().get(midoe.progress_heap().size() - 1);
        if (top instanceof Finear finer) {
            if (midoe.progress_heap().size() == 1) {
                return finer;
            }
            top = midoe.progress_heap().get(0);
        }
        Finear result;
        if (top instanceof Insoe insoe) {
            result = execute(insoe, env);
        } else if (top instanceof Midoe child) {
            result = evaluate(child, env);
        } else {
            result = Finear.UNKNOWN;
        }
        midoe.progress_heap().add(result);
        return result;
    }

    /** Dispatches execution based on the concrete instruction type. */
    public static Finear execute(Insoe insoe, Environment env) {
        if (insoe instanceof Program p) {
            return executeProgram(p, env);
        } else if (insoe instanceof Brane b) {
            return executeBrane(b, env);
        } else if (insoe instanceof Assignment a) {
            return executeAssignment(a, env);
        } else if (insoe instanceof BinaryExpr b) {
            return executeBinary(b, env);
        } else if (insoe instanceof UnaryExpr u) {
            return executeUnary(u, env);
        } else if (insoe instanceof IdentifierExpr id) {
            return executeIdentifier(id, env);
        } else if (insoe instanceof IfExpr iff) {
            return executeIf(iff, env);
        }
        return Finear.UNKNOWN;
    }

    private static Finear executeProgram(Program program, Environment env) {
        return execute(program.brane(), env);
    }

    private static Finear executeBrane(Brane brane, Environment env) {
        Finear result = Finear.UNKNOWN;
        for (Midoe stmt : brane.statements()) {
            result = evaluate(stmt, env);
        }
        return result;
    }

    private static Finear executeAssignment(Assignment asg, Environment env) {
        Finear value = evaluate(asg.expr(), env);
        env.define(asg.id(), value);
        return value;
    }

    private static Finear executeBinary(BinaryExpr expr, Environment env) {
        Finear l = evaluate(expr.left(), env);
        Finear r = evaluate(expr.right(), env);
        if (l.isUnknown() || r.isUnknown()) return Finear.UNKNOWN;
        long lv = ((Number) l.value()).longValue();
        long rv = ((Number) r.value()).longValue();
        long res = switch (expr.op()) {
            case "+" -> lv + rv;
            case "-" -> lv - rv;
            case "*" -> lv * rv;
            case "/" -> lv / rv;
            default -> throw new IllegalArgumentException("Unknown op: " + expr.op());
        };
        return Finear.of(res);
    }

    private static Finear executeUnary(UnaryExpr expr, Environment env) {
        Finear res = evaluate(expr.expr(), env);
        if (res.isUnknown()) return Finear.UNKNOWN;
        long v = ((Number) res.value()).longValue();
        long val = switch (expr.op()) {
            case "+" -> +v;
            case "-" -> -v;
            case "*" -> v; // '*' unary no-op for now
            default -> throw new IllegalArgumentException("Unknown unary op: " + expr.op());
        };
        return Finear.of(val);
    }

    private static Finear executeIdentifier(IdentifierExpr expr, Environment env) {
        return env.lookup(expr.id());
    }

    private static Finear executeIf(IfExpr expr, Environment env) {
        if (asBoolean(evaluate(expr.condition(), env))) {
            return evaluate(expr.thenExpr(), env);
        }
        for (IfExpr elif : expr.elseIfs()) {
            if (asBoolean(evaluate(elif.condition(), env))) {
                return evaluate(elif.thenExpr(), env);
            }
        }
        Midoe elseExpr = expr.elseExpr();
        if (elseExpr != null) {
            return evaluate(elseExpr, env);
        }
        return Finear.UNKNOWN;
    }

    private static boolean asBoolean(Finear f) {
        if (f.isUnknown()) return false;
        Object o = f.value();
        if (o instanceof Boolean b) return b;
        if (o instanceof Number n) return n.longValue() != 0;
        return o != null;
    }
}
