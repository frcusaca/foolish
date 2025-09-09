package org.foolish.fvm;

import java.util.List;

/**
 * Implements an if/elif/else expression using depth first evaluation.
 */
public class IfExpr extends Instruction {
    private final Targoe condition;
    private final Targoe thenExpr;
    private final Targoe elseExpr;
    private final List<IfExpr> elseIfs;

    public IfExpr(Targoe condition, Targoe thenExpr, Targoe elseExpr, List<IfExpr> elseIfs) {
        super(TargoeType.IF_EXPR);
        this.condition = condition;
        this.thenExpr = thenExpr;
        this.elseExpr = elseExpr;
        this.elseIfs = elseIfs == null ? List.of() : List.copyOf(elseIfs);
    }

    private boolean asBoolean(Resoe r) {
        Object o = r.value();
        if (o instanceof Boolean b) return b;
        if (o instanceof Number n) return n.longValue() != 0;
        return o != null;
    }

    @Override
    public EvalResult execute(Environment env) {
        EvalResult cond = condition.execute(env);
        if (cond.value() != Unknown.INSTANCE && asBoolean(cond.value())) {
            return thenExpr.execute(cond.env());
        }
        Environment current = cond.env();
        for (IfExpr elif : elseIfs) {
            EvalResult ec = elif.condition.execute(current);
            if (ec.value() != Unknown.INSTANCE && asBoolean(ec.value())) {
                return elif.thenExpr.execute(ec.env());
            }
            current = ec.env();
        }
        if (elseExpr != null) {
            return elseExpr.execute(current);
        }
        return new EvalResult(Resoe.UNKNOWN, current);
    }
}

