package org.foolish.fvm;

/**
 * Literal long value.
 */
public class IntegerLiteral implements Instruction {
    private final long value;

    public IntegerLiteral(long value) {
        this.value = value;
    }

    public long value() { return value; }

    @Override
    public Object execute(Environment env) {
        return value;
    }
}
