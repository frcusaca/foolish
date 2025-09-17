package org.foolish.fvm;

/**
 * Literal long value.
 */
public class IntegerLiteral extends Finer implements Insoe {
    private final long value;

    public IntegerLiteral(long value) {
        this.value = value;
    }

    @Override
    public Long value() { return value; }

    @Override
    public Targoe execute(Environment env) {
        return this;
    }
}

