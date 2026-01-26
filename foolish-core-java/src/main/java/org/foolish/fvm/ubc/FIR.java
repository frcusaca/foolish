package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.ast.SearchOperator;

/**
 * Foolish Internal Representation (FIR).
 * The FIR is the internal representation of computation that holds an AST
 * and tracks evaluation progress.
 */
public abstract class FIR {
    protected final AST ast;
    protected final String comment;
    private boolean initialized;
    private Nyes nyes;
    private FIR parentFir = null;  // The FIR that contains this FIR (set during enqueue)

    protected FIR(AST ast, String comment) {
        this.ast = ast;
        this.comment = comment;
        this.initialized = false;
        this.nyes = Nyes.UNINITIALIZED;
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
     * Returns the current NYE state of this FIR.
     */
    protected Nyes getNyes() {
        return nyes;
    }

    /**
     * Sets the NYE state of this FIR.
     * All changes to a Firoe's Nyes must be made through this method.
     */
    protected void setNyes(Nyes nyes) {
        this.nyes = nyes;
    }

    /**
     * Sets the parent FIR that contains this FIR.
     * This is called when this FIR is enqueued into a parent's braneMind.
     */
    protected void setParentFir(FIR parent) {
        this.parentFir = parent;
    }

    /**
     * Gets the parent FIR that contains this FIR.
     * Returns null if this FIR has no parent (e.g., root brane).
     */
    protected FIR getParentFir() {
        return parentFir;
    }

    public final boolean atConstant() {
        return nyes == Nyes.CONSTANT;
    }

    public final boolean atConstanic() {
        return nyes == Nyes.CONSTANIC;
    }

    /**
     * Perform one step of evaluation on this FIR.
     * This method should advance the FIR's evaluation state by one step.
     * For simple values that don't require stepping, this can be a no-op.
     *
     * @return the amount of work done in this step:
     *         0 for empty transitions (no-op, already evaluated, or waiting)
     *         1 for meaningful work (state transitions, evaluations, searches)
     *         Values are accumulated for step counting
     */
    public abstract int step();

    /**
     * Query method returning false if an additional step on this FIR does not change it.
     * Returns true when an additional step would change the FIR.
     * Not Yet Evaluated (NYE) indicates the FIR requires further evaluation steps.
     * Anything before CONSTANIC is considered NYE.
     */
    public boolean isNye() {
        return nyes.ordinal() < Nyes.CONSTANIC.ordinal();
    }

    /**
     * Returns true if the state is CONSTANIC or later (CONSTANT).
     * Something that is CONSTANT is also Constanic.
     */
    public boolean isConstanic() {
        return nyes.ordinal() >= Nyes.CONSTANIC.ordinal();
    }

    /**
     * Returns true if the state is CONSTANT or later (which is just CONSTANT).
     */
    public boolean isConstant() {
        return nyes.ordinal() >= Nyes.CONSTANT.ordinal();
    }

    /**
     * Gets the value from this FIR if it represents a simple value.
     * For ValueFiroe and evaluated expressions, returns the integer value.
     *
     * @return the integer value
     * @throws UnsupportedOperationException if this FIR doesn't support getValue
     * @throws IllegalStateException         if this FIR is not fully evaluated
     */
    public long getValue() {
        throw new UnsupportedOperationException("getValue not supported for " + getClass().getSimpleName());
    }

    /**
     * Gets the BraneFiroe that contains this FIR in its statement list.
     * Chains through parent FIRs until finding one whose parent is a BraneFiroe.
     * <p>
     * The containing brane is the closest BraneFiroe in the FIR hierarchy,
     * representing the brane where this FIR appears as a statement.
     * <p>
     * Parallel expressions (such as operands in a+b) are at the "same height"
     * and all statements in a brane are parallel/same height. The height does
     * NOT deepen with deepening FIR structures - height only changes when
     * crossing brane boundaries.
     *
     * @return the containing BraneFiroe, or null if this FIR is not contained
     *         in a brane (e.g., at root level)
     */
    public BraneFiroe getMyBrane() {
        // Chain through parents until we find one whose parent is a BraneFiroe
        if (parentFir instanceof BraneFiroe bf) {
		return bf;
	}
	return parentFir.getMyBrane();
    }

    /**
     * Gets the index of this FIR in its containing brane's memory.
     * Chains through parent FIRs to find the statement-level FIR, then returns its position.
     * <p>
     * The brane index defines the "order of expressions" within a height level.
     * All statements at the same brane level are parallel/same height, and the
     * index orders them for operations like unanchored backward search.
     * <p>
     * Note: The index is for the statement containing this FIR, not necessarily
     * this exact FIR object. For example, if this is a sub-expression of an
     * assignment, it returns the assignment's index in the brane.
     *
     * @return the index in the containing brane's memory (0-based), or -1 if
     *         this FIR is not in a brane (root level)
     */
    public int getMyBraneIndex() {
        return switch (parentFir) {
            case null ->  -1;
	    case BraneFiroe bf -> bf.getIndexOf(this);
	    default -> parentFir.getMyBraneIndex();
	};
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
            case AST.RegexpSearchExpr regexpSearch -> {
                if (DerefSearchFiroe.isExactMatch(regexpSearch.pattern())) {
                    return new DerefSearchFiroe(regexpSearch);
                }
                return new RegexpSearchFiroe(regexpSearch);
            }
            case AST.OneShotSearchExpr oneShotSearch -> {
                return new OneShotSearchFiroe(oneShotSearch);
            }
            case AST.DereferenceExpr dereferenceExpr -> {
                AST.RegexpSearchExpr synthetic = new AST.RegexpSearchExpr(dereferenceExpr.anchor(), SearchOperator.REGEXP_LOCAL, dereferenceExpr.coordinate().toString());
                return new DerefSearchFiroe(synthetic, dereferenceExpr);
            }
            case AST.SeekExpr seekExpr -> {
                return new SeekFiroe(seekExpr);
            }
            case AST.UnanchoredSeekExpr unanchoredSeekExpr -> {
                return new UnanchoredSeekFiroe(unanchoredSeekExpr);
            }
            default -> {
                // Placeholder for unsupported types
                return new NKFiroe();
            }
        }
    }
}
