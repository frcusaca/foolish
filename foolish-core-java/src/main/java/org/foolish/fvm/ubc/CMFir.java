package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Context Manipulation FIR (CMFir).
 * Adapts a FIR to the current context (brane).
 * Used for handling "Stay Foolish" behavior where a brane or expression
 * defined in one scope is evaluated in another.
 */
public class CMFir extends FiroeWithBraneMind {
    private FIR o;
    private FIR o2;
    private boolean phaseBStarted = false;

    public CMFir(AST ast, FIR o) {
        super(ast);
        this.o = o;
    }

    protected void initialize() {
        setInitialized();
        // We manage 'o' stepping manually in step(), not in braneMind
    }

    public int step() {
        if (!isInitialized()) {
            initialize();
            return 1;
        }

        if (getNyes() == Nyes.CONSTANT) return 0;

        // Phase A: Step o until it is CONSTANT or CONSTANIC
        if (!phaseBStarted) {
            // Check if o is CONSTANIC *before* stepping.
            if (isConstanic(o)) {
                startPhaseB();
                return 1;
            }

            try {
                o.step();
            } catch (Exception e) {
                throw e;
            }

            if (o.getNyes() == Nyes.CONSTANT) {
                // If o becomes CONSTANT, we check if it is Constanic (e.g. NK).
                if (o.isConstanic()) {
                    startPhaseB();
                    return 1;
                }

                setNyes(Nyes.CONSTANT);
                return 1;
            }
            return 1;
        } else {
            // Phase B: Step o2 (the clone in the new context)
            o2.step();

            // Mirror o2's state to CMFir
            if (o2.getNyes() == Nyes.CONSTANT) {
                setNyes(Nyes.CONSTANT);
            } else if (o2.atConstanic()) {
                // If o2 ends up constanic, we're also constanic
                setNyes(Nyes.CONSTANIC);
            }
            return 1;
        }
    }

    private boolean isConstanic(FIR f) {
        return f.isConstanic();
    }

    private void startPhaseB() {
        phaseBStarted = true;
        // make a stay_foolish_clone of o
        o2 = stayFoolishClone(o);

        // set o2's sub-expressions parent to o2, o2's parent to c.
        if (o2 instanceof FiroeWithBraneMind fwbm) {
            // We link o2 to this CMFir's memory context
            fwbm.braneMemory.setParent(this.braneMemory);
        }
    }

    private FIR stayFoolishClone(FIR original) {
        // Simplified implementation: Create fresh FIR from AST.
        // This effectively "re-evaluates" the expression in the new context.
        if (original instanceof AssignmentFiroe af) {
             return new AssignmentFiroe((AST.Assignment) af.ast());
        }
        if (original.ast() instanceof AST.Expr expr) {
             return FIR.createFiroeFromExpr(expr);
        }
        // Fallback for non-Expr ASTs if any (shouldn't happen for FIRs wrapping Expr)
        throw new UnsupportedOperationException("Cannot clone FIR with AST type: " + original.ast().getClass());
    }



    public long getValue() {
        if (phaseBStarted) {
            if (o2.isConstanic()) {
                 if (o2 instanceof AssignmentFiroe af && af.getResult() instanceof NKFiroe nk) {
                     throw new IllegalStateException("Cannot get value from NK (not-known): " + nk.getNkComment());
                 }
                 if (o2.getNyes() != Nyes.CONSTANT) {
                    throw new IllegalStateException("CMFir not fully evaluated (o2 not constant)");
                 }
                 // If CONSTANT but still constanic (e.g. NK directly)
                 if (o2 instanceof NKFiroe nk) {
                     throw new IllegalStateException("Cannot get value from NK (not-known): " + nk.getNkComment());
                 }
            }
            return o2.getValue();
        } else {
            return o.getValue();
        }
    }

    public FIR getResult() {
         return phaseBStarted ? o2 : o;
    }

    public String toString() {
        return "CMFir(" + (phaseBStarted ? o2 : o) + ")";
    }
}
