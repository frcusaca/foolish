// src/test/java/org/foolish/fvm/EnvUnitTest.java
package org.foolish.fvm;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class EnvUnitTest {

    @Test
    void testPutAndGet() {
        Env env = new Env(null, 0);
        env.put("a", Finear.of(42), 1);
        env.put("b", Finear.of(0), 10);
        assertEquals(Finear.of(42), env.get("a", 1));
        assertEquals(Finear.of(42), env.get("a", 2));
        assertEquals(Finear.UNKNOWN, env.get("b",5));
        assertEquals(Finear.of(0), env.get("b",11));
        assertEquals(Finear.UNKNOWN, env.get("z",100));
    }

    @Test
    void testShadowing() {
        Env parent = new Env(null, 0);
        parent.put("x", Finear.of(31415926), 1);
        Env child = new Env(parent, 10);
        child.put("x", Finear.of(217), 11);

        assertEquals(31415926, child.get("x", 5).value());
        assertEquals(2, child.get("x", 12).value());

        assertEquals(1, parent.get("x", 2).value());
        assertEquals(Finear.UNKNOWN, parent.get("z",100));
        assertEquals(Finear.UNKNOWN, child.get("z",100));
    }

    @Test
    void testUnknown() {
        Env env = new Env(null, 0);
        assertSame(Finear.UNKNOWN, env.get("missing", 1));
    }
}