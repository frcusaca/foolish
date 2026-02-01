package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * BinaryFiroe represents a binary expression in the UBC system.
 * The operator and AST for the operands are stored.
 * In one step, it converts to two Firoes, enqueued into braneMind.
 * They are stepped until not NYE (Not Yet Evaluated).
 * The last step performs the binary operation.
 *
 * Arithmetic errors (division by zero, etc.) result in NK (not-known) values.
 */
public class BinaryFiroe extends FiroeWithBraneMind {
    private final String operator;
    private FIR result;

    public BinaryFiroe(AST.BinaryExpr binaryExpr) {
        super(binaryExpr);
        this.operator = binaryExpr.op();
        this.result = null;
    }

    /**
     * Copy constructor for cloneConstanic.
     * Creates a copy with independent braneMind/braneMemory and updated parent chain.
     * Resets result to null so the copy can re-evaluate.
     *
     * @param original the BinaryFiroe to copy
     * @param newParent the new parent for this clone
     */
    protected BinaryFiroe(BinaryFiroe original, FIR newParent) {
        super(original, newParent);
        this.operator = original.operator;
        // Always reset result to null - the copy will re-evaluate
        this.result = null;
    }

    @Override
    protected void initialize() {
        AST.BinaryExpr binaryExpr = (AST.BinaryExpr) ast;
        storeExprs(binaryExpr.left(), binaryExpr.right());  // Store in braneMemory, not braneMind
        setInitialized();
    }

    @Override
    public int step() {
        if (result != null) {
            // Already computed - ensure we're in CONSTANT state
            setNyes(Nyes.CONSTANT);
            return 0;
        }

        switch (getNyes()) {
            case UNINITIALIZED, INITIALIZED, CHECKED, PRIMED -> {
                // Let parent handle state progression through these phases
                return super.step();
            }
            case EVALUATING -> {
                // Step operands through evaluation
                if (isBrainEmpty()) {
                    // All operands evaluated, compute result
                    computeResult();
                    return 1;
                }

                FIR current = brainDequeue();
                try {
                    int work = current.step();
                    if (current.isNye()) {
                        brainEnqueue(current);
                    }
                    return work;
                } catch (Exception e) {
                    brainEnqueueFirst(current); // Re-enqueue on error
                    throw new RuntimeException("Error during operand evaluation", e);
                }
            }
            case CONSTANT -> {
                // Should not reach here if result is null, but handle gracefully
                if (result == null && isBrainEmpty()) {
                    computeResult();
                    return 1;
                }
                return 0;
            }
        }
        return 0;
    }

    private void computeResult() {
        FIR leftFir = memoryGet(0);
        FIR rightFir = memoryGet(1);

        // If either operand is Constanic (unresolved), the result is Constanic.
        // We do NOT convert to NK. result stays null.
        // Use atConstanic() to check for exactly CONSTANIC state, not CONSTANT.
        if (leftFir.atConstanic() || rightFir.atConstanic()) {
            result = null; // Stay Constanic
            setNyes(Nyes.CONSTANIC);
            return;
        }

        try {
            long left = leftFir.getValue();
            long right = rightFir.getValue();

            // Handle division and modulo by zero -> NK
            if ((operator.equals("/") || operator.equals("%")) && right == 0) {
                String errorMsg = operator.equals("/") ? "Division by zero" : "Modulo by zero";
                result = new NKFiroe(ast, errorMsg);
                setNyes(Nyes.CONSTANT);
                return;
            }

            long resultValue = switch (operator) {
                case "+" -> left + right;
                case "-" -> left - right;
                case "*" -> left * right;
                case "/" -> left / right;  // Zero check done above
                case "%" -> left % right;  // Zero check done above
                case "==" -> left == right ? 1L : 0L;
                case "!=" -> left != right ? 1L : 0L;
                case "<>" -> left != right ? 1L : 0L;
                case "<" -> left < right ? 1L : 0L;
                case "<=" -> left <= right ? 1L : 0L;
                case ">" -> left > right ? 1L : 0L;
                case ">=" -> left >= right ? 1L : 0L;
                case "&&" -> (left != 0 && right != 0) ? 1L : 0L;
                case "||" -> (left != 0 || right != 0) ? 1L : 0L;
                default -> throw new UnsupportedOperationException("Unknown operator: " + operator);
            };
            result = new ValueFiroe(null, resultValue);
            setNyes(Nyes.CONSTANT);
        } catch (Exception e) {
            // Catch any runtime errors during evaluation
            result = new NKFiroe(ast, e.getMessage() != null ? e.getMessage() : e.getClass().getSimpleName());
            setNyes(Nyes.CONSTANT);
        }
    }

    // Removed isNye override to use parent's state-based logic

    /**
     * Returns true if the result is Constanic (e.g. missing vars).
     */
    @Override
    public boolean isConstanic() {
        if (result == null) {
            return true; // No result computed yet (or computed as null/Constanic)
        }
        return result.isConstanic();
    }

    /**
     * Get the computed result value.
     */
    @Override
    public long getValue() {
        if (result == null) {
            // If result is null, we are Constanic (unresolved).
            // Calling getValue() on an unresolved expression is an error.
            if (atConstanic()) {
                throw new IllegalStateException("BinaryFiroe is Constanic (unresolved)");
            }
            throw new IllegalStateException("BinaryFiroe not fully evaluated");
        }
        return result.getValue();
    }

    /**
     * Clones this CONSTANIC BinaryFiroe with updated parent chain.
     * Uses copy constructor to create independent braneMind/braneMemory with reset result.
     */
    @Override
    protected FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes) {
        if (!isConstanic()) {
            throw new IllegalStateException(
                formatErrorMessage("cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
                                  "but this FIR is in state: " + getNyes()));
        }

        if (isConstant()) {
            return this;  // Share CONSTANT binary expressions completely
        }

        // CONSTANIC: use copy constructor
        BinaryFiroe copy = new BinaryFiroe(this, newParent);

        // Set target state if specified, otherwise copy from original
        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }

        return copy;
    }
}
