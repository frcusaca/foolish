package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.apache.commons.lang3.tuple.Pair;

import java.util.ArrayList;
import java.util.LinkedList;
import java.util.List;
import java.util.Optional;
import java.util.stream.Stream;
import java.util.IdentityHashMap;

/**
 * FIR with a braneMind queue for managing evaluation tasks.
 * The braneMind enables breadth-first execution of nested expressions.
 * <p>
 * This class centrally manages the braneMind queue operations:
 * <ul>
 *   <li>Enqueuing sub-FIRs that need evaluation</li>
 *   <li>Stepping through NYE (Not Yet Evaluated) FIRs</li>
 *   <li>Dequeuing FIRs when they complete evaluation</li>
 * </ul>
 * <p>
 * Derived classes must implement initialize() and should override step() and isNye() for their specific logic.
 * <p>
 * <b>=== CONSTRAINTS ===</b>
 * <p>
 * <b>C5: PRIMED STATE SEPARATION</b><br>
 * When a FIR reaches isConstanic() (CONSTANIC or CONSTANT), braneMind MUST be empty.
 * This is critical for {@link #cloneConstanic(FIR, java.util.Optional)} which assumes empty braneMind.
 * The PRIMED state exists specifically to ensure this invariant.
 * <p>
 * <b>C6: BRANEMIND WORK QUEUE INVARIANT</b><br>
 * braneMind only contains NYE FIRs (FIRs with {@code isNye() == true}).
 * Once a FIR is {@code !isNye()}, it is removed from braneMind (stays in braneMemory).
 * <p>
 * <b>C7: BRANEMEMORY PERSISTENCE</b><br>
 * braneMemory is append-only during normal operation.
 * Items are never removed from braneMemory during evaluation.
 * <p>
 * <b>C8: ORDINATION REQUIREMENT</b><br>
 * Before braneMemory can resolve identifiers from parent context,
 * {@link #ordinateToParentBraneMind(FiroeWithBraneMind, int)} must be called exactly once.
 * The {@code ordinated} flag tracks this.
 * <p>
 * <b>=== FIELD ACCESS ===</b>
 * <p>
 * The braneMind and braneMemory fields are private.
 * Subclasses must use the accessor methods defined in the ACCESSOR METHODS section.
 * For external diagnostic access, use {@link #getBraneMemory()}.
 * <p>
 * Direct field access is only permitted within FiroeWithBraneMind itself.
 * This prevents one FIR from directly mutating another FIR's internal state.
 */
public abstract class FiroeWithBraneMind extends FIR {
    private final LinkedList<FIR> braneMind;
    private final BraneMemory braneMemory;
    protected boolean ordinated;
    protected IdentityHashMap indexLookup = new IdentityHashMap<FIR,Integer>();

    protected FiroeWithBraneMind(AST ast, String comment) {
        super(ast, comment);
        this.braneMind = new LinkedList<>();
        this.braneMemory = new BraneMemory(null);
        this.ordinated = false;
    }

    public void ordinateToParentBraneMind(FiroeWithBraneMind parent, int myPos) {
        assert !this.ordinated;
        linkMemoryParent(parent.braneMemory);
        // For BraneFiroe: position is tracked via parent FIR relationships (getMyBraneIndex)
        // For other FIRs (like IdentifierFiroe): we need to set myPos since they don't have owningBrane
        if (!(this instanceof BraneFiroe)) {
            setMemoryPosition(myPos);
        }
        this.ordinated = true;
    }

    protected FiroeWithBraneMind(AST ast) {
        this(ast, null);
    }

