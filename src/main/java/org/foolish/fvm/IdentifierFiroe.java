package org.foolish.fvm;

import org.foolish.ast.AST;

/**
 * Firoe wrapper for an identifier expression.
 */
class IdentifierFiroe extends Firoe {
    private final Characterizable id;

    IdentifierFiroe(String id) {
        super(null);
        this.id = new Characterizable(id);
    }
        IdentifierFiroe(Insoe base) {
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
