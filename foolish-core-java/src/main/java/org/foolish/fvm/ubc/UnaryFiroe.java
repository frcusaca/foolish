package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * UnaryFiroe represents a unary expression in the UBC system.
 * The operator and AST for the operand are stored.
 * In one step, it converts to the operand Firoe.
 * Evaluation continues until the operand is done.
 * One more step computes the result of the unary expression.
 *
 * Arithmetic errors during evaluation result in NK (not-known) values.
 */
public class UnaryFiroe extends FiroeWithBraneMind {
    private final String operator;
    private FIR operandFiroe;
    private FIR result;

    public UnaryFiroe(AST.UnaryExpr unaryExpr) {
        super(unaryExpr);
        this.operator = unaryExpr.op();
        this.result = null;
    }

    @Override
    protected void initialize() {
        setInitialized();
        AST.UnaryExpr op = (AST.UnaryExpr) ast;
        storeExprs(op.expr());
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
                // Step operand through evaluation
                if (braneMind.isEmpty()) {
                    // Operand evaluated, compute result
                    computeResult();
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
                    throw new RuntimeException("Error during operand evaluation", e);
                }
            }
            case CONSTANT -> {
                // Should not reach here if result is null, but handle gracefully
                if (result == null && braneMind.isEmpty()) {
                    computeResult();
                    return 1;
                }
                return 0;
            }
        }
        return 0;
    }

    private void computeResult() {
        if (braneMemory.isEmpty()) {
             // Should not happen if initialized
             return;
        }
        operandFiroe = braneMemory.get(0); // Use get(0) instead of removeFirst to keep history if needed? Or does it matter? Original code used removeFirst. Let's use get(0).

        // If operand is Constanic, the result is Constanic
        // Use atConstanic() to check for exactly CONSTANIC state, not CONSTANT.
        if (operandFiroe.atConstanic()) {
            result = null; // Stay Constanic
            setNyes(Nyes.CONSTANIC);
            return;
        }

        try {
            // Final step: Compute the unary operation result
            long operandValue = operandFiroe.getValue();
            long resultValue = switch (operator) {
                case "-" -> -operandValue;
                case "!" -> operandValue == 0 ? 1L : 0L;
                default -> throw new UnsupportedOperationException("Unknown operator: " + operator);
            };
            result = new ValueFiroe(ast, resultValue);
            setNyes(Nyes.CONSTANT);
        } catch (Exception e) {
            // Catch any runtime errors during evaluation
            result = new NKFiroe(ast, e.getMessage() != null ? e.getMessage() : e.getClass().getSimpleName());
            setNyes(Nyes.CONSTANT);
        }
    }

    @Override
    public boolean isConstanic() {
        if (result != null) {
            return result.isConstanic();
        }
        if (getNyes() == Nyes.CONSTANT) return true; // Constant state but no result -> constanic
        return super.isConstanic();
    }

    /**
     * Get the computed result value.
     */
    @Override
    public long getValue() {
        if (result == null) {
            if (atConstanic()) {
                throw new IllegalStateException("UnaryFiroe is Constanic (unresolved)");
            }
            throw new IllegalStateException("UnaryFiroe not fully evaluated");
        }
        return result.getValue();
    }

    @Override
    public String toString() {
        if (result != null) {
            return result.toString();
        }
        return operator + (operandFiroe != null ? operandFiroe : "?");
    }

}
