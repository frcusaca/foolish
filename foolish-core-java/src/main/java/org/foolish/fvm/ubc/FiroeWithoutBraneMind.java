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
     *
     * @return 0 (no work done - already constant)
     */
    @Override
    public int step() {
        // No-op: Values don't require stepping
        return 0;
    }

    // Removed isNye override, relies on Nyes state (CONSTANT)

    /**
     * Values are generally not constanic (unresolved).
     * Subclasses like NKFiroe might override behavior if they represent errors,
     * but base assumption is False.
     * Wait, FIR.isConstanic checks Nyes state.
     * Since FiroeWithoutBraneMind sets Nyes.CONSTANT, isConstanic() returns TRUE because CONSTANT >= CONSTANIC.
     *
     * However, FiroeWithoutBraneMind typically represents fully resolved values (like integers) or Errors.
     * If it is a Value, isConstanic() should be true (as it is constant).
     * The `atConstanic()` method will return false.
     *
     * So we don't need to override isConstanic().
     */
    @Override
    public boolean isConstanic() {
        return super.isConstanic();
    }
}