    /**
     * Copy constructor for cloneConstanic.
     * Creates an exact copy of braneMemory with cloned items (updated parent chains).
     * BraneMemory is verified to be empty for CONSTANIC FIRs.
     *
     * @param original the FiroeWithBraneMind to copy
     * @param newParent the new parent for this clone
     */
    protected FiroeWithBraneMind(FiroeWithBraneMind original, FIR newParent) {
        super(original.ast(), original.comment);
        setParentFir(newParent);

        // Verify braneMind is empty (critical invariant for CONSTANIC FIRs)
        if (!original.braneMind.isEmpty()) {
            throw new IllegalStateException(
                formatErrorMessage("cloneConstanic requires empty braneMind, but found " +
                                  original.braneMind.size() + " items. " +
                                  "This indicates the FIR is not truly CONSTANIC."));
        }

        // Create empty braneMind (already verified original is empty)
        this.braneMind = new LinkedList<>();

        // Create new braneMemory with null parent initially
        // The parent will be set by the caller (e.g., CMFir.startPhaseB) based on new context
        this.braneMemory = new BraneMemory(null);
        int index = 0;
        for (FIR fir : original.braneMemory) {
            // Clone each item, resetting to INITIALIZED so they will re-evaluate
            // This is critical: nested FIRs must also be reset, not just the top-level brane
            FIR clonedFir = fir.cloneConstanic(this, java.util.Optional.of(Nyes.INITIALIZED));
            this.braneMemory.put(clonedFir);

            // Add to indexLookup so getIndexOf works for cloned FIRs
            this.indexLookup.put(clonedFir, index);

            // Link cloned FIR's braneMemory to this brane's memory (ordination)
            // This is critical for identifier resolution in the new context
            if (clonedFir instanceof FiroeWithBraneMind fwbm) {
                // Reset ordinated flag so we can re-ordinate in new context
                fwbm.ordinated = false;
                fwbm.ordinateToParentBraneMind(this, index);
            }
            index++;
        }

        this.ordinated = original.ordinated;

        // Mark as initialized since braneMemory is already populated
        setInitialized();
    }

    static FiroeWithBraneMind ofExpr(AST.Expr... tasks) {
        return of(List.of(tasks).stream().map(FIR::createFiroeFromExpr).toArray(FIR[]::new));
    }

    static FiroeWithBraneMind of(FIR... tasks) {
        FiroeWithBraneMind result = new FiroeWithBraneMind((AST) null, (String) null) {
            @Override
            protected void initialize() {
                setInitialized();
            }
        };
        for (FIR task : tasks) {
            result.enqueueFirs(task);
        }
        return result;
    }

    /**
     * Stores subfir created from AST expressions into braneMemory (not braneMind).
     */
    protected void storeSubfirOfExprs(AST.Expr... tasks) {
        storeFirs(ofExpr(tasks));
    }

    /**
     * DEPRECATED: Use storeSubfirOfExprs() during initialize() instead.
     */
    @Deprecated
    protected void enqueueSubfirOfExprs(AST.Expr... tasks) {
        enqueueFirs(ofExpr(tasks));
    }

    /**
     * Initialize this FIR by setting up its state and enqueuing sub-FIRs.
     * This method should be called once during the first step().
     * Implementations should call setInitialized(true) when complete.
     */
    protected abstract void initialize();

    /**
     * Primes the braneMind by enqueueing non-constant items from braneMemory.
     * <p>
     * This is called during the CHECKED → PRIMED transition. By separating
     * the priming logic from initialization, we ensure CONSTANIC FIRs have
     * empty braneMind, which is critical for cloneConstanic.
     * <p>
     * When a CONSTANIC FIR is cloned and reset to INITIALIZED, it will:
     * 1. Skip initialize() (already done)
     * 2. Reach CHECKED state
     * 3. Call prime() to enqueue non-constant items
     * 4. Continue evaluation in EVALUATING state
     */
    protected void prime() {
        // Enqueue all non-constant items from braneMemory into braneMind
        for (FIR fir : braneMemory) {
            if (!fir.isConstanic()) {
                braneMind.add(fir);
            }
        }
    }

    /**
     * Enqueues a FIR into the braneMind if it's NYE (Not Yet Evaluated).
     * If the FIR is already evaluated, place it directly into braneMemory.
     */
    /**
     * Adds FIRs to braneMemory only (not braneMind).
     * Called during initialize() to store FIRs without enqueueing for evaluation.
     * The prime() method will later enqueue non-constant items into braneMind.
     */
    protected void storeFirs(FIR... firs) {
        for (FIR fir : firs) {
            braneMemory.put(fir);
            // Set parent FIR relationship
            fir.setParentFir(this);
            int index = braneMemory.size() - 1;
            indexLookup.put(fir, index);
            switch (fir) {
                case FiroeWithBraneMind fwbm:
                    fwbm.ordinateToParentBraneMind(this, index);
                    break;
                default:
                    // Non-braneMind FIRs don't need ordination
                    break;
            }
        }
    }

