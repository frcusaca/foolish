package org.foolish.fvm;

/**
 * FVM program consisting of a top-level brane.
 */
public class Program implements Insoe {
    private final Brane brane;

    public Program(Brane brane) {
        this.brane = brane;
    }

    @Override
    public Finer execute(Environment env) {
        return brane.execute(env);
    }
}
