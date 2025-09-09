package org.foolish.fvm;

/**
 * FVM program consisting of a top-level brane.
 */
public class Program extends Instruction {
    private final Brane brane;

    public Program(Brane brane) {
        super(TargoeType.PROGRAM);
        this.brane = brane;
    }

    @Override
    public EvalResult execute(Environment env) {
        return brane.execute(env);
    }
}

