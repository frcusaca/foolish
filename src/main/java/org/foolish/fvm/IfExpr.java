package org.foolish.fvm;

import java.util.List;

/**
 * Implements an if/elif/else expression.
 */
public class IfExpr implements Instruction {
    private final Instruction condition;
    private final Instruction thenExpr;
    private final Instruction elseExpr;
    private final List<IfExpr> elseIfs;

    public IfExpr(Instruction condition, Instruction thenExpr, Instruction elseExpr, List<IfExpr> elseIfs) {
        this.condition = condition;
        this.thenExpr = thenExpr;
        this.elseExpr = elseExpr;
        this.elseIfs = elseIfs == null ? List.of() : List.copyOf(elseIfs);
    }

    private boolean asBoolean(Object o) {
        if (o instanceof Boolean b) return b;
        if (o instanceof Number n) return n.longValue() != 0;
        return o != null;
    }

    @Override
    public Object execute(Environment env) {
        if (asBoolean(condition.execute(env))) {
            return thenExpr.execute(env);
        }
        for (IfExpr elif : elseIfs) {
            if (asBoolean(elif.condition.execute(env))) {
                return elif.thenExpr.execute(env);
            }
        }
        if (elseExpr != null) {
            return elseExpr.execute(env);
        }
        return null;
    }
}
