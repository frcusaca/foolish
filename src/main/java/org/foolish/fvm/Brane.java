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

    protected abstract List<Targoe> statements();

    protected Targoe executeBraneStatement(Targoe stmt, Environment env) {
        return Evaluator.eval(stmt, env);
    }

    @Override
    public Targoe execute(Environment env) {
        Targoe result = Unknown.INSTANCE;
        for (Targoe stmt : statements()) {
            result = executeBraneStatement(stmt, env);
        }
        return result;
    }
}

