package org.foolish.fvm;

import org.foolish.ast.AST;

import java.util.List;

/**
 * Midoe wrapper for brane constructs.
 */
class BraneMidoe extends Midoe {
    private final List<Midoe> statements;

    BraneMidoe(Insoe base, List<Midoe> statements) {
        super(base);
        AST ast = base.ast();
        if (!(ast instanceof AST.Brane) && !(ast instanceof AST.Branes)) {
            throw new IllegalArgumentException("BraneMidoe requires AST.Brane or AST.Branes");
        }
        this.statements = List.copyOf(statements);
    }

    public List<Midoe> statements() {
        return statements;
    }

    public String toString() {
        StringBuffer statements = new StringBuffer();
        for (Midoe stmt : statements()) {
            statements.append(stmt).append(";");
        }
        return "MidoeBrane{" + statements + "}";
    }
}
