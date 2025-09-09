package org.foolish.fvm;

/** Midoe wrapper for a {@link UnaryExpr}. */
class UnaryMidoe extends Midoe {
    private final Midoe expr;

    UnaryMidoe(UnaryExpr base, Midoe expr) {
        super(base);
        this.expr = expr;
    }

    public Midoe expr() { return expr; }
}
