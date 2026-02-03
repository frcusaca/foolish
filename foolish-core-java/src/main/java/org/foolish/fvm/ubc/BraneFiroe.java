package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;

/**
 * BraneFiroe represents a brane in the UBC system.
 * The BraneFiroe processes its AST to create Expression Firoes,
 * which are enqueued into the braneMind and evaluated breadth-first.
 * <p>
 * <b>EXPERIMENTAL FEATURES:</b>
 * <ul>
 *   <li>EXPRMNT_brane_depth: Tracks nesting depth from root brane (0-based).
 *       Used to prevent infinite recursion and stack overflow.</li>
 * </ul>
 */
public class BraneFiroe extends FiroeWithBraneMind implements Constanicable {

    /**
     * EXPERIMENTAL: Maximum allowed brane depth before limiting instantiation.
     * Branes at or exceeding this depth are immediately set to CONSTANT with "???" value.
     * This prevents infinite recursion and excessive memory usage.
     * <p>
     * Value: 96,485 (chosen as a large but bounded limit)
     */
    private static final int EXPRMNT_MAX_BRANE_DEPTH = 96_485;

    /**
     * EXPERIMENTAL: The nesting depth of this brane from the root brane.
     * Root brane has depth 0, its child branes have depth 1, etc.
     * <p>
     * This is used to detect pathological cases of excessive nesting that could
     * lead to stack overflow or memory exhaustion.
     * <p>
     * Note: This is not final because it may be updated when a brane is copied
     * or coordinated (e.g., via CMFir wrapping or assignment coordination).
     */
    private int EXPRMNT_brane_depth;

    public BraneFiroe(AST ast) {
        super(ast);

        // Calculate and set depth from parent brane chain
        int calculatedDepth = calculateBraneDepth();
        setExprmntBraneDepth(calculatedDepth);
    }

    /**
     * Copy constructor for cloneConstanic.
     * Creates a copy with independent braneMind/braneMemory and updated parent chain.
     *
     * @param original the BraneFiroe to copy
     * @param newParent the new parent for this clone
     */
    protected BraneFiroe(BraneFiroe original, FIR newParent) {
        super(original, newParent);
        this.EXPRMNT_brane_depth = original.EXPRMNT_brane_depth;
        // Set owning brane for the new braneMemory (created by super)
        setMemoryOwner(this);
    }

    /**
     * EXPERIMENTAL: Sets the brane depth and checks if it exceeds the maximum allowed depth.
     * If the depth exceeds the limit, this brane is immediately set to CONSTANT and an alarm is raised.
     * <p>
     * This method should be called:
     * - During construction (automatically via calculateBraneDepth)
     * - When a brane is copied (via copy())
     * - When a brane is coordinated (e.g., CMFir wrapping, assignment coordination)
     *
     * @param depth the new depth to set
     */
    protected void setExprmntBraneDepth(int depth) {
        this.EXPRMNT_brane_depth = depth;

        // Check if depth exceeds limit
        if (this.EXPRMNT_brane_depth >= EXPRMNT_MAX_BRANE_DEPTH) {
            // Immediately set to CONSTANT - this brane will not evaluate
            setNyesConstant();

            // Raise medium level alarm
            org.foolish.fvm.AlarmSystem.raise(
                null,
                formatErrorMessage("Brane depth limit exceeded: " + this.EXPRMNT_brane_depth +
                                  " >= " + EXPRMNT_MAX_BRANE_DEPTH +
                                  ". This brane has been terminated with ??? value to prevent " +
                                  "infinite recursion or memory exhaustion."),
                org.foolish.fvm.AlarmSystem.MILD  // Medium severity level
            );

            // Do not register this brane - early return to prevent initialization
            return;
        }

        // Normal case: register this brane as the owner of its memory
        setMemoryOwner(this);
    }

