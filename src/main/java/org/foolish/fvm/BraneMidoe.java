package org.foolish.fvm;

import java.util.List;

/** Midoe wrapper for a {@link Brane}. */
class BraneMidoe extends Midoe {
    private final List<Midoe> statements;

    BraneMidoe(Brane base, List<Midoe> statements) {
        super(base);
        this.statements = List.copyOf(statements);
    }

    public List<Midoe> statements() {
        return statements;
    }
}
