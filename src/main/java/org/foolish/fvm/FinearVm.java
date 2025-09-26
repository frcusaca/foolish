package org.foolish.fvm;

import org.foolish.ast.AST;

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
            Finear value = evaluate(am.expr(), env);
            env.define(am.id(), value);
            result = value;
        } else if (midoe instanceof BinaryMidoe bm) {
            Finear l = evaluate(bm.left(), env);
            Finear r = evaluate(bm.right(), env);
            if (l.isUnknown() || r.isUnknown()) {
                result = Finear.UNKNOWN;
            } else {
                long lv = ((Number) l.value()).longValue();
                long rv = ((Number) r.value()).longValue();
                long val = switch (bm.op()) {
                    case "+" -> lv + rv;
                    case "-" -> lv - rv;
                    case "*" -> lv * rv;
                    case "/" -> lv / rv;
                    default -> throw new IllegalArgumentException("Unknown op: " + bm.op());
                };
                result = Finear.of(val);
            }
        } else if (midoe instanceof UnaryMidoe um) {
            Finear res = evaluate(um.expr(), env);
            if (res.isUnknown()) {
                result = Finear.UNKNOWN;
            } else {
                long v = ((Number) res.value()).longValue();
                long val = switch (um.op()) {
                    case "+" -> +v;
                    case "-" -> -v;
                    case "*" -> v; // '*' unary no-op for now
                    default -> throw new IllegalArgumentException("Unknown unary op: " + um.op());
                };
                result = Finear.of(val);
            }
        } else if (midoe instanceof IdentifierMidoe im) {
            result = env.lookup(im.id());
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
        } else if (midoe.base() instanceof Insoe in && in.ast() instanceof AST.UnknownExpr) {
            result = Finear.UNKNOWN;
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
        if (f.isUnknown()) return Finear.UNKNOWN ;
        Object o = f.value();
        if (o instanceof Boolean b) return b;
        if (o instanceof Number n) return n.longValue() != 0;
        return o != null;
    }
}
