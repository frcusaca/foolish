package org.foolish.fvm;

/**
 * Basic binary arithmetic operations on long values.
 */
public class BinaryExpr implements Insoe {
    private final String op;
    private final Targoe left;
    private final Targoe right;

    public BinaryExpr(String op, Targoe left, Targoe right) {
        this.op = op;
        this.left = left;
        this.right = right;
    }

    @Override
    public Finer execute(Environment env) {
        Finer l = Midoe.evaluate(left, env);
        Finer r = Midoe.evaluate(right, env);
        if (l.isUnknown() || r.isUnknown()) return Finer.UNKNOWN;
        long lv = ((Number) l.value()).longValue();
        long rv = ((Number) r.value()).longValue();
        long res = switch (op) {
            case "+" -> lv + rv;
            case "-" -> lv - rv;
            case "*" -> lv * rv;
            case "/" -> lv / rv;
            default -> throw new IllegalArgumentException("Unknown op: " + op);
        };
        return Finer.of(res);
    }
}
