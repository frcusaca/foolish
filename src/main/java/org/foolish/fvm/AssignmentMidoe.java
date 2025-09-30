package org.foolish.fvm;

import org.foolish.ast.AST;

/**
 * Midoe wrapper for an assignment expression.
 */
class AssignmentMidoe extends Midoe {
    private final Characterizable id;
    private final Midoe expr;

    AssignmentMidoe(Insoe base, Midoe expr) {
        this(base,
                new Characterizable(base.as(AST.Assignment.class).id()),
                expr);
    }

    AssignmentMidoe(Insoe base, Characterizable id, Midoe expr) {
        super(base);
        if (base!=null) {
            AST.Assignment ast = base.as(AST.Assignment.class);
            assert ast.id().equals(id.id());
        }
        this.id = id;
        this.expr = expr;
    }

    public Characterizable id() {
        return id;
    }

    public Midoe expr() {
        return expr;
    }

    public String toString() {
        return "MidoeAssignment(" + id + "," + expr + ")";
    }
}
