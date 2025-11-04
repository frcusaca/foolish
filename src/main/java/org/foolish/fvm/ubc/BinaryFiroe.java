package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * BinaryFiroe represents a binary expression in the UBC system.
 * The operator and AST for the operands are stored.
 * In one step, it converts to two Firoes, enqueued into braneMind.
 * They are stepped until not NYE (Not Yet Evaluated).
 * The last step performs the binary operation.
 */
public class BinaryFiroe extends FiroeWithBraneMind {
    private final String operator;
    private ValueFiroe result;

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
            // Already computed
            return;
        }

        if (!isInitialized()) {
            // Step 1: Create left and right Firoes from AST
            initialize();
            return;
        }

        // Step 2+: Let the parent class handle braneMind stepping
        super.step();

        // Check if we can compute the final result
        if (super.isNye()) {
            return;
        }

        long left = braneMemory.removeFirst().getValue();
        long right = braneMemory.removeFirst().getValue();
        long resultValue = switch (operator) {
            case "+" -> left + right;
            case "-" -> left - right;
            case "*" -> left * right;
            case "/" -> right != 0 ? left / right : 0L;
            case "%" -> right != 0 ? left % right : 0L;
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
    }

    @Override
    public boolean isNye() {
        return result == null;
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
