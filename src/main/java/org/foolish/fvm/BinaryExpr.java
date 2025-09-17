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
    public Targoe execute(Environment env) {
        Finer l = Evaluator.eval(left, env);
        Finer r = Evaluator.eval(right, env);
        if (l instanceof Unknown || r instanceof Unknown) {
            return Unknown.INSTANCE;
        }
        long lv = ((Number) l.value()).longValue();
        long rv = ((Number) r.value()).longValue();
        long result = switch (op) {
            case "+" -> lv + rv;
            case "-" -> lv - rv;
            case "*" -> lv * rv;
            case "/" -> lv / rv;
            default -> throw new IllegalArgumentException("Unknown op: " + op);
        };
        return new IntegerLiteral(result);
    }
}

