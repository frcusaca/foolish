package org.foolish.fvm.v1;

import org.foolish.fvm.v1.Characterizable;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class CharacterizableUnitTest {
    @Test
    void equalityAndHashing() {
        Characterizable a1 = new Characterizable("a");
        Characterizable a2 = new Characterizable("a");
        assertEquals(a1, a2);
        assertEquals(a1.hashCode(), a2.hashCode());

        Characterizable c1 = new Characterizable(a1, "b"); // a'b
        Characterizable c2 = new Characterizable(new Characterizable("a"), "b");
        assertEquals(c1, c2);
        assertEquals("a'b", c1.toString());

        Characterizable d = new Characterizable(new Characterizable("x"), "b");
        assertNotEquals(c1, d);
    }
}
