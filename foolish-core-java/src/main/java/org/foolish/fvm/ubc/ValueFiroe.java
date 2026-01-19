package org.foolish.fvm.ubc;

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
    public long getValue() {
        return value;
    }

    /**
     * ValueFiroe is never abstract as it represents a concrete value.
     */

    public String toString() {
        return String.valueOf(value);
    }

    public FIR clone(){
        return this; // I'm a constant
    }
}
