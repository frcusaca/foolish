package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Context Manipulation FIR (CMFir).
 * This is a state tracking FIR that will let a referenced object (o) reach Constanic
 * before making a copy of it as o2 and continuing to evaluate o2 in the new context.
 * <p>
 * CMFir operates in two phases:
 * - Phase A: Evaluate wrapped FIR (o) until it reaches CONSTANIC or CONSTANT
 * - Phase B: If o is CONSTANIC, clone it and re-evaluate in current brane's context
 * <p>
 * <b>Nested CMFir Wrapping:</b> When a CMFir wraps another CMFir as its object,
 * the current implementation directly wraps the inner CMFir without optimization.
 * This creates multiple levels of indirection (CMFir(CMFir(CMFir(...)))).
 * Future optimization could potentially flatten these nested CMFirs to reduce
 * indirection levels, but for now the straightforward approach is used.
 */
public class CMFir extends FiroeWithoutBraneMind {
    protected FIR o;
    protected FIR o2;
    protected boolean phaseBStarted = false;

    public CMFir(AST ast, FIR o) {
        super(ast);
        this.o = o;
        // CMFir starts as UNINITIALIZED (set by parent constructor)
        // No need to set again - parent FIR constructor already did this
    }

    @Override
    public int step() {
        if (atConstant()) return 0;

        if (!phaseBStarted && o.atConstanic()) {
            startPhaseB();
        }

        // Phase A: Step o until it is CONSTANT or CONSTANIC
        if (!phaseBStarted) {
            o.step();

            if (o.atConstant()) {
                // o reached CONSTANT without being CONSTANIC
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

    protected void syncO2Nyes(){
	    if (o2.atConstant()) {
                setNyes(Nyes.CONSTANT);
            } else if (o2.atConstanic()) {
                // If o2 ends up constanic, we're also constanic
                setNyes(Nyes.CONSTANIC);
            }

    }

    protected void startPhaseB() {
        phaseBStarted = true;

        // Clone the CONSTANIC FIR with updated parent chain and reset to INITIALIZED
        // cloneConstanic handles parent updating and state setting in one call
        o2 = o.cloneConstanic(this, java.util.Optional.of(Nyes.INITIALIZED));

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
                fwbm.braneMemory.setParent(myBrane.braneMemory);
            }
        }
        syncO2Nyes();
    }

    // Accessors for subclasses
    protected FIR getO() {
        return o;
    }

    protected void setO2(FIR o2) {
        this.o2 = o2;
    }

    protected void setPhaseBStarted(boolean started) {
        this.phaseBStarted = started;
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

    protected Nyes getNyes() {
	if(phaseBStarted){
		return o2.getNyes();
	}else{
		return o.getNyes();
	}
    }

    @Override
    public boolean isNye() {
        // CMFir is NYE until phase B completes
        if (!phaseBStarted) {
            // Phase A: NYE if o is still evaluating, OR if o is CONSTANIC (need to start phase B)
            return o.isNye() || o.atConstanic();
        } else {
            // Phase B: NYE while o2 is evaluating
            return o2.isNye();
        }
    }

    /**
     * Override copy() to unwrap CMFir when possible.
     * If this CMFir has completed phase B and inner FIR is CONSTANIC,
     * unwrap and return the inner FIR's copy instead.
     * Otherwise, use default copy behavior.
     */
    @Override
    public FIR copy(Nyes targetNyes) {
        if (isConstanic() && phaseBStarted) {
            // Unwrap CMFir - return inner FIR's copy
            FIR inner = o2;
            return inner.copy(targetNyes);
        } else if (isConstanic() && !phaseBStarted) {
            // Phase B not started yet, use o
            return o.copy(targetNyes);
        } else {
            // Not yet CONSTANIC, use default behavior
            return super.copy(targetNyes);
        }
    }
}
