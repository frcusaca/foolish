package org.foolish.fvm;

import java.util.List;

/**
 * Base class for brane execution constructs.  A brane executes a sequence of
 * instructions within a given environment.
 */
public abstract class Brane implements Instruction {
    private final Characterizable characterization;

    protected Brane(Characterizable characterization) {
        this.characterization = characterization;
    }

    public Characterizable characterization() {
        return characterization;
    }

    protected abstract List<Instruction> statements();

    protected Object executeBraneStatement(Instruction stmt, Environment env) {
        if (stmt instanceof Brane b) {
            return b.execute(new Environment(env));
        }
        return stmt.execute(env);
    }

    @Override
    public Object execute(Environment env) {
        Object result = null;
        for (Instruction stmt : statements()) {
            result = executeBraneStatement(stmt, env);
        }
        return result;
    }
}
