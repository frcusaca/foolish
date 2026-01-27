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
        super(ast, comment, false);  // Has AST, not auto-generated
        this.nkComment = comment;
    }

    public NKFiroe(AST ast) {
        this(ast, null);
    }

    /**
     * Auto-instruction constructor: NK without AST.
     */
    public NKFiroe(String comment) {
        super(comment);  // Auto-instruction constructor
        this.nkComment = comment;
    }

    public NKFiroe() {
        this("Unknown reason");
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
     * NKFiroe is NOT constanic. It is a concrete (constant) error value.
     */

    /**
     * Cannot get a value from NK - it's not known.
     *
     * @throws IllegalStateException always
     */
    public long getValue() {
        String errorMsg = formatErrorMessage("Cannot get value from NK (not-known): " + nkComment);
        throw new IllegalStateException(errorMsg);
    }

    public String toString() {
        return "???";
    }

    public FIR clone() {
        return this; // I'm a constant
    }

}
