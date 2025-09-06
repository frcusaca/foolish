package org.foolish.fvm;

/**
 * Unary arithmetic operations on long values.
 */
public class UnaryExpr implements Instruction {
    private final String op;
    private final Instruction expr;

    public UnaryExpr(String op, Instruction expr) {
        this.op = op;
        this.expr = expr;
    }

    @Override
    public Object execute(Environment env) {
        long v = ((Number) expr.execute(env)).longValue();
        return switch (op) {
            case "+" -> +v;
            case "-" -> -v;
            case "*" -> v; // '*' unary no-op for now
            default -> throw new IllegalArgumentException("Unknown unary op: " + op);
        };
    }
}
