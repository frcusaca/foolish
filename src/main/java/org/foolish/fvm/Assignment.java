package org.foolish.fvm;

/**
 * Assigns the result of an expression to an identifier in the current
 * environment.
 */
public class Assignment implements Instruction {
    private final Characterizable id;
    private final Instruction expr;

    public Assignment(Characterizable id, Instruction expr) {
        this.id = id;
        this.expr = expr;
    }

    @Override
    public Object execute(Environment env) {
        Object value = expr.execute(env);
        env.define(id, value);
        return value;
    }
}
