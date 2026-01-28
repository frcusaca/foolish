package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Stay-Fully-Foolish FIR (SFMarkFFiroe).
 * Implements the SFF marker behavior: <<expr>>
 * Reconstructs from AST, creating fresh instance with no prior coordination.
 */
public class SFMarkFFiroe extends CMFir {

    public SFMarkFFiroe(AST.StayFullyFoolishExpr ast, FIR o) {
        super(ast, o);
    }

    /**
     * SFF marker behavior differs from SF in Phase B:
     * Instead of cloning with preserved coordinations,
     * it reconstructs entirely from the original AST.
     */
    @Override
    protected FIR makeCopy(FIR o) {
        // SFF always reconstructs from AST, ignoring any coordinations
        if (o.ast() instanceof AST.Expr expr) {
            return FIR.createFiroeFromExpr(expr);
        }
        if (o instanceof AssignmentFiroe af) {
            return new AssignmentFiroe((AST.Assignment) af.ast());
        }
        throw new UnsupportedOperationException("Cannot SFF-clone FIR with AST type: " + o.ast().getClass());
    }

    @Override
    public String toString() {
        return "SFF(" + getResult() + ")";
    }
}
