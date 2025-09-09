package org.foolish.fvm;

/** Midoe wrapper for an {@link Assignment}. */
class AssignmentMidoe extends Midoe {
    private final Midoe expr;

    AssignmentMidoe(Assignment base, Midoe expr) {
        super(base);
        this.expr = expr;
    }

    public Midoe expr() {
        return expr;
    }
}
