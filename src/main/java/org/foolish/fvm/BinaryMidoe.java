package org.foolish.fvm;

import org.foolish.ast.AST;

/** Midoe wrapper for a binary expression. */
class BinaryMidoe extends Midoe {
    private final String op;
    private final Midoe left;
    private final Midoe right;

    BinaryMidoe(Insoe base, Midoe left, Midoe right) {
        super(base);
        AST.BinaryExpr ast = base.as(AST.BinaryExpr.class);
        this.op = ast.op();
        this.left = left;
        this.right = right;
    }

    public String op() { return op; }
    public Midoe left() { return left; }
    public Midoe right() { return right; }
    public String toString() { return "MidoeBinary(" + left + op +  right + ")"; }
}
