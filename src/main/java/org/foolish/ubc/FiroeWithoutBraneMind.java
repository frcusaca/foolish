package org.foolish.ubc;

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
     * FiroeWithoutBraneMind instances are never underevaluated.
     * They represent finalized values.
     */
    @Override
    public boolean underevaluated() {
        return false;
    }
}
