package org.foolish.fvm;

/**
 * Executes {@link BinaryExpr} instructions.
 */
public final class BinaryExprVm {
    private BinaryExprVm() {}

    public static Finear execute(BinaryExpr expr, Environment env) {
        Finear l = expr.left().evaluate(env);
        Finear r = expr.right().evaluate(env);
        if (l.isUnknown() || r.isUnknown()) return Finear.UNKNOWN;
        long lv = ((Number) l.value()).longValue();
        long rv = ((Number) r.value()).longValue();
        long res = switch (expr.op()) {
            case "+" -> lv + rv;
            case "-" -> lv - rv;
            case "*" -> lv * rv;
            case "/" -> lv / rv;
            default -> throw new IllegalArgumentException("Unknown op: " + expr.op());
        };
        return Finear.of(res);
    }
}
