package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.fvm.Env;

/**
 * Foolish Internal Representation (FIR).
 * The FIR is the internal representation of computation that holds an AST
 * and tracks evaluation progress.
 */
public abstract class FIR {
    protected final AST ast;
    protected final String comment;
    private boolean initialized;

    protected FIR(AST ast, String comment) {
        this.ast = ast;
        this.comment = comment;
        this.initialized = false;
    }

    protected FIR(AST ast) {
        this(ast, null);
    }

    /**
     * Returns the AST node associated with this FIR.
     */
    public AST ast() {
        return ast;
    }

    /**
     * Returns the optional comment for this FIR.
     */
    public String comment() {
        return comment;
    }

    /**
     * Returns whether this FIR has been initialized.
     */
    protected boolean isInitialized() {
        return initialized;
    }

    /**
     * Sets the initialized state of this FIR.
     */
    protected void setInitialized() {
        this.initialized = true;
    }

    /**
     * Perform one step of evaluation on this FIR.
     * This method should advance the FIR's evaluation state by one step.
     * For simple values that don't require stepping, this can be a no-op.
     */
    public abstract void step();

    /**
     * Query method returning false if an additional step on this FIR does not change it.
     * Returns true when an additional step would change the FIR.
     * Not Yet Evaluated (NYE) indicates the FIR requires further evaluation steps.
     */
    public abstract boolean isNye();

    /**
     * Query method returning false only when all identifiers are bound.
     * Returns true if there are unbound identifiers (abstract state).
     */
    public abstract boolean isAbstract();

    /**
     * Gets the value from this FIR if it represents a simple value.
     * For ValueFiroe and evaluated expressions, returns the integer value.
     * For BraneFiroe, throws an exception (use getEnvironment instead).
     *
     * @return the integer value
     * @throws UnsupportedOperationException if this FIR doesn't support getValue
     * @throws IllegalStateException         if this FIR is not fully evaluated
     */
    public long getValue() {
        throw new UnsupportedOperationException("getValue not supported for " + getClass().getSimpleName());
    }

    /**
     * Gets the environment from this FIR if it represents a brane.
     * For BraneFiroe, returns the frozen Env after full evaluation.
     * For other types, throws an exception.
     *
     * @return the environment
     * @throws UnsupportedOperationException if this FIR doesn't support getEnvironment
     * @throws IllegalStateException         if this FIR is not fully evaluated
     */
    public Env getEnvironment() {
        throw new UnsupportedOperationException("getEnvironment not supported for " + getClass().getSimpleName());
    }

    /**
     * Creates a FIR from an AST expression.
     */
    protected static FIR createFiroeFromExpr(AST.Expr expr) {
        switch (expr) {
            case AST.IntegerLiteral literal -> {
                return new ValueFiroe(expr, literal.value());
            }
            case AST.BinaryExpr binary -> {
                return new BinaryFiroe(binary);
            }
            case AST.UnaryExpr unary -> {
                return new UnaryFiroe(unary);
            }
            case AST.IfExpr ifExpr -> {
                return new IfFiroe(ifExpr);
            }
            case AST.Brane brane -> {
                return new BraneFiroe(brane);
            }
            case AST.Assignment assignment -> {
                return new AssignmentFiroe(assignment);
            }
            case AST.Identifier identifier -> {
                return new IdentifierFiroe(identifier);
            }
            default -> {
                // Placeholder for unsupported types
                return new ValueFiroe(0L);
            }
        }
    }
}
