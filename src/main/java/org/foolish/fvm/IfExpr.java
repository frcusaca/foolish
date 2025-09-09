package org.foolish.fvm;

import java.util.List;

/**
 * Implements an if/elif/else expression.
 */
public class IfExpr implements Insoe {
    private final Midoe condition;
    private final Midoe thenExpr;
    private final Midoe elseExpr;
    private final List<IfExpr> elseIfs;

    public IfExpr(Midoe condition, Midoe thenExpr, Midoe elseExpr, List<IfExpr> elseIfs) {
        this.condition = condition;
        this.thenExpr = thenExpr;
        this.elseExpr = elseExpr;
        this.elseIfs = elseIfs == null ? List.of() : List.copyOf(elseIfs);
    }

    Midoe condition() { return condition; }
    Midoe thenExpr() { return thenExpr; }
    Midoe elseExpr() { return elseExpr; }
    List<IfExpr> elseIfs() { return elseIfs; }

    @Override
    public Finear execute(Environment env) {
        return FinearVm.execute(this, env);
    }
}
