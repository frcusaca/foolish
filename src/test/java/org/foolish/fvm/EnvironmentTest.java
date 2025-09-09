package org.foolish.fvm;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class EnvironmentTest {
    @Test
    void cascadesLookups() {
        Environment parent = new Environment();
        Characterizable x = new Characterizable("x");
        parent.define(x, Finear.of(1));

        Environment child = new Environment(parent);
        assertEquals(1L, child.lookup(x).value());

        child.define(x, Finear.of(2));
        assertEquals(2L, child.lookup(x).value());
        assertEquals(1L, parent.lookup(x).value());
    }
}
