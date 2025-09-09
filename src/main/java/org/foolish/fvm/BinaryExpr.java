package org.foolish.fvm;

/**
 * Basic binary arithmetic operations on long values.
 */
public class BinaryExpr implements Insoe {
    private final String op;
    private final Insoe left;
    private final Insoe right;

    public BinaryExpr(String op, Insoe left, Insoe right) {
        this.op = op;
        this.left = left;
        this.right = right;
    }

    String op() { return op; }
    Insoe left() { return left; }
    Insoe right() { return right; }

    @Override
    public Finear execute(Environment env) {
        return FinearVm.execute(this, env);
    }
}
