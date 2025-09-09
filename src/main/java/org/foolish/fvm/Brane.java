package org.foolish.fvm;

import java.util.List;

/**
 * Base class for brane execution constructs.  A brane executes a sequence of
 * instructions within a given environment.
 */
public abstract class Brane implements Insoe {
    private final Characterizable characterization;

    protected Brane(Characterizable characterization) {
        this.characterization = characterization;
    }

    public Characterizable characterization() {
        return characterization;
    }

    protected abstract List<Midoe> statements();

    @Override
    public Finear execute(Environment env) {
        return BraneVm.execute(this, env);
    }
}
