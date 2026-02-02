package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.fvm.AlarmSystem;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * ConcatenationFiroe combines multiple branes/expressions into a single evaluation context.
 * <p>
 * Semantics:
 * - Each element is first evaluated to PRIMED in its original context
 * - Then all elements are cloned and re-parented into this concatenation's braneMemory
 * - Later elements can resolve identifiers from earlier elements
 * <p>
 * Stage A: UNINITIALIZED → INITIALIZED → CHECKED
 *   - Create FIRs from source elements
 *   - Use ExecutionFir to step all FIRs to PRIMED state (breadth-first)
 *   - FIRs retain their original parents during this stage
 *   - LHS searches are blocked during this stage
 * <p>
 * Stage B: CHECKED → PRIMED
 *   - Clone and re-parent all source FIRs into braneMemory
 *   - Brane becomes LhsSearchable at end of this stage
 * <p>
 * Stage C: PRIMED → EVALUATING → CONSTANT
 *   - Normal evaluation with shared memory context
 *
 * @see projects/009-Concatenation_Project.md for detailed design specification
 */
public class ConcatenationFiroe extends FiroeWithBraneMind {

    private final List<AST.Expr> sourceElements;  // Original element ASTs
    private ExecutionFir stageAExecutor;          // Coordinates stepping to PRIMED
    private List<FIR> sourceFirs;                 // FIRs created from source elements
    private boolean joinComplete = false;

    public ConcatenationFiroe(AST.Concatenation concatenation) {
        super(concatenation);
        this.sourceElements = concatenation.elements();
    }

    /**
     * Copy constructor for cloneConstanic.
     */
    protected ConcatenationFiroe(ConcatenationFiroe original, FIR newParent) {
        super(original, newParent);
        this.sourceElements = original.sourceElements;
        this.stageAExecutor = null;  // Not needed for cloned state
        this.sourceFirs = null;       // Not needed for cloned state
        this.joinComplete = true;     // Cloned concatenation is already joined
        setMemoryOwner(this);
    }

    /**
     * Checks if this concatenation is ready for LHS identifier searches.
     * A concatenation becomes searchable when it reaches PRIMED state,
     * meaning all component branes have been cloned and stitched together.
     */
    public boolean isLhsSearchable() {
        return getNyes().ordinal() >= Nyes.PRIMED.ordinal();
    }

    @Override
    protected void initialize() {
        if (isInitialized()) return;
        setInitialized();

        // Create FIRs from source elements
        // NOT in braneMemory yet - we'll move them after they reach PRIMED
        sourceFirs = new ArrayList<>();
        for (AST.Expr element : sourceElements) {
            FIR fir = createFiroeFromExpr(element);
            fir.setParentFir(this);
            sourceFirs.add(fir);
        }

        // Create ExecutionFir to coordinate stepping to CONSTANIC
        // setParent(false) - FIRs keep their original parent relationships
        // We need CONSTANIC (not PRIMED) because cloneConstanic requires isConstanic()
        stageAExecutor = ExecutionFir.stepping(sourceFirs)
            .setParent(false)
            .stepUntil(Nyes.CONSTANIC)
            .build();
    }

    @Override
    public int step() {
        switch (getNyes()) {
            case UNINITIALIZED -> {
                initialize();
                setNyes(Nyes.INITIALIZED);
                return 1;
            }
            case INITIALIZED -> {
                // Stage A: Use ExecutionFir to step source FIRs to PRIMED (breadth-first)
                if (!stageAExecutor.isConstanic()) {
                    stageAExecutor.step();
                    return 1;
                }

                // ExecutionFir finished - check result
                if (stageAExecutor.isStuck()) {
                    // Some FIRs couldn't reach PRIMED - we become CONSTANIC
                    setNyes(Nyes.CONSTANIC);
                    return 1;
                }

                // All FIRs reached PRIMED - proceed to Stage B
                setNyes(Nyes.CHECKED);
                return 1;
            }
            case CHECKED -> {
                // Stage B: Clone and re-parent all source FIRs into braneMemory
                performJoin();
                prime();
                setNyes(Nyes.PRIMED);
                return 1;
            }
            case PRIMED -> {
                setNyes(Nyes.EVALUATING);
                return 1;
            }
            case EVALUATING -> {
                // Stage C: Normal evaluation
                return super.step();
            }
            case CONSTANIC, CONSTANT -> {
                return 0;
            }
        }
        return 0;
    }

    /**
     * Performs the join operation: clone all source FIRs and add to braneMemory.
     * After this, later elements can see identifiers from earlier elements.
     */
    private void performJoin() {
        for (FIR fir : sourceFirs) {
            FIR resolved = unwrapIfIdentifier(fir);

            if (resolved instanceof FiroeWithBraneMind fwbm) {
                // Clone the brane with this as new parent, reset to INITIALIZED
                FIR cloned = fwbm.cloneConstanic(this, Optional.of(Nyes.INITIALIZED));
                storeFirs(cloned);
            } else {
                // For simple values, just store them directly
                storeFirs(resolved);
            }
        }
        // Clean up stage A resources
        sourceFirs = null;
        stageAExecutor = null;
        joinComplete = true;
    }

    /**
     * If fir is an IdentifierFiroe that resolved to a brane, unwrap it.
     * Otherwise return the fir as-is.
     */
    private FIR unwrapIfIdentifier(FIR fir) {
        if (fir instanceof IdentifierFiroe idFir && idFir.isConstanic()) {
            FIR resolved = idFir.getResolvedFir();
            if (resolved != null) {
                return resolved;
            }
        }
        return fir;
    }

    @Override
    protected FIR cloneConstanic(FIR newParent, Optional<Nyes> targetNyes) {
        if (!isConstanic()) {
            throw new IllegalStateException(
                formatErrorMessage("cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
                    "but this FIR is in state: " + getNyes()));
        }

        if (isConstant()) {
            return this;  // Share CONSTANT concatenations
        }

        ConcatenationFiroe copy = new ConcatenationFiroe(this, newParent);

        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }

        return copy;
    }

    /**
     * Returns the value of this concatenation.
     * For concatenations containing branes, getValue() is not meaningful -
     * use sequencing instead. This throws UnsupportedOperationException.
     */
    @Override
    public long getValue() {
        if (!joinComplete || isMemoryEmpty()) {
            throw new IllegalStateException(
                formatErrorMessage("Cannot get value from incomplete or empty concatenation"));
        }
        FIR last = memoryGetLast();
        // Branes don't support getValue, they need to be sequenced
        if (last instanceof BraneFiroe || last instanceof ConcatenationFiroe) {
            throw new UnsupportedOperationException("getValue not supported for " + getClass().getSimpleName() + " containing branes");
        }
        return last.getValue();
    }

    @Override
    public String toString() {
        if (!joinComplete) {
            int pendingCount = sourceFirs != null ? sourceFirs.size() : 0;
            return "ConcatenationFiroe[pending=" + pendingCount + ", state=" + getNyes() + "]";
        }
        return new Sequencer4Human().sequence(this);
    }
}
