package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Stay-Foolish FIR (SFMarkFiroe).
 * Implements the SF marker behavior: <expr>
 * Reactivates original liberations without additional resolution.
 * Only affects brane references, not fresh branes.
 */
public class SFMarkFiroe extends CMFir {

    public SFMarkFiroe(AST.StayFoolishExpr ast, FIR o) {
        super(ast, o);
    }

    @Override
    protected FIR makeCopy(FIR o) {
        // Create a CONSTANIC copy which does not continue to resolve
        return o.copy(Nyes.CONSTANIC);
    }

    @Override
    public String toString() {
        return "SF(" + getResult() + ")";
    }
}
