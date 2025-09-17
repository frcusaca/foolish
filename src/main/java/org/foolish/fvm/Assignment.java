package org.foolish.fvm;

/**
 * Assigns the result of an expression to an identifier in the current
 * environment.
 */
public class Assignment implements Insoe {
    private final Characterizable id;
    private final Targoe expr;

    public Assignment(Characterizable id, Targoe expr) {
        this.id = id;
        this.expr = expr;
    }

    @Override
    public Targoe execute(Environment env) {
        Finer value = Evaluator.eval(expr, env);
        env.define(id, value);
        return value;
    }
}

