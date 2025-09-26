package org.foolish.fvm;

import org.foolish.ast.AST;

/** Midoe wrapper for an assignment expression. */
class AssignmentMidoe extends Midoe {
    private final Characterizable id;
    private final Midoe expr;

    AssignmentMidoe(Insoe base, Midoe expr) {
        super(base);
        AST.Assignment ast = base.as(AST.Assignment.class);
        this.id = new Characterizable(ast.id());
        this.expr = expr;
    }

    public Characterizable id() { return id; }
    public Midoe expr() { return expr; }
    public String toString() { return "MidoeAssignment(" + id + "," + expr + ")"; }
}
