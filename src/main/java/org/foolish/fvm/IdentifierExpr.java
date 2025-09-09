package org.foolish.fvm;

/**
 * Resolves the value of an identifier from the environment.
 */
public class IdentifierExpr extends Instruction {
    private final Characterizable id;

    public IdentifierExpr(Characterizable id) {
        super(TargoeType.IDENTIFIER_EXPR);
        this.id = id;
    }

    public Characterizable id() {
        return id;
    }

    @Override
    public EvalResult execute(Environment env) {
        return new EvalResult(env.lookup(id), env);
    }
}

