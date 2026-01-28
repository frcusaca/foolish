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
public class CMFir extends FIR {
    protected FIR o;
    protected FIR o2;
    protected boolean phaseBStarted = false;

    public CMFir(AST ast, FIR o) {
        super(ast);
        this.o = o;
        // CMFir starts as NYE and will step through phases
        setNyes(Nyes.UNINITIALIZED);
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
        // make a stay_foolish_clone of o
        o2 = makeCopy(o);

        // Set o2's parent FIR to this CMFir
        o2.setParentFir(this);

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

    protected FIR makeCopy(FIR o) {
        return o.copy(Nyes.INITIALIZED);
    }

    @Override
    public FIR copy(Nyes newNyes) {
        if (isConstanic()) {
            return getResult().copy(newNyes); // Remove CMFir in copying if we can.
        } else {
            return super.copy(newNyes);
        }
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
}
