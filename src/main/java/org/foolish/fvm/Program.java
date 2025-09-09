package org.foolish.fvm;

/**
 * FVM program consisting of a top-level brane.
 */
public class Program implements Insoe {
    private final Brane brane;

    public Program(Brane brane) {
        this.brane = brane;
    }

    Brane brane() { return brane; }

    @Override
    public Finear execute(Environment env) {
        return FinearVm.execute(this, env);
    }
}
