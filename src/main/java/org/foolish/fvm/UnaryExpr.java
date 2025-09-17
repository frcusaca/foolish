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
    public Targoe execute(Environment env) {
        Finer f = Evaluator.eval(expr, env);
        if (f instanceof Unknown) {
            return Unknown.INSTANCE;
        }
        long v = ((Number) f.value()).longValue();
        long result = switch (op) {
            case "+" -> +v;
            case "-" -> -v;
            case "*" -> v; // '*' unary no-op for now
            default -> throw new IllegalArgumentException("Unknown unary op: " + op);
        };
        return new IntegerLiteral(result);
    }
}

