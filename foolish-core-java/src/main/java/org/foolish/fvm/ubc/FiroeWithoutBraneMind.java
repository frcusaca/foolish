package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * FIR without a braneMind queue.
 * Used for simple values that don't require step-by-step evaluation.
 */
public abstract class FiroeWithoutBraneMind extends FIR {

    protected FiroeWithoutBraneMind(AST ast, String comment, boolean ai) {
        super(ast, comment, ai);
        // FiroeWithoutBraneMind instances are immediately CONSTANT
        setNyes(Nyes.CONSTANT);
    }

    protected FiroeWithoutBraneMind(AST ast, String comment) {
        this(ast, comment, false);
    }

    protected FiroeWithoutBraneMind(AST ast) {
        this(ast, null, false);
    }

    /**
     * Auto-instruction constructor.
     */
    protected FiroeWithoutBraneMind(String comment) {
        super(comment);
        setNyes(Nyes.CONSTANT);
    }

    /**
     * FiroeWithoutBraneMind instances don't require stepping as they represent finalized values.
     * This is a no-op implementation.
     *
     * @return 0 (no work done - already constant)
     */
    @Override
    public int step() {
        // If we are somehow reset to non-CONSTANT (e.g. via cloneConstanic resetting to INITIALIZED),
        // we must transition back to CONSTANT because we are intrinsically constant.
        // If we are somehow reset to non-CONSTANT (e.g. via cloneConstanic resetting to INITIALIZED),
        // we must transition back to CONSTANT because we are intrinsically constant.
        if (getNyes() != Nyes.CONSTANT) {
            System.out.println("FiroeWithoutBraneMind " + this + " resetting to CONSTANT from " + getNyes());
            setNyes(Nyes.CONSTANT);
            return 1;
        }
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
