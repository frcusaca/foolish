package org.foolish.fvm;

import org.foolish.ast.AST;

/**
 * Firoe wrapper for an assignment expression.
 */
class AssignmentFiroe extends Firoe {
    private final Characterizable id;
    private final Firoe expr;

    AssignmentFiroe(Insoe base, Firoe expr) {
        this(base,
                new Characterizable(base.as(AST.Assignment.class).id()),
                expr);
    }

    AssignmentFiroe(Insoe base, Characterizable id, Firoe expr) {
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

    public Firoe expr() {
        return expr;
    }

    public String toString() {
        return FormatterFactory.verbose().format(this);
    }
}
