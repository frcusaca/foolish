package org.foolish.fvm;

/**
 * Basic binary arithmetic operations on long values.
 */
public class BinaryExpr implements Instruction {
    private final String op;
    private final Instruction left;
    private final Instruction right;

    public BinaryExpr(String op, Instruction left, Instruction right) {
        this.op = op;
        this.left = left;
        this.right = right;
    }

    @Override
    public Object execute(Environment env) {
        long l = ((Number) left.execute(env)).longValue();
        long r = ((Number) right.execute(env)).longValue();
        return switch (op) {
            case "+" -> l + r;
            case "-" -> l - r;
            case "*" -> l * r;
            case "/" -> l / r;
            default -> throw new IllegalArgumentException("Unknown op: " + op);
        };
    }
}
