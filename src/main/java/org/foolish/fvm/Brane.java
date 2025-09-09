package org.foolish.fvm;

import java.util.List;

/**
 * Base class for brane execution constructs.  A brane executes a sequence of
 * targoes within a new child environment providing copy-on-write semantics.
 */
public abstract class Brane extends Instruction {
    private final Characterizable characterization;

    protected Brane(Characterizable characterization, TargoeType type) {
        super(type);
        this.characterization = characterization;
    }

    public Characterizable characterization() {
        return characterization;
    }

    /**
     * @return sequential list of targoes/statements contained in this brane.
     */
    protected abstract List<Targoe> statements();

    protected EvalResult executeBraneStatement(Targoe stmt, Environment env) {
        return stmt.execute(env);
    }

    @Override
    public EvalResult execute(Environment env) {
        Environment local = env.child();
        Resoe result = Resoe.UNKNOWN;
        for (Targoe stmt : statements()) {
            EvalResult er = executeBraneStatement(stmt, local);
            result = er.value();
            local = er.env();
        }
        // changes are not propagated to parent to preserve copy-on-write
        return new EvalResult(result, env);
    }
}

