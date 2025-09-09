package org.foolish.fvm;

import java.util.List;

/**
 * Implements an if/elif/else expression.
 */
public class IfExpr implements Insoe {
    private final Insoe condition;
    private final Insoe thenExpr;
    private final Insoe elseExpr;
    private final List<IfExpr> elseIfs;

    public IfExpr(Insoe condition, Insoe thenExpr, Insoe elseExpr, List<IfExpr> elseIfs) {
        this.condition = condition;
        this.thenExpr = thenExpr;
        this.elseExpr = elseExpr;
        this.elseIfs = elseIfs == null ? List.of() : List.copyOf(elseIfs);
    }

    Insoe condition() { return condition; }
    Insoe thenExpr() { return thenExpr; }
    Insoe elseExpr() { return elseExpr; }
    List<IfExpr> elseIfs() { return elseIfs; }

    @Override
    public Finear execute(Environment env) {
        return FinearVm.execute(this, env);
    }
}
