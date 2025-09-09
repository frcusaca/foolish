package org.foolish.fvm;

/**
 * Executes {@link Assignment} instructions.
 */
public final class AssignmentVm {
    private AssignmentVm() {}

    public static Finear execute(Assignment asg, Environment env) {
        Finear value = asg.expr().evaluate(env);
        env.define(asg.id(), value);
        return value;
    }
}
