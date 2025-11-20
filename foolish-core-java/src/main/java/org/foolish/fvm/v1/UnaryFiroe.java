package org.foolish.fvm.v1;

import org.foolish.ast.AST;

/**
 * Firoe wrapper for a unary expression.
 */
class UnaryFiroe extends Firoe {
    private final String op;
    private final Firoe expr;

    UnaryFiroe(String op, Firoe expr) {
        super(null);
        this.op = op;
        this.expr = expr;
    }

    UnaryFiroe(Insoe base, Firoe expr) {
        super(base);
        AST.UnaryExpr ast = base.as(AST.UnaryExpr.class);
        this.op = ast.op();
        this.expr = expr;
    }

    public String op() {
        return op;
    }

    public Firoe expr() {
        return expr;
    }

    public String toString() {
        return FormatterFactory.verbose().format(this);
    }
}
