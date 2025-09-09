package org.foolish.fvm;

/**
 * RESult Of Evaluation.  Literals and already evaluated expressions are
 * represented as {@code Resoe} instances.  A {@code Resoe} is itself an
 * {@link Instruction} whose execution simply yields itself.
 */
public class Resoe extends Instruction {
    private final Object value;

    /** pre-built instance representing an unknown value */
    public static final Resoe UNKNOWN = Unknown.INSTANCE;

    public Resoe(Object value) {
        super(TargoeType.RESOE);
        this.value = value;
    }

    public Object value() {
        return value;
    }

    /**
     * Convenience accessor interpreting the result as a long value.
     */
    public long asLong() {
        return ((Number) value).longValue();
    }

    @Override
    public EvalResult execute(Environment env) {
        return new EvalResult(this, env);
    }

    @Override
    public String toString() {
        return value == null ? "???" : value.toString();
    }
}

