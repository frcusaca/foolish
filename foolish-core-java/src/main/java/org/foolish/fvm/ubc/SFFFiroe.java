package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Stay-Fully-Foolish FIR (SFFFiroe).
 * Implements the SFF marker behavior: <<expr>>
 * Reconstructs from AST, creating fresh instance with no prior coordination.
 */
public class SFFFiroe extends CMFir {

    public SFFFiroe(AST.StayFullyFoolishExpr ast, FIR o) {
        super(ast, o);
    }

    /**
     * SFF marker behavior differs from SF in Phase B:
     * Instead of cloning with preserved coordinations,
     * it reconstructs entirely from the original AST.
     *
     * This is achieved by overriding stayFoolishClone to always reconstruct from AST.
     */

    @Override
    protected FIR stayFoolishClone(FIR original) {
        // SFF always reconstructs from AST, ignoring any coordinations
        // This is the key difference from SF behavior
        if (original.ast() instanceof AST.Expr expr) {
            return FIR.createFiroeFromExpr(expr);
        }
        if (original instanceof AssignmentFiroe af) {
            return new AssignmentFiroe((AST.Assignment) af.ast());
        }
        throw new UnsupportedOperationException("Cannot SFF-clone FIR with AST type: " + original.ast().getClass());
    }

    @Override
    public String toString() {
        return "SFF(" + getResult() + ")";
    }
}
