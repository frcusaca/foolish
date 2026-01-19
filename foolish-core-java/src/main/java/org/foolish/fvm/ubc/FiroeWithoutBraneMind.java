package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * FIR without a braneMind queue.
 * Used for simple values that don't require step-by-step evaluation.
 */
public abstract class FiroeWithoutBraneMind extends FIR {

    protected FiroeWithoutBraneMind(AST ast, String comment) {
        super(ast, comment);
        // FiroeWithoutBraneMind instances are immediately CONSTANT
        setNyes(Nyes.CONSTANT);
    }

    protected FiroeWithoutBraneMind(AST ast) {
        this(ast, null);
    }

    /**
     * FiroeWithoutBraneMind instances don't require stepping as they represent finalized values.
     * This is a no-op implementation.
     */
    @Override
    public void step() {
        // No-op: Values don't require stepping
    }

    // Removed isNye override, relies on Nyes state (CONSTANT)

    /**
     * Values are generally not constantic (unresolved).
     * Subclasses like NKFiroe might override behavior if they represent errors,
     * but base assumption is False.
     * Wait, FIR.isConstantic checks Nyes state.
     * Since FiroeWithoutBraneMind sets Nyes.CONSTANT, isConstantic() returns TRUE because CONSTANT >= CONSTANTIC.
     *
     * However, FiroeWithoutBraneMind typically represents fully resolved values (like integers) or Errors.
     * If it is a Value, isConstantic() should be true (as it is constant).
     * The `atConstantic()` method will return false.
     *
     * So we don't need to override isConstantic().
     */
    @Override
    public boolean isConstantic() {
        return super.isConstantic();
    }
}
