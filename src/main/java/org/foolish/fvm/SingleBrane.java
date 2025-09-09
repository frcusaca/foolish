package org.foolish.fvm;

import java.util.List;

/**
 * A single brane containing a list of statements.
 */
public class SingleBrane extends Brane {
    private final List<Targoe> statements;

    public SingleBrane(Characterizable characterization, List<Targoe> statements) {
        super(characterization, TargoeType.SINGLE_BRANE);
        this.statements = List.copyOf(statements);
    }

    @Override
    protected List<Targoe> statements() {
        return statements;
    }
}

