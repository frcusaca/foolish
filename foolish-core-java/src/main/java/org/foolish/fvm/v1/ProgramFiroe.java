package org.foolish.fvm.v1;

import org.foolish.ast.AST;

/**
 * Firoe wrapper for a program.
 */
class ProgramFiroe extends Firoe {
    private final Firoe brane;

    ProgramFiroe(Insoe base, Firoe brane) {
        super(base);
        base.as(AST.Program.class);
        this.brane = brane;
    }

    public Firoe brane() {
        return brane;
    }

    public String toString() {
        return FormatterFactory.verbose().format(this);
    }
}
