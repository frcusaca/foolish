package org.foolish.fvm;

import org.foolish.ast.AST;

/**
 * Midoe wrapper for a binary expression.
 */
class BinaryMidoe extends Midoe {
    private final String op;
    private final Midoe left;
    private final Midoe right;

    BinaryMidoe(Insoe base, Midoe left, Midoe right) {
        this(base,
                base.as(AST.BinaryExpr.class).op(),
                left,
                right);
    }

    BinaryMidoe(Insoe base, String op, Midoe left, Midoe right) {
        super(base);
        this.op = op;
        this.left = left;
        this.right = right;
    }

    public String op() {
        return op;
    }

    public Midoe left() {
        return left;
    }

    public Midoe right() {
        return right;
    }

    public String toString() {
        return FormatterFactory.verbose().format(this);
    }
}
