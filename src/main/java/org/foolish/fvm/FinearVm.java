package org.foolish.fvm;

/**
 * Evaluation utilities that operate on {@link Midoe} trees and push them toward
 * {@link Finear} results.
 */
public final class FinearVm {
    private FinearVm() {}

    /** Evaluates the given midoe within the provided environment. */
    public static Finear evaluate(Midoe midoe, Environment env) {
        Finear result;
        if (midoe instanceof ProgramMidoe pm) {
            result = evaluate(pm.brane(), env);
        } else if (midoe instanceof BraneMidoe bm) {
            result = Finear.UNKNOWN;
            for (Midoe stmt : bm.statements()) {
                result = evaluate(stmt, env);
            }
        } else if (midoe instanceof AssignmentMidoe am) {
            Assignment base = (Assignment) am.base();
            Finear value = evaluate(am.expr(), env);
            env.define(base.id(), value);
            result = value;
        } else if (midoe instanceof BinaryMidoe bm) {
            BinaryExpr base = (BinaryExpr) bm.base();
            Finear l = evaluate(bm.left(), env);
            Finear r = evaluate(bm.right(), env);
            if (l.isUnknown() || r.isUnknown()) {
                result = Finear.UNKNOWN;
            } else {
                long lv = ((Number) l.value()).longValue();
                long rv = ((Number) r.value()).longValue();
                long val = switch (base.op()) {
                    case "+" -> lv + rv;
                    case "-" -> lv - rv;
                    case "*" -> lv * rv;
                    case "/" -> lv / rv;
                    default -> throw new IllegalArgumentException("Unknown op: " + base.op());
                };
                result = Finear.of(val);
            }
        } else if (midoe instanceof UnaryMidoe um) {
            UnaryExpr base = (UnaryExpr) um.base();
            Finear res = evaluate(um.expr(), env);
            if (res.isUnknown()) {
                result = Finear.UNKNOWN;
            } else {
                long v = ((Number) res.value()).longValue();
                long val = switch (base.op()) {
                    case "+" -> +v;
                    case "-" -> -v;
                    case "*" -> v; // '*' unary no-op for now
                    default -> throw new IllegalArgumentException("Unknown unary op: " + base.op());
                };
                result = Finear.of(val);
            }
        } else if (midoe instanceof IdentifierMidoe im) {
            IdentifierExpr base = (IdentifierExpr) im.base();
            result = env.lookup(base.id());
        } else if (midoe instanceof IfMidoe im) {
            if (asBoolean(evaluate(im.condition(), env))) {
                result = evaluate(im.thenExpr(), env);
            } else {
                result = Finear.UNKNOWN;
                for (IfMidoe elif : im.elseIfs()) {
                    if (asBoolean(evaluate(elif.condition(), env))) {
                        result = evaluate(elif.thenExpr(), env);
                        break;
                    }
                }
                if (result.isUnknown()) {
                    Midoe elseExpr = im.elseExpr();
                    if (elseExpr != null) {
                        result = evaluate(elseExpr, env);
                    }
                }
            }
        } else {
            Targoe base = midoe.base();
            if (base instanceof Finear f) {
                result = f;
            } else {
                result = Finear.UNKNOWN;
            }
        }
        midoe.progress_heap().add(result);
        return result;
    }

    /** Executes an instruction by wrapping it in a {@link Midoe}. */
    public static Finear execute(Insoe insoe, Environment env) {
        return evaluate(MidoeVm.wrap(insoe), env);
    }

    private static boolean asBoolean(Finear f) {
        if (f.isUnknown()) return false;
        Object o = f.value();
        if (o instanceof Boolean b) return b;
        if (o instanceof Number n) return n.longValue() != 0;
        return o != null;
    }
}
