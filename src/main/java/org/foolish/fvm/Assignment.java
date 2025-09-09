package org.foolish.fvm;

/**
 * Assigns the result of an expression to an identifier in the current
 * environment.  Because environments are immutable, the updated environment is
 * returned via the {@link EvalResult}.
 */
public class Assignment extends Instruction {
    private final Characterizable id;
    private final Targoe expr;

    public Assignment(Characterizable id, Targoe expr) {
        super(TargoeType.ASSIGNMENT);
        this.id = id;
        this.expr = expr;
    }

    @Override
    public EvalResult execute(Environment env) {
        EvalResult er = expr.execute(env);
        Environment updated = er.env().define(id, er.value());
        return new EvalResult(er.value(), updated);
    }
}