    /**
     * EXPERIMENTAL: Calculates the depth of this brane by walking up the parent chain.
     * Root brane has depth 0.
     * <p>
     * Package-private so that CMFir can recalculate depth when coordinating branes.
     *
     * @return the nesting depth from root (0-based)
     */
    int calculateBraneDepth() {
        int depth = 0;
        FIR current = this.getParentFir();

        while (current != null) {
            if (current instanceof BraneFiroe) {
                depth++;
            }
            current = current.getParentFir();
        }

        return depth;
    }

    /**
     * EXPERIMENTAL: Returns the brane depth of this BraneFiroe.
     * Root brane returns 0, nested branes return their nesting level.
     *
     * @return the nesting depth (0-based)
     */
    public int getExprmntBraneDepth() {
        return EXPRMNT_brane_depth;
    }

    /**
     * Override clone to recalculate and update depth when a brane is copied.
     * This is crucial for CMFir coordination where a brane from one context
     * is copied into another context with potentially different nesting depth.
     */
    @Override
    protected FIR clone() {
        BraneFiroe cloned = (BraneFiroe) super.clone();

        // Recalculate depth based on new parent context
        // The clone will have the same parentFir initially, but when it's
        // re-parented (e.g., in CMFir.startPhaseB), the depth needs updating
        int newDepth = cloned.calculateBraneDepth();
        cloned.setExprmntBraneDepth(newDepth);

        return cloned;
    }

    /**
     * For container types, getResult() returns this since the brane IS the result.
     */
    @Override
    public FIR getResult() {
        return this;
    }

    /**
     * Clones this CONSTANIC BraneFiroe with updated parent chain.
     * Uses copy constructor to create independent braneMind/braneMemory.
     */
    @Override
    protected FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes) {
        if (!isConstanic()) {
            throw new IllegalStateException(
                formatErrorMessage("cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
                                  "but this FIR is in state: " + getNyes()));
        }

        if (isConstant()) {
            return this;  // Share CONSTANT branes completely
        }

        // CONSTANIC: use copy constructor
        BraneFiroe copy = new BraneFiroe(this, newParent);

        // Set target state if specified, otherwise copy from original
        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }

        return copy;
    }

    /**
     * Initialize the BraneFiroe by converting AST statements to Expression Firoes.
     */
    @Override
    protected void initialize() {
        if (isInitialized()) return;
        setInitialized();

        if (ast instanceof AST.Brane brane) {
            for (AST.Expr expr : brane.statements()) {
                FIR firoe = createFiroeFromExpr(expr);
                storeFirs(firoe);  // Store in braneMemory only, prime() will enqueue to braneMind
            }
        }else{
            throw new IllegalArgumentException("AST must be of type AST.Brane");
        }
    }

    // Removed isNye override

    @Override
    public int step() {
        if (!isInitialized()) {
            initialize();
            return 1;
        }

        return super.step();
    }

    /**
     * Returns the list of expression Firoes in this brane.
     * Includes both completed (in braneMemory) and pending (in braneMind) FIRs.
     */
    public List<FIR> getExpressionFiroes() {
        List<FIR> allFiroes = new ArrayList<>();
        stream().forEach(allFiroes::add);
        return allFiroes;
    }

    /**
     * Returns the value of this brane.
     * For depth-limited branes (exceeding EXPRMNT_MAX_BRANE_DEPTH), returns
     * a special error value indicating the depth limit was reached.
     */
    @Override
    public long getValue() {
        if (EXPRMNT_brane_depth >= EXPRMNT_MAX_BRANE_DEPTH) {
            throw new IllegalStateException(
                formatErrorMessage("Cannot get value from depth-limited brane (depth=" +
                                  EXPRMNT_brane_depth + " >= " + EXPRMNT_MAX_BRANE_DEPTH + ")")
            );
        }
        return super.getValue();
    }

    @Override
    public String toString() {
        if (EXPRMNT_brane_depth >= EXPRMNT_MAX_BRANE_DEPTH) {
            return "BraneFiroe[depth-limited@" + EXPRMNT_brane_depth + "]";
        }
        return new Sequencer4Human().sequence(this);
    }
}
