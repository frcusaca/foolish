package org.foolish.fvm;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class EnvironmentTest {
    @Test
    void cascadesLookups() {
        // Foolish: parent { x = 1 } child { x = 2 }
        Environment parent = new Environment();
        Characterizable x = new Characterizable("x");
        parent.define(x, new IntegerLiteral(1));

        Environment child = new Environment(parent);
        assertEquals(1L, ((Finer) child.lookup(x)).value());

        child.define(x, new IntegerLiteral(2));
        assertEquals(2L, ((Finer) child.lookup(x)).value());
        assertEquals(1L, ((Finer) parent.lookup(x)).value());
    }
}

