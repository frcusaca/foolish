package org.foolish.fvm;

/**
 * Unary arithmetic operations on long values.
 */
public class UnaryExpr extends Instruction {
    private final String op;
    private final Targoe expr;

    public UnaryExpr(String op, Targoe expr) {
        super(TargoeType.UNARY_EXPR);
        this.op = op;
        this.expr = expr;
    }

    @Override
    public EvalResult execute(Environment env) {
        EvalResult er = expr.execute(env);
        if (er.value() == Unknown.INSTANCE) {
            return er;
        }
        long v = er.value().asLong();
        long result = switch (op) {
            case "+" -> +v;
            case "-" -> -v;
            case "*" -> v; // '*' unary no-op for now
            default -> throw new IllegalArgumentException("Unknown unary op: " + op);
        };
        return new EvalResult(new Resoe(result), er.env());
    }
}

