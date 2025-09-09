package org.foolish.fvm;

/**
 * Basic binary arithmetic operations on long values.
 */
public class BinaryExpr implements Insoe {
    private final String op;
    private final Midoe left;
    private final Midoe right;

    public BinaryExpr(String op, Midoe left, Midoe right) {
        this.op = op;
        this.left = left;
        this.right = right;
    }

    String op() { return op; }
    Midoe left() { return left; }
    Midoe right() { return right; }

    @Override
    public Finear execute(Environment env) {
        return BinaryExprVm.execute(this, env);
    }
}
