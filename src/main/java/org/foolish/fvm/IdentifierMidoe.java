package org.foolish.fvm;

import org.foolish.ast.AST;

/**
 * Midoe wrapper for an identifier expression.
 */
class IdentifierMidoe extends Midoe {
    private final Characterizable id;

    IdentifierMidoe(String id) {
        super(null);
        this.id = new Characterizable(id);
    }
        IdentifierMidoe(Insoe base) {
        super(base);
        AST.Identifier ast = base.as(AST.Identifier.class);
        this.id = Characterizable.fromAst(ast);
    }

    public Characterizable id() {
        return id;
    }

    public String toString() {
        return FormatterFactory.verbose().format(this);
    }
}
