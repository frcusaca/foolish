package org.foolish.fvm;

import java.util.List;

/**
 * Implements an if/elif/else expression.
 */
public class IfExpr implements Insoe {
    private final Targoe condition;
    private final Targoe thenExpr;
    private final Targoe elseExpr;
    private final List<IfExpr> elseIfs;

    public IfExpr(Targoe condition, Targoe thenExpr, Targoe elseExpr, List<IfExpr> elseIfs) {
        this.condition = condition;
        this.thenExpr = thenExpr;
        this.elseExpr = elseExpr;
        this.elseIfs = elseIfs == null ? List.of() : List.copyOf(elseIfs);
    }

    private boolean asBoolean(Finer f) {
        if (f.isUnknown()) return false;
        Object o = f.value();
        if (o instanceof Boolean b) return b;
        if (o instanceof Number n) return n.longValue() != 0;
        return o != null;
    }

    @Override
    public Finer execute(Environment env) {
        if (asBoolean(Midoe.evaluate(condition, env))) {
            return Midoe.evaluate(thenExpr, env);
        }
        for (IfExpr elif : elseIfs) {
            if (asBoolean(Midoe.evaluate(elif.condition, env))) {
                return Midoe.evaluate(elif.thenExpr, env);
            }
        }
        if (elseExpr != null) {
            return Midoe.evaluate(elseExpr, env);
        }
        return Finer.UNKNOWN;
    }
}
