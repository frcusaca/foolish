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

        // If either operand is abstract (NK), the result is NK
        if (leftFir.isAbstract() || rightFir.isAbstract()) {
            result = new NKFiroe(ast, "Operand is not-known");
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

    @Override
    public boolean isNye() {
        // BinaryFiroe is NYE until we have computed the final result
        return result == null;
    }

    /**
     * Returns true if the result is NK (not-known).
     */
    @Override
    public boolean isAbstract() {
        if (result != null) {
            return result.isAbstract();
        }
        return super.isAbstract();
    }

    /**
     * Get the computed result value.
     */
    @Override
    public long getValue() {
        if (result == null) {
            throw new IllegalStateException("BinaryFiroe not fully evaluated");
        }
        return result.getValue();
    }
}
