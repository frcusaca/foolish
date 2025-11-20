package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * FIR without a braneMind queue.
 * Used for simple values that don't require step-by-step evaluation.
 */
public abstract class FiroeWithoutBraneMind extends FIR {

    protected FiroeWithoutBraneMind(AST ast, String comment) {
        super(ast, comment);
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

    /**
     * FiroeWithoutBraneMind instances are never NYE (Not Yet Evaluated).
     * They represent finalized values.
     */
    @Override
    public boolean isNye() {
        return false;
    }
}
