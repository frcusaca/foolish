package org.foolish.fvm;

/**
 * Unary arithmetic operations on long values.
 */
public class UnaryExpr implements Insoe {
    private final String op;
    private final Insoe expr;

    public UnaryExpr(String op, Insoe expr) {
        this.op = op;
        this.expr = expr;
    }

    String op() { return op; }
    Insoe expr() { return expr; }

    @Override
    public Finear execute(Environment env) {
        return FinearVm.execute(this, env);
    }
}
