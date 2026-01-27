package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Stay-Foolish FIR (SFFiroe).
 * Implements the SF marker behavior: <expr>
 * Reactivates original liberations without additional resolution.
 * Only affects brane references, not fresh branes.
 */
public class SFFiroe extends CMFir {

    public SFFiroe(AST.StayFoolishExpr ast, FIR o) {
        super(ast, o);
    }

    /**
     * SF marker uses the default CMFir behavior:
     * - Phase A: Wait for o to reach CONSTANIC
     * - Phase B: Clone o and re-evaluate in current context (stayFoolishClone)
     *
     * This reactivates liberations that were originally uncoordinated.
     */
    // No override needed - default CMFir behavior is correct for SF

    @Override
    public String toString() {
        return "SF(" + getResult() + ")";
    }
}
