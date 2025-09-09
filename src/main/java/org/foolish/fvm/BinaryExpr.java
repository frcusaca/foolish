package org.foolish.fvm;

/**
 * Basic binary arithmetic operations on long values.
 */
public class BinaryExpr extends Instruction {
    private final String op;
    private final Targoe left;
    private final Targoe right;

    public BinaryExpr(String op, Targoe left, Targoe right) {
        super(TargoeType.BINARY_EXPR);
        this.op = op;
        this.left = left;
        this.right = right;
    }

    @Override
    public EvalResult execute(Environment env) {
        EvalResult l = left.execute(env);
        EvalResult r = right.execute(l.env());
        if (l.value() == Unknown.INSTANCE || r.value() == Unknown.INSTANCE) {
            return new EvalResult(Unknown.INSTANCE, r.env());
        }
        long lv = l.value().asLong();
        long rv = r.value().asLong();
        if ("/".equals(op) && rv == 0) {
            return new EvalResult(Unknown.INSTANCE, r.env());
        }
        long result = switch (op) {
            case "+" -> lv + rv;
            case "-" -> lv - rv;
            case "*" -> lv * rv;
            case "/" -> lv / rv;
            default -> throw new IllegalArgumentException("Unknown op: " + op);
        };
        return new EvalResult(new Resoe(result), r.env());
    }
}

