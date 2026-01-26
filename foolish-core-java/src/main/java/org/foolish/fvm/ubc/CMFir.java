package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Context Manipulation FIR (CMFir).
 * Adapts a FIR to the current context (brane).
 * Used for handling "Stay Foolish" behavior where a brane or expression
 * defined in one scope is evaluated in another.
 * <p>
 * <b>Nested CMFir Wrapping:</b> When a CMFir wraps another CMFir as its object,
 * the current implementation directly wraps the inner CMFir without optimization.
 * This creates multiple levels of indirection (CMFir(CMFir(CMFir(...)))).
 * Future optimization could potentially flatten these nested CMFirs to reduce
 * indirection levels, but for now the straightforward approach is used.
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

        if (atConstant())  return 0;

        if (!phaseBStarted && o.atConstanic()){
                startPhaseB();
	}
        // Phase A: Step o until it is CONSTANT or CONSTANIC
        if (!phaseBStarted) {
            o.step();

            if (o.atConstant()) {
                // O reached CONSTANT without being CONSTANIC
                // This means it's fully resolved (not abstract)
                setNyes(Nyes.CONSTANT);
            }
            return 1;
        } else {
            // Phase B: Step o2 (the clone in the new context)
            o2.step();
	    syncO2Nyes();
            return 1;
        }
    }

    private void syncO2Nyes(){
	    if (o2.atConstant()) {
                setNyes(Nyes.CONSTANT);
            } else if (o2.atConstanic()) {
                // If o2 ends up constanic, we're also constanic
                setNyes(Nyes.CONSTANIC);
            }

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
	syncO2Nyes();
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
            // Check for NK (not-known) cases before attempting getValue
            if (o2 instanceof AssignmentFiroe af && af.getResult() instanceof NKFiroe nk) {
                throw new IllegalStateException("Cannot get value from NK (not-known): " + nk.getNkComment());
            }
            if (o2 instanceof NKFiroe nk) {
                throw new IllegalStateException("Cannot get value from NK (not-known): " + nk.getNkComment());
            }
            // Verify o2 is fully evaluated
            if (!o2.atConstant()) {
                throw new IllegalStateException("CMFir not fully evaluated (o2 not constant, state: " + o2.getNyes() + ")");
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
