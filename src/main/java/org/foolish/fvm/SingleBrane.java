package org.foolish.fvm;

import java.util.List;

/**
 * A single brane containing a list of statements.
 */
public class SingleBrane extends Brane {
    private final List<Insoe> statements;

    public SingleBrane(Characterizable characterization, List<Insoe> statements) {
        super(characterization);
        this.statements = List.copyOf(statements);
    }

    @Override
    protected List<Insoe> statements() {
        return statements;
    }
}