    /**
     * DEPRECATED: Use storeFirs() during initialize() instead.
     * This method adds to both braneMind and braneMemory, which breaks the
     * PRIMED state separation. Kept for backward compatibility.
     */
    @Deprecated
    protected void enqueueFirs(FIR... firs) {
        for (FIR fir : firs) {
            // Direct field access to bypass constanic check - this is deprecated anyway
            braneMind.addLast(fir);
            braneMemory.put(fir);
            // Set parent FIR relationship
            fir.setParentFir(this);
	    int index = braneMind.size() - 1;
            indexLookup.put(fir,index);
            switch (fir) {
                case FiroeWithBraneMind fwbm:
                    fwbm.ordinateToParentBraneMind(this, index);
                default:
                    ;
            }
        }
    }

    protected int getIndexOf(FIR f){
        Integer idx = (Integer) indexLookup.get(f);
        return idx != null ? idx : -1;
    }

    /**
     * Stores FIRs created from AST expressions into braneMemory (not braneMind).
     * Use during initialize() instead of enqueueExprs().
     */
    protected void storeExprs(AST.Expr... exprs) {
        for (AST.Expr expr : exprs)
            storeFirs(FIR.createFiroeFromExpr(expr));
    }

    /**
     * DEPRECATED: Use storeExprs() during initialize() instead.
     */
    @Deprecated
    protected void enqueueExprs(AST.Expr... exprs) {
        for (AST.Expr expr : exprs)
            enqueueFirs(FIR.createFiroeFromExpr(expr));
    }

    /**
     * Checks if a FIR is a brane (BraneFiroe).
     */
    protected boolean isBrane(FIR fir) {
        return fir instanceof BraneFiroe;
    }

    /**
     * Steps the next FIR in the braneMind queue with state-aware brane handling.
     * <p>
     * State transitions:
     * - UNINITIALIZED → INITIALIZED: Initialize this FIR
     * - INITIALIZED → CHECKED: Step non-branes only until all are CHECKED (type/reference checking)
     * - CHECKED → EVALUATING: Immediate transition when detected
     * - EVALUATING → CONSTANT: Step everything (including branes) until all complete
     * <p>
     * Derived classes should override this method and call super.step() as needed.
     *
     * @return 1 for meaningful work, 0 for empty transitions
     */
    public int step() {
        switch (getNyes()) {
            case UNINITIALIZED -> {
                // First step: initialize this FIR
                initialize();
                setNyes(Nyes.INITIALIZED);
                return 1;
            }
            case INITIALIZED -> {
                // Step non-brane expressions until all are CHECKED
                if (stepNonBranesUntilState(Nyes.CHECKED)) {
                    // All non-branes have reached CHECKED (type/reference checking complete)
                    setNyes(Nyes.CHECKED);
                }
                return 1; // Did work stepping sub-expressions
            }
            case CHECKED -> {
                // Prime braneMind with non-constant items from braneMemory
                prime();
                setNyes(Nyes.PRIMED);
                return 1;
            }
            case PRIMED -> {
                // Immediate transition to EVALUATING
                setNyes(Nyes.EVALUATING);
                return 1;
            }
            case EVALUATING -> {
                // Step everything including sub-branes
                if (isBraneEmpty()) {
                    // All expressions evaluated, check if any are CONSTANIC
                    boolean anyConstanic = false;
                    for (FIR fir : braneMemory) {
                        if (fir.atConstanic()) {
                            anyConstanic = true;
                            break;
                        }
                    }
                    setNyes(anyConstanic ? Nyes.CONSTANIC : Nyes.CONSTANT);
                    return 1;
                }
                
                FIR current = braneMind.removeFirst();
                try {
                    int work = current.step();

                    if (current.isNye()) {
                        braneMind.addLast(current);
                    }
                    return work;
                } catch (Exception e) {
                    braneMind.addFirst(current); // Re-enqueue on error
                    org.foolish.fvm.AlarmSystem.raise(braneMemory, "Error during braneMind step execution: " + e.getMessage(), org.foolish.fvm.AlarmSystem.PANIC);
                    throw new RuntimeException("Error during braneMind step execution", e);
                }
            }
            case CONSTANIC, CONSTANT -> {
                // Already evaluated, nothing to do
                return 0;
            }
        }
        return 0; // Should not reach here
    }

