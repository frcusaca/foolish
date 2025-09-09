package org.foolish.fvm;

/**
 * Unary arithmetic operations on long values.
 */
public class UnaryExpr implements Insoe {
    private final String op;
    private final Midoe expr;

    public UnaryExpr(String op, Midoe expr) {
        this.op = op;
        this.expr = expr;
    }

    String op() { return op; }
    Midoe expr() { return expr; }

    @Override
    public Finear execute(Environment env) {
        return FinearVm.execute(this, env);
    }
}
