package org.foolish.fvm;

/** Midoe wrapper for a {@link Program}. */
class ProgramMidoe extends Midoe {
    private final Midoe brane;

    ProgramMidoe(Program base, Midoe brane) {
        super(base);
        this.brane = brane;
    }

    public Midoe brane() {
        return brane;
    }
}
