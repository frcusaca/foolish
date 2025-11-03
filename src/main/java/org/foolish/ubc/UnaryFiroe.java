package org.foolish.ubc;

import org.foolish.ast.AST;

/**
 * UnaryFiroe represents a unary expression in the UBC system.
 * The operator and AST for the operand are stored.
 * In one step, it converts to the operand Firoe.
 * Evaluation continues until the operand is done.
 * One more step computes the result of the unary expression.
 */
public class UnaryFiroe extends FiroeWithBraneMind {
    private final String operator;
    private FIR operandFiroe;
    private boolean operandCreated;
    private ValueFiroe result;

    public UnaryFiroe(AST.UnaryExpr unaryExpr) {
        super(unaryExpr);
        this.operator = unaryExpr.op();
        this.operandCreated = false;
        this.result = null;
    }

    @Override
    public void step() {
        if (result != null) {
            // Already computed
            return;
        }

        if (!operandCreated) {
            // Step 1: Create the operand Firoe from AST
            AST.UnaryExpr unaryExpr = (AST.UnaryExpr) ast;
            operandFiroe = createFiroeFromExpr(unaryExpr.expr());
            operandCreated = true;

            enqueueFir(operandFiroe);
            return;
        }

        // Step 2+: Let the parent class handle braneMind stepping
        super.step();
        
        // Check if we can compute the final result
        if (super.underevaluated()) {
            return;
        }

        // Final step: Compute the unary operation result
        long operandValue = operandFiroe.getValue();
        long resultValue = switch (operator) {
            case "-" -> -operandValue;
            case "!" -> operandValue == 0 ? 1L : 0L;
            default -> throw new UnsupportedOperationException("Unknown operator: " + operator);
        };
        result = new ValueFiroe(ast, resultValue);
    }

    @Override
    public boolean underevaluated() {
        return result == null;
    }

    /**
     * Get the computed result value.
     */
    @Override
    public long getValue() {
        if (result == null) {
            throw new IllegalStateException("UnaryFiroe not fully evaluated");
        }
        return result.getValue();
    }

    @Override
    public boolean isAbstract() {
        return operandFiroe != null && operandFiroe.isAbstract();
    }

    @Override
    public String toString() {
        if (result != null) {
            return result.toString();
        }
        return operator + (operandFiroe != null ? operandFiroe : "?");
    }

}
