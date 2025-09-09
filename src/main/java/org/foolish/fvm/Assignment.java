package org.foolish.fvm;

/**
 * Assigns the result of an expression to an identifier in the current
 * environment.
 */
public class Assignment implements Insoe {
    private final Characterizable id;
    private final Insoe expr;

    public Assignment(Characterizable id, Insoe expr) {
        this.id = id;
        this.expr = expr;
    }

    Characterizable id() { return id; }
    Insoe expr() { return expr; }

    @Override
    public Finear execute(Environment env) {
        return FinearVm.execute(this, env);
    }
}
