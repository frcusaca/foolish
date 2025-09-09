package org.foolish.fvm;

import java.util.List;

/**
 * A single brane containing a list of statements.
 */
public class SingleBrane extends Brane {
    private final List<Midoe> statements;

    public SingleBrane(Characterizable characterization, List<Midoe> statements) {
        super(characterization);
        this.statements = List.copyOf(statements);
    }

    @Override
    protected List<Midoe> statements() {
        return statements;
    }
}
