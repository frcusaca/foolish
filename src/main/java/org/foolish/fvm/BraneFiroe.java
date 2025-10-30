package org.foolish.fvm;

import org.foolish.ast.AST;

import java.util.List;

/**
 * Firoe wrapper for brane constructs.
 */
class BraneFiroe extends Firoe {
    private final List<Firoe> statements;

    BraneFiroe(Insoe base, List<Firoe> statements) {
        super(base);
        if (base != null) {
            AST ast = base.ast();
            if (!(ast instanceof AST.Brane) && !(ast instanceof AST.Branes)) {
                throw new IllegalArgumentException("BraneFiroe requires AST.Brane or AST.Branes");
            }
        }
        this.statements = List.copyOf(statements);
    }

    public List<Firoe> statements() {
        return statements;
    }

    public String toString() {
        return FormatterFactory.verbose().format(this);
    }
}
