package org.foolish.fvm;

/**
 * Unary arithmetic operations on long values.
 */
public class UnaryExpr implements Insoe {
    private final String op;
    private final Targoe expr;

    public UnaryExpr(String op, Targoe expr) {
        this.op = op;
        this.expr = expr;
    }

    @Override
    public Finer execute(Environment env) {
        Finer res = Midoe.evaluate(expr, env);
        if (res.isUnknown()) return Finer.UNKNOWN;
        long v = ((Number) res.value()).longValue();
        long val = switch (op) {
            case "+" -> +v;
            case "-" -> -v;
            case "*" -> v; // '*' unary no-op for now
            default -> throw new IllegalArgumentException("Unknown unary op: " + op);
        };
        return Finer.of(val);
    }
}
