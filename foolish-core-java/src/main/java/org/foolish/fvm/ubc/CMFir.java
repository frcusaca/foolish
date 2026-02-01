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
            // CMFir delegates state to o via overridden getNyes(), atConstant(), atConstanic()
            return 1;
        } else {
            // Phase B: Step o2 (the clone in the new context)
            o2.step();
            syncO2Nyes();
            return 1;
        }
    }

    protected void syncO2Nyes(){
        // CMFir delegates state to inner FIR via overridden getNyes(), atConstant(), atConstanic()
    }

    protected void startPhaseB() {
        phaseBStarted = true;

        // Clone the CONSTANIC FIR with updated parent chain and reset to INITIALIZED
        o2 = o.cloneConstanic(this, java.util.Optional.of(Nyes.INITIALIZED));

        // If o2 is a BraneFiroe, recalculate its depth in the new context
        if (o2 instanceof BraneFiroe braneFiroe) {
            int newDepth = braneFiroe.calculateBraneDepth();
            braneFiroe.setExprmntBraneDepth(newDepth);
        }

        // If o2 has a braneMind, link its memory to the containing brane's memory
        if (o2 instanceof FiroeWithBraneMind fwbm) {
            BraneFiroe myBrane = getMyBrane();
            if (myBrane != null) {
                fwbm.braneMemory.setParent(myBrane.braneMemory);
                fwbm.braneMemory.setMyPosInternal(myBrane.braneMemory.size() - 1);
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
        if (phaseBStarted) {
            return o2.getNyes();
        } else {
            return o.getNyes();
        }
    }

    @Override
    public boolean atConstant() {
        if (phaseBStarted) {
            return o2.atConstant();
        } else {
            return o.atConstant();
        }
    }

    @Override
    public boolean atConstanic() {
        if (phaseBStarted) {
            return o2.atConstanic();
        } else {
            return o.atConstanic();
        }
    }

    @Override
    public boolean isNye() {
        if (!phaseBStarted) {
            return o.isNye() || o.atConstanic();
        } else {
            return o2.isNye();
        }
    }

    @Override
    public FIR copy(Nyes targetNyes) {
        if (isConstanic() && phaseBStarted) {
            return o2.copy(targetNyes);
        } else if (isConstanic() && !phaseBStarted) {
            return o.copy(targetNyes);
        } else {
            return super.copy(targetNyes);
        }
    }
}