    /**
     * Steps non-brane FIRs until they reach the target state.
     * Branes are re-enqueued without stepping.
     *
     * @param targetState The state that non-branes should reach
     * @return true if all non-branes have reached the target state, false otherwise
     */
    private boolean stepNonBranesUntilState(Nyes targetState) {
        if (braneMind.isEmpty() || allNonBranesReachedState(targetState)) {
            return true;
        }

        FIR current = braneDequeue();

        //Reach here only when we have found the first non-brane sub-targetSetate member
        try {
            // Step non-brane expression
            current.step();

            // re-enqueue if still NYE
            if (current.isNye()) {
                braneMind.addLast(current);
            }
            // Check if it has reached the target state
            if (current.getNyes().ordinal() < targetState.ordinal()) {
                return false; // don't need to detect further, at least one non-brane is not at target state
            }

        } catch (Exception e) {
            braneMind.addFirst(current); // Re-enqueue on error
            org.foolish.fvm.AlarmSystem.raise(braneMemory, "Error during braneMind step execution: " + e.getMessage(), org.foolish.fvm.AlarmSystem.PANIC);
            throw new RuntimeException("Error during braneMind step execution", e);
        }

        // Check if all non-branes in the queue have reached target state
        return allNonBranesReachedState(targetState);
    }

    /**
     * Checks if all non-brane FIRs in the braneMind have reached at least the target state.
     * The furst sub-target-state non-brane member is shifted to the front of the queue.
     */
    private boolean allNonBranesReachedState(Nyes targetState) {
        if (isBraneEmpty()) {
            return true;
        }
        FIR current = branePeek();
        int seen = 1;
        // Let's skip branes and the sub expressions that has already reached desired state.
        while (isBrane(current) || (current.getNyes().ordinal() >= targetState.ordinal())) {
            // Re-enqueue brane without stepping - keep its place in line
            if (seen++ > braneSize()) {
                return true;
            }
            braneEnqueue(braneDequeue());
            current = branePeek();
        }
        return false;
    }

    public Stream<FIR> stream() {
        return braneMemory.stream();
    }

    /**
     * Gets the statement index/number of the given FIR within this brane's statement queue.
     * Returns -1 if the FIR is not found in this brane's queue.
     *
     * @param fir the FIR to find
     * @return the 0-based index, or -1 if not found
     */
    public int getStatementIndex(FIR fir) {
        return braneMemory.getStatementIndex(fir);
    }

    // ========== ACCESSOR METHODS ==========
    // Phase 6: Controlled access to braneMemory and braneMind

    /**
     * BraneMind (work queue) accessors
     */

    /**
     * Enqueues a FIR at the end of braneMind.
     * Note: C6 states braneMind should only contain NYE FIRs, but in practice
     * we may temporarily enqueue constanic FIRs during evaluation (they'll be
     * removed on the next iteration when isNye() returns false).
     */
    protected void braneEnqueue(FIR fir) {
        braneMind.addLast(fir);
    }

    /**
     * Enqueues a FIR at the beginning of braneMind (for error recovery).
     * Note: During error recovery, we may need to re-enqueue FIRs that are already constanic.
     * This is acceptable as an exception to C6 since we're in an error state.
     */
    protected void braneEnqueueFirst(FIR fir) {
        // Allow re-enqueueing for error recovery (exception to C6)
        braneMind.addFirst(fir);
    }

    /**
     * Dequeues and returns the first FIR from braneMind.
     * @throws java.util.NoSuchElementException if braneMind is empty
     */
    protected FIR braneDequeue() {
        return braneMind.removeFirst();
    }

    /**
     * Returns the first FIR in braneMind without removing it.
     * @throws java.util.NoSuchElementException if braneMind is empty
     */
    protected FIR branePeek() {
        return braneMind.getFirst();
    }

    /**
     * Checks if braneMind is empty.
     */
    protected boolean isBraneEmpty() {
        return braneMind.isEmpty();
    }

    /**
     * Returns the size of braneMind.
     */
    protected int braneSize() {
        return braneMind.size();
    }

