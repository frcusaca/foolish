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
        // But we DO need to ordinate them so they can resolve identifiers through
        // this concatenation's parent chain during Stage A
        sourceFirs = new ArrayList<>();
        int index = 0;
        for (AST.Expr element : sourceElements) {
            FIR fir = createFiroeFromExpr(element);
            fir.setParentFir(this);
            // Ordinate FiroeWithBraneMind children so they can resolve identifiers
            // through this concatenation's parent chain
            if (fir instanceof FiroeWithBraneMind fwbm) {
                fwbm.ordinateToParentBraneMind(this, index);
            }
            sourceFirs.add(fir);
            index++;
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
     * Performs the join operation: flatten and clone all statements from source branes
     * into this concatenation's braneMemory.
     * <p>
     * Each source brane's statements are cloned individually and reset to INITIALIZED
     * (or kept as-is if CONSTANT). This flattening allows unanchored seeks (#-1, etc.)
     * to find statements from earlier branes in the concatenation.
     * <p>
     * All elements must resolve to FiroeWithBraneMind (branes or concatenations).
     * If any element resolves to a non-brane, an alarm is raised.
     */
    private void performJoin() {
        for (FIR fir : sourceFirs) {
            FIR resolved = unwrapToResolvedBrane(fir);

            if (resolved instanceof FiroeWithBraneMind fwbm) {
                // Flatten: iterate over the brane's statements and clone each one
                // into this concatenation's braneMemory
                fwbm.stream().forEach(statement -> {
                    // Clone each statement with this concatenation as new parent
                    // Reset to INITIALIZED if not CONSTANT so it can re-evaluate
                    Optional<Nyes> targetState = statement.isConstant()
                        ? Optional.empty()  // Keep CONSTANT as-is
                        : Optional.of(Nyes.INITIALIZED);  // Reset others to re-evaluate

                    FIR cloned = statement.cloneConstanic(this, targetState);

                    // Reset ordinated flag and memory position so storeFirs can
                    // re-ordinate in new context
                    if (cloned instanceof FiroeWithBraneMind clonedFwbm) {
                        clonedFwbm.ordinated = false;
                        clonedFwbm.resetMemoryPosition();
                    }
                    storeFirs(cloned);
                });
            } else {
                // Non-brane in concatenation is a critical error
                String errorMsg = "Concatenation element resolved to non-brane: " +
                    resolved.getClass().getSimpleName() +
                    " at " + fir.getLocationDescription() +
                    ". Concatenation can only contain branes.";
                AlarmSystem.raiseFromFir(this, errorMsg, AlarmSystem.PANIC);
                throw new IllegalStateException(formatErrorMessage(errorMsg));
            }
        }
        // Clean up stage A resources
        sourceFirs = null;
        stageAExecutor = null;
        joinComplete = true;
    }

    /**
     * Unwraps wrapper FIRs (Identifier, Search, CMFir, etc.) to get the resolved brane.
     * Follows the chain of wrappers until reaching the actual resolved FIR.
     *
     * @param fir the FIR to unwrap
     * @return the resolved FIR (should be a brane), or the original if not a wrapper
     */
    private FIR unwrapToResolvedBrane(FIR fir) {
        FIR current = fir;

        // Follow the wrapper chain
        while (current != null && current.isConstanic()) {
            switch (current) {
                case IdentifierFiroe idFir -> {
                    FIR resolved = idFir.getResolvedFir();
                    if (resolved == null) return current;
                    current = resolved;
                }
                case AbstractSearchFiroe searchFir -> {
                    FIR result = searchFir.getResult();
                    if (result == null) return current;
                    current = result;
                }
                case CMFir cmFir -> {
                    FIR result = cmFir.getResult();
                    if (result == null) return current;
                    current = result;
                }
                case AssignmentFiroe assignFir -> {
                    FIR result = assignFir.getResult();
                    if (result == null) return current;
                    current = result;
                }
                case UnanchoredSeekFiroe seekFir -> {
                    FIR result = seekFir.getResult();
                    if (result == null) return current;
                    current = result;
                }
                case FiroeWithBraneMind fwbm -> {
                    // Reached a brane - stop unwrapping
                    return current;
                }
                default -> {
                    // Not a wrapper, return as-is
                    return current;
                }
            }
        }

        return current;
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
