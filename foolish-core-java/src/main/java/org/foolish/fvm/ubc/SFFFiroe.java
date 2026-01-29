package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * DEPRECATED: Stay-Fully-Foolish marker <<==>> (LT-LT-EQ-GT-GT) is low priority.
 * <p>
 * This marker is complex due to AI tokenization issues with the symbol sequence.
 * The <<==>> syntax is difficult for AI models to process reliably in code contexts.
 * <p>
 * Commented out to focus on detachment branes and SF marker implementation.
 * The SFF marker would provide full re-evaluation of expressions by reconstructing
 * from AST, creating fresh instances with no prior coordination.
 * <p>
 * TODO: Re-enable after detachment implementation is stable.
 * See projects/003-Detachment_Project.md for continuation plan.
 * <p>
 * IMPLEMENTATION NOTE: When re-enabled, SFF should override recreateFromAst()
 * to always reconstruct from AST, ignoring any coordinations. This is the key
 * difference from SF marker behavior.
 */

/*
 * Full implementation commented out below for future reference:
 *
public class SFFFiroe extends CMFir {

    public SFFFiroe(AST.StayFullyFoolishExpr ast, FIR o) {
        super(ast, o);
    }

    @Override
    protected FIR recreateFromAst(FIR original) {
        // SFF always reconstructs from AST, ignoring any coordinations
        // This is the key difference from SF behavior
        if (original.ast() instanceof AST.Expr expr) {
            return FIR.createFiroeFromExpr(expr);
        }
        if (original instanceof AssignmentFiroe af) {
            return new AssignmentFiroe((AST.Assignment) af.ast());
        }
        throw new UnsupportedOperationException(
            formatErrorMessage("Cannot SFF-recreate FIR with AST type: " + original.ast().getClass()));
    }

    @Override
    public String toString() {
        return "SFF(" + getResult() + ")";
    }
}
*/
