package org.foolish.ubc;

import org.foolish.ast.AST;

/**
 * BinaryFiroe represents a binary expression in the UBC system.
 * The operator and AST for the operands are stored.
 * In one step, it converts to two Firoes, enqueued into braneMind.
 * They are stepped until not underevaluated.
 * The last step performs the binary operation.
 */
public class BinaryFiroe extends FiroeWithBraneMind {
    private final String operator;
    private FIR leftFiroe;
    private FIR rightFiroe;
    private boolean operandsCreated;
    private ValueFiroe result;

    public BinaryFiroe(AST.BinaryExpr binaryExpr) {
        super(binaryExpr);
        this.operator = binaryExpr.op();
        this.operandsCreated = false;
        this.result = null;
    }

    @Override
    public void step() {
        if (result != null) {
            // Already computed
            return;
        }

        if (!operandsCreated) {
            // Step 1: Create left and right Firoes from AST
            AST.BinaryExpr binaryExpr = (AST.BinaryExpr) ast;
            leftFiroe = createFiroeFromExpr(binaryExpr.left());
            rightFiroe = createFiroeFromExpr(binaryExpr.right());
            operandsCreated = true;

            // Enqueue them if they need evaluation
            if (leftFiroe.underevaluated()) {
                braneMind.offer(leftFiroe);
            }
            if (rightFiroe.underevaluated()) {
                braneMind.offer(rightFiroe);
            }
            return;
        }

        // Step 2+: Process underevaluated operands
        if (!braneMind.isEmpty()) {
            FIR current = braneMind.peek();
            if (current instanceof FiroeWithBraneMind firoeWithMind) {
                firoeWithMind.step();
                if (!current.underevaluated()) {
                    braneMind.poll();
                }
            } else {
                braneMind.poll();
            }
            return;
        }

        // Final step: Both operands evaluated, compute the binary operation
        // Both operands should be evaluable to values now
        long left = leftFiroe.getValue();
        long right = rightFiroe.getValue();
        long resultValue = switch (operator) {
            case "+" -> left + right;
            case "-" -> left - right;
            case "*" -> left * right;
            case "/" -> right != 0 ? left / right : 0L;
            case "%" -> right != 0 ? left % right : 0L;
            case "==" -> left == right ? 1L : 0L;
            case "!=" -> left != right ? 1L : 0L;
            case "<" -> left < right ? 1L : 0L;
            case "<=" -> left <= right ? 1L : 0L;
            case ">" -> left > right ? 1L : 0L;
            case ">=" -> left >= right ? 1L : 0L;
            case "&&" -> (left != 0 && right != 0) ? 1L : 0L;
            case "||" -> (left != 0 || right != 0) ? 1L : 0L;
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
            throw new IllegalStateException("BinaryFiroe not fully evaluated");
        }
        return result.getValue();
    }

    @Override
    public boolean isAbstract() {
        return (leftFiroe != null && leftFiroe.isAbstract()) ||
               (rightFiroe != null && rightFiroe.isAbstract());
    }

    @Override
    public String toString() {
        if (result != null) {
            return result.toString();
        }
        return "(" + (leftFiroe != null ? leftFiroe : "?") + " " + operator + " " +
               (rightFiroe != null ? rightFiroe : "?") + ")";
    }

    /**
     * Creates a FIR from an AST expression.
     */
    private FIR createFiroeFromExpr(AST.Expr expr) {
        if (expr instanceof AST.IntegerLiteral literal) {
            return new ValueFiroe(expr, literal.value());
        } else if (expr instanceof AST.BinaryExpr binary) {
            return new BinaryFiroe(binary);
        } else if (expr instanceof AST.UnaryExpr unary) {
            return new UnaryFiroe(unary);
        } else if (expr instanceof AST.Brane brane) {
            return new BraneFiroe(brane);
        } else {
            // Placeholder for unsupported types
            return new ValueFiroe(0L);
        }
    }
}
