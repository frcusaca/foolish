package org.foolish.fvm;

import java.util.List;

/**
 * Executes {@link IfExpr} instructions.
 */
public final class IfExprVm {
    private IfExprVm() {}

    public static Finear execute(IfExpr expr, Environment env) {
        if (asBoolean(expr.condition().evaluate(env))) {
            return expr.thenExpr().evaluate(env);
        }
        for (IfExpr elif : expr.elseIfs()) {
            if (asBoolean(elif.condition().evaluate(env))) {
                return elif.thenExpr().evaluate(env);
            }
        }
        Midoe elseExpr = expr.elseExpr();
        if (elseExpr != null) {
            return elseExpr.evaluate(env);
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
