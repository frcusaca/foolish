package org.foolish.fvm;

import java.util.List;

/** Midoe wrapper for an {@link IfExpr}. */
class IfMidoe extends Midoe {
    private final Midoe condition;
    private final Midoe thenExpr;
    private final Midoe elseExpr;
    private final List<IfMidoe> elseIfs;

    IfMidoe(IfExpr base, Midoe condition, Midoe thenExpr, Midoe elseExpr, List<IfMidoe> elseIfs) {
        super(base);
        this.condition = condition;
        this.thenExpr = thenExpr;
        this.elseExpr = elseExpr;
        this.elseIfs = elseIfs == null ? List.of() : List.copyOf(elseIfs);
    }

    public Midoe condition() { return condition; }
    public Midoe thenExpr() { return thenExpr; }
    public Midoe elseExpr() { return elseExpr; }
    public List<IfMidoe> elseIfs() { return elseIfs; }
}
