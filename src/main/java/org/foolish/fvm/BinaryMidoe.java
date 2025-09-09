package org.foolish.fvm;

/** Midoe wrapper for a {@link BinaryExpr}. */
class BinaryMidoe extends Midoe {
    private final Midoe left;
    private final Midoe right;

    BinaryMidoe(BinaryExpr base, Midoe left, Midoe right) {
        super(base);
        this.left = left;
        this.right = right;
    }

    public Midoe left() { return left; }
    public Midoe right() { return right; }
}