    /**
     * BraneMemory (persistent storage) accessors
     */

    /**
     * Gets a FIR from braneMemory by index (read-only).
     * @param index the 0-based index
     * @return the FIR at the given index
     * @throws IndexOutOfBoundsException if index is out of range
     */
    protected FIR memoryGet(int index) {
        return braneMemory.get(index);
    }

    /**
     * Checks if braneMemory is empty (read-only).
     */
    protected boolean isMemoryEmpty() {
        return braneMemory.isEmpty();
    }

    /**
     * Gets the last FIR from braneMemory (read-only).
     * @throws java.util.NoSuchElementException if braneMemory is empty
     */
    protected FIR memoryGetLast() {
        return braneMemory.getLast();
    }

    /**
     * Returns the size of braneMemory (read-only).
     */
    protected int memorySize() {
        return braneMemory.size();
    }

    /**
     * Searches braneMemory for an identifier matching the query.
     * @param query the search query
     * @param fromLine the starting line for the search
     * @return pair of (index, FIR) if found, empty otherwise
     */
    protected Optional<Pair<Integer, FIR>> memoryGet(Query query, int fromLine) {
        return braneMemory.get(query, fromLine);
    }

    /**
     * Links this braneMemory to a parent BraneMemory (controlled mutation).
     * This is a legitimate parent chain modification (C4 exception).
     * Used during ordination and context manipulation (CMFir, ConcatenationFiroe).
     * <p>
     * Accepts ReadOnlyBraneMemory to allow use with getBraneMemory(), but internally
     * requires actual BraneMemory for the parent link.
     */
    protected void linkMemoryParent(ReadOnlyBraneMemory parent) {
        if (!(parent instanceof BraneMemory)) {
            throw new IllegalArgumentException("Parent must be an actual BraneMemory instance");
        }
        braneMemory.setParent((BraneMemory) parent);
    }

    /**
     * Sets the position of this braneMemory within its parent (controlled mutation).
     * Used during ordination and context manipulation.
     */
    protected void setMemoryPosition(int pos) {
        braneMemory.setMyPosInternal(pos);
    }

    /**
     * Resets the memory position to allow re-ordination in a new context.
     * Used when cloning FIRs for concatenation flattening.
     */
    protected void resetMemoryPosition() {
        braneMemory.resetMyPos();
    }

    /**
     * Sets the owning brane for this braneMemory (controlled mutation).
     * Used during construction and cloning operations.
     */
    protected void setMemoryOwner(FiroeWithBraneMind owner) {
        braneMemory.setOwningBrane(owner);
    }

    /**
     * Gets a read-only view of the BraneMemory.
     * Used by AlarmSystem and other diagnostic tools.
     * <p>
     * External code receives a {@link ReadOnlyBraneMemory} interface that only
     * exposes query and iteration operations, preventing modification of the
     * brane's internal memory structure.
     *
     * @return read-only view of braneMemory (never null)
     */
    public ReadOnlyBraneMemory getBraneMemory() {
        return braneMemory;
    }

    public FIR getMemoryItem(int index){
        return braneMemory.get(index);
    }

    // ========== PACKAGE-PRIVATE TEST ACCESSORS ==========
    // For unit tests only - allows inspection of internal state

    /**
     * Gets braneMind size (for testing/debugging only).
     */
    int getBraneMindSize() {
        return braneMind.size();
    }

    /**
     * Checks if braneMind is empty (for testing/debugging only).
     */
    boolean isBraneMindEmpty() {
        return braneMind.isEmpty();
    }

    /**
     * Peeks at first FIR in braneMind (for testing/debugging only).
     */
    FIR peekBraneMind() {
        return braneMind.isEmpty() ? null : braneMind.getFirst();
    }

    @Override
    protected FIR clone() {
        return (FiroeWithBraneMind) super.clone();
    }

    @Override
    protected FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes) {
        if (!isConstanic()) {
            throw new IllegalStateException(
                formatErrorMessage("cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
                                  "but this FIR is in state: " + getNyes()));
        }

        if (isConstant()) {
            return this;
        }

        FiroeWithBraneMind copy = new FiroeWithBraneMind(this, newParent) {
            @Override
            protected void initialize() {
                setInitialized();
            }
        };

        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }

        return copy;
    }
}
