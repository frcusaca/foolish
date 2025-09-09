package org.foolish.fvm;

/**
 * Assigns the result of an expression to an identifier in the current
 * environment.
 */
public class Assignment implements Insoe {
    private final Characterizable id;
    private final Midoe expr;

    public Assignment(Characterizable id, Midoe expr) {
        this.id = id;
        this.expr = expr;
    }

    Characterizable id() { return id; }
    Midoe expr() { return expr; }

    @Override
    public Finear execute(Environment env) {
        return AssignmentVm.execute(this, env);
    }
}
