package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Stay-Foolish Marker FIR (SFMarkFiroe).
 * Implements the SF marker behavior: <expr>
 * <p>
 * Captures CONSTANIC copy WITHOUT further resolution. The marker:
 * - Phase A: Waits for wrapped expression to reach CONSTANIC
 * - Phase B: Calls copy(null) which returns the immutable CONSTANIC FIR as-is
 * - NO re-evaluation occurs - the CONSTANIC state is preserved
 * <p>
 * This differs from regular CMFir which uses copy(INITIALIZED) to re-evaluate
 * the expression in a new context.
 * <p>
 * Only affects brane references, not fresh branes.
 */
public class SFMarkFiroe extends CMFir {

    public SFMarkFiroe(AST.StayFoolishExpr ast, FIR o) {
        super(ast, o);
    }

    /**
     * SF marker differs from base CMFir in Phase B:
     * - Phase A: Wait for o to reach CONSTANIC
     * - Phase B: Create copy with copy(null) to preserve CONSTANIC state
     *
     * This captures the CONSTANIC state without further resolution.
     * The copy(null) call preserves the constanic state rather than re-evaluating.
     */
    @Override
    protected void startPhaseB() {
        phaseBStarted = true;
        // Use copy(null) to preserve CONSTANIC state - no re-evaluation
        o2 = o.copy(null);

        // Set o2's parent FIR to this CMFir
        o2.setParentFir(this);

        // If o2 is a BraneFiroe, recalculate its depth in the new context
        // This is critical for depth limit checking when branes are coordinated
        if (o2 instanceof BraneFiroe braneFiroe) {
            int newDepth = braneFiroe.calculateBraneDepth();
            braneFiroe.setExprmntBraneDepth(newDepth);
        }

        // If o2 has a braneMind, link its memory to the containing brane's memory
        if (o2 instanceof FiroeWithBraneMind fwbm) {
            BraneFiroe myBrane = getMyBrane();
            if (myBrane != null) {
                // Set o2's memory parent to the containing brane's memory
                fwbm.linkMemoryParent(myBrane.getBraneMemory());
            }
        }
        syncO2Nyes();
    }

    @Override
    public String toString() {
        return "SF(" + getResult() + ")";
    }
}
