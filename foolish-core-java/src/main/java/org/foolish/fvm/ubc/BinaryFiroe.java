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

    @Override
    protected void initialize() {
        AST.BinaryExpr binaryExpr = (AST.BinaryExpr) ast;
        enqueueExprs(binaryExpr.left(), binaryExpr.right());
        setInitialized();
    }

    @Override
    public void step() {
        if (result != null) {
            // Already computed - ensure we're in CONSTANT state
            setNyes(Nyes.CONSTANT);
            return;
        }

        switch (getNyes()) {
            case UNINITIALIZED, INITIALIZED, REFERENCES_IDENTIFIED, ALLOCATED, RESOLVED -> {
                // Let parent handle state progression through these phases
                super.step();
            }
            case EVALUATING -> {
                // Step operands through evaluation
                super.step();

                // After parent steps, check if we can compute the final result
                // If parent says CONSTANT, it means braneMind is empty.
                if (getNyes() == Nyes.CONSTANT && braneMind.isEmpty() && result == null) {
                    computeResult();
                }
            }
            case CONSTANT -> {
                // Should not reach here if result is null, but handle gracefully
                if (result == null && braneMind.isEmpty()) {
                    computeResult();
                }
            }
        }
    }

    private void computeResult() {
        FIR leftFir = braneMemory.get(0);
        FIR rightFir = braneMemory.get(1);

        // If either operand is Constantic (unresolved), the result is Constantic.
        // We do NOT convert to NK. result stays null.
        if (leftFir.isConstantic() || rightFir.isConstantic()) {
            result = null; // Stay Constantic
            setNyes(Nyes.CONSTANT);
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
     * Returns true if the result is Constantic (e.g. missing vars).
     */
    @Override
    public boolean isConstantic() {
        if (result == null) {
            return true; // No result computed yet (or computed as null/Constantic)
        }
        return result.isConstantic();
    }

    /**
     * Get the computed result value.
     */
    @Override
    public long getValue() {
        if (result == null) {
            // If result is null but state is CONSTANT, it means we are Constantic (unresolved).
            // Calling getValue() on an unresolved expression is an error.
            if (getNyes() == Nyes.CONSTANT) {
                throw new IllegalStateException("BinaryFiroe evaluated to Constantic (unresolved)");
            }
            throw new IllegalStateException("BinaryFiroe not fully evaluated");
        }
        return result.getValue();
    }
}
