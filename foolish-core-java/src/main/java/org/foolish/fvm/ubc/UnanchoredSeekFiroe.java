package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * UnanchoredSeekFiroe implements unanchored backward seek in the current brane.
 * Syntax: #-1 (previous statement), #-2 (two statements back), etc.
 *
 * Unanchored seek searches backwards from the current position in the brane.
 * Only negative offsets are allowed (excluding 0).
 * The search is bound by the current brane and becomes CONSTANIC if the offset goes beyond the brane start.
 *
 * Examples:
 *   {a=1; b=2; c=#-1 + #-2}  // c = 2 + 1 = 3
 *   {x=10; y=#-5}            // y is CONSTANIC (out of bounds, rendered as ⎵⎵)
 *
 * IMPORTANT: Unanchored seeks with out-of-bounds offsets become CONSTANIC, not NK (???).
 * This is intentional - the seek awaits potential brane concatenation that could provide
 * the missing statements. When branes are concatenated, a previously out-of-bounds seek
 * may become resolved.
 *
 * Future Feature - Brane Concatenation:
 *   When branes are concatenated (e.g., {a=1}{b=#-1}), the unanchored seek #-1 in the second
 *   brane should find 'a' from the first brane. Currently, brane concatenation is not fully
 *   implemented, so out-of-bounds seeks remain CONSTANIC.
 *
 * TODO: When implementing brane concatenation, ensure:
 *   1. Unanchored seeks re-evaluate after concatenation
 *   2. CONSTANIC seeks transition to CHECKED when they find their target
 *   3. The search uses the concatenated brane's full memory, not just the original brane
 */
public class UnanchoredSeekFiroe extends FiroeWithBraneMind {

    private final int offset;
    private FIR value = null;

    public UnanchoredSeekFiroe(AST.UnanchoredSeekExpr seekExpr) {
        super(seekExpr);
        this.offset = seekExpr.offset();

        // Validate: only negative offsets allowed
        if (offset >= 0) {
            throw new IllegalArgumentException("Unanchored seek only allows negative offsets: #" + offset);
        }
    }

    /**
     * Copy constructor for cloneConstanic.
     */
    protected UnanchoredSeekFiroe(UnanchoredSeekFiroe original, FIR newParent) {
        super(original, newParent);
        this.offset = original.offset;
        this.value = null;  // Reset for re-evaluation
    }

    @Override
    protected void initialize() {
        setInitialized();
    }

    /**
     * An unanchored seek is Constanic if it hasn't been resolved yet or if its resolved value is Constanic.
     */
    @Override
    public boolean isConstanic() {
        if (value == null) {
            return true; // Not yet resolved or out of bounds
        }
        return value.isConstanic();
    }

    /**
     * Resolve the unanchored seek during the INITIALIZED phase.
     *
     * Unanchored seeks search in the containing brane's memory, not in the current
     * expression's memory. Since this FIR is typically ordinated to an AssignmentFiroe
     * (which has empty memory), we need to access the parent's memory (the actual brane).
     */
    @Override
    public int step() {
        switch (getNyes()) {
            case INITIALIZED -> {
                // UnanchoredSeek searches within the IMMEDIATE containing brane/concatenation.
                // It does NOT traverse up to parent branes - the search is bounded by the
                // brane or concatenation boundary.
                //
                // Example: { a=1; b={ c=#-1 } }
                //   Inside brane b, #-1 looks for the previous statement within b.
                //   Since c is the first statement in b, #-1 is CONSTANIC (out of bounds).
                //   It does NOT find a=1 from the outer brane.
                //
                // For concatenations like OB = OB1 OB2:
                //   After joining, all statements are flattened into the concatenation's memory.
                //   A seek in a cloned statement can find preceding statements from earlier branes.
                //
                // Algorithm:
                // 1. Find the closest FiroeWithBraneMind memory (brane or concatenation)
                // 2. Find our current position within that memory
                // 3. Seek backwards by offset within that memory only

                // Use getMyBraneContainer() to find BraneFiroe or ConcatenationFiroe
                FiroeWithBraneMind containingMind = getMyBraneContainer();
                int currentPos = getMyBraneContainerIndex();

                if (containingMind == null || currentPos < 0) {
                    // No containing brane/concatenation found - out of bounds
                    value = null;
                    setNyes(Nyes.CONSTANIC);
                    return 1;
                }

                ReadOnlyBraneMemory targetMemory = containingMind.getBraneMemory();
                int size = targetMemory.size();

                // Calculate target index: currentPos + offset (offset is negative)
                // Example: currentPos=2, offset=-1 -> targetIdx=1 (previous statement)
                int targetIdx = currentPos + offset;

                // Check bounds
                if (targetIdx >= 0 && targetIdx < size) {
                    value = targetMemory.get(targetIdx);
                    setNyes(Nyes.CHECKED);
                } else {
                    // Out of bounds - return constanic (not found)
                    value = null;
                    setNyes(Nyes.CONSTANIC);
                }
                return 1;
            }
            default -> {
                return super.step();
            }
        }
    }

    @Override
    public long getValue() {
        if (value == null) {
            if (atConstanic()) throw new IllegalStateException("Unanchored seek is Constanic (out of bounds): #" + offset);
            throw new IllegalStateException("Unanchored seek not resolved: #" + offset);
        }
        return value.getValue();
    }

    /**
     * Returns the resolved value for unwrapping in search operations.
     * This allows OneShotSearchFiroe and other search operations to access
     * the result of the unanchored seek.
     */
    public FIR getResult() {
        return value;
    }

    @Override
    public String toString() {
        return ((AST.UnanchoredSeekExpr) ast).toString();
    }

    @Override
    protected FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes) {
        if (!isConstanic()) {
            throw new IllegalStateException(
                formatErrorMessage("cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
                    "but this FIR is in state: " + getNyes()));
        }

        if (isConstant()) {
            return this;  // Share CONSTANT seeks
        }

        // CONSTANIC: use copy constructor
        UnanchoredSeekFiroe copy = new UnanchoredSeekFiroe(this, newParent);

        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }

        return copy;
    }
}
