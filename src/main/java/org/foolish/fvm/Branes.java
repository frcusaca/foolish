package org.foolish.fvm;

import java.util.ArrayList;
import java.util.List;

/**
 * A collection of branes.  Execution delegates to each contained brane in
 * order.  The primary difference from {@link SingleBrane} is how statements are
 * retrieved.
 */
public class Branes extends Brane {
    private final List<Brane> branes;

    public Branes(List<Brane> branes) {
        super(null);
        this.branes = List.copyOf(branes);
    }

    @Override
    protected List<Targoe> statements() {
        List<Targoe> stmts = new ArrayList<>();
        for (Brane b : branes) {
            stmts.addAll(b.statements());
        }
        return stmts;
    }
}
