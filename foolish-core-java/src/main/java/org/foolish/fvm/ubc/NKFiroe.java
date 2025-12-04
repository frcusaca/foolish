package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * NKFiroe represents a Not-Known (NK) value, displayed as ???.
 * This occurs when an operation cannot produce a valid result, such as:
 * - Division by zero
 * - Modulo by zero
 * - Other arithmetic errors
 * <p>
 * The comment field stores the reason for the NK value (e.g., "Division by zero").
 * <p>
 * Future: For floating-point operations, NaN should also result in NK.
 */
public class NKFiroe extends FiroeWithoutBraneMind {
    private final String nkComment;

    public NKFiroe(AST ast, String comment) {
        super(ast);
        this.nkComment = comment;
    }

    public NKFiroe(AST ast) {
        this(ast, null);
    }

    public NKFiroe(String comment) {
        this(null, comment);
    }

    public NKFiroe() {
        this(null, null);
    }

    /**
     * Gets the comment explaining why this value is not known.
     *
     * @return the comment, or null if no specific reason was provided
     */
    public String getNkComment() {
        return nkComment;
    }

    /**
     * NKFiroe is abstract because the value is not known.
     */
    @Override
    public boolean isAbstract() {
        return true;
    }

    /**
     * Cannot get a value from NK - it's not known.
     *
     * @throws IllegalStateException always
     */
    @Override
    public long getValue() {
        throw new IllegalStateException("Cannot get value from NK (not-known)");
    }

    @Override
    public String toString() {
        return "???";
    }

    public FIR clone() {
        return this; // I'm a constant
    }

}

