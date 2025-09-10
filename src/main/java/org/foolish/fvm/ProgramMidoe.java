package org.foolish.fvm;

import org.foolish.ast.AST;

/** Midoe wrapper for a program. */
class ProgramMidoe extends Midoe {
    private final Midoe brane;

    ProgramMidoe(Insoe base, Midoe brane) {
        super(base);
        base.as(AST.Program.class);
        this.brane = brane;
    }

    public Midoe brane() {
        return brane;
    }
}
