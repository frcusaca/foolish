package org.foolish.fvm;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class EnvironmentTest {
    @Test
    void cascadesLookups() {
        Environment parent = new Environment();
        Characterizable x = new Characterizable("x");
        parent.define(x, 1L);

        Environment child = new Environment(parent);
        assertEquals(1L, child.lookup(x));

        child.define(x, 2L);
        assertEquals(2L, child.lookup(x));
        assertEquals(1L, parent.lookup(x));
    }
}
