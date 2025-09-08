package org.foolish.fvm;

/**
 * Resolves the value of an identifier from the environment.
 */
public class IdentifierExpr implements Instruction {
    private final Characterizable id;

    public IdentifierExpr(Characterizable id) {
        this.id = id;
    }

    public Characterizable id() { return id; }

    @Override
    public Object execute(Environment env) {
        return env.lookup(id);
    }
}
