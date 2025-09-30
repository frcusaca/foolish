package org.foolish.fvm;

import org.foolish.ast.AST;

/** Midoe wrapper for a unary expression. */
class UnaryMidoe extends Midoe {
    private final String op;
    private final Midoe expr;

    UnaryMidoe(String op, Midoe expr) {
        super(null);
        this.op = op;
        this.expr = expr;
    }
    UnaryMidoe(Insoe base, Midoe expr) {
        super(base);
        AST.UnaryExpr ast = base.as(AST.UnaryExpr.class);
        this.op = ast.op();
        this.expr = expr;
    }

    public String op() { return op; }
    public Midoe expr() { return expr; }
    public String toString() { return "MidoeUnary(" + op + expr + ")"; }
}
