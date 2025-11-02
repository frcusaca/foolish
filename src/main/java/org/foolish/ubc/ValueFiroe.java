package org.foolish.ubc;

import org.foolish.ast.AST;

/**
 * ValueFiroe represents a finalized value in the computation.
 * For now, supports integral long values.
 */
public class ValueFiroe extends FiroeWithoutBraneMind {
    private final long value;

    public ValueFiroe(AST ast, long value) {
        super(ast);
        this.value = value;
    }

    public ValueFiroe(long value) {
        this(null, value);
    }

    /**
     * Returns the integral value stored in this ValueFiroe.
     */
    @Override
    public long getValue() {
        return value;
    }

    /**
     * ValueFiroe is never abstract as it represents a concrete value.
     */
    @Override
    public boolean isAbstract() {
        return false;
    }

    @Override
    public String toString() {
        return String.valueOf(value);
    }
}
