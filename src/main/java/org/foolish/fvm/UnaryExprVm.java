package org.foolish.fvm;

/**
 * Executes {@link UnaryExpr} instructions.
 */
public final class UnaryExprVm {
    private UnaryExprVm() {}

    public static Finear execute(UnaryExpr expr, Environment env) {
        Finear res = expr.expr().evaluate(env);
        if (res.isUnknown()) return Finear.UNKNOWN;
        long v = ((Number) res.value()).longValue();
        long val = switch (expr.op()) {
            case "+" -> +v;
            case "-" -> -v;
            case "*" -> v; // '*' unary no-op for now
            default -> throw new IllegalArgumentException("Unknown unary op: " + expr.op());
        };
        return Finear.of(val);
    }
}
