package org.foolish.fvm.v1;

import org.foolish.ast.AST;

/**
 * Firoe wrapper for a binary expression.
 */
class BinaryFiroe extends Firoe {
    private final String op;
    private final Firoe left;
    private final Firoe right;

    BinaryFiroe(Insoe base, Firoe left, Firoe right) {
        this(base,
                base.as(AST.BinaryExpr.class).op(),
                left,
                right);
    }

    BinaryFiroe(Insoe base, String op, Firoe left, Firoe right) {
        super(base);
        this.op = op;
        this.left = left;
        this.right = right;
    }

    public String op() {
        return op;
    }

    public Firoe left() {
        return left;
    }

    public Firoe right() {
        return right;
    }

    public String toString() {
        return FormatterFactory.verbose().format(this);
    }
}
