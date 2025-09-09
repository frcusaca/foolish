package org.foolish.fvm;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests identifier resolution across nested branes with copy-on-write semantics.
 */
public class IdentifierResolutionTest {

    @Test
    void resolvesAcrossBraneScopes() {
        Characterizable x = new Characterizable("x");
        Characterizable y = new Characterizable("y");
        Characterizable before = new Characterizable("before");
        Characterizable after = new Characterizable("after");

        // Level 3 branes
        SingleBrane b3mut = new SingleBrane(null, List.of(
                new Assignment(x, new IntegerLiteral(30)),
                new IdentifierExpr(x)
        ));
        SingleBrane b3lookup = new SingleBrane(null, List.of(
                new IdentifierExpr(x)
        ));
        SingleBrane b3unknown = new SingleBrane(null, List.of(
                new IdentifierExpr(y)
        ));

        // Level 2 branes
        SingleBrane b2target = new SingleBrane(null, List.of(
                new Assignment(before, new IdentifierExpr(x)),
                new Assignment(x, new IntegerLiteral(20)),
                b3mut,
                b3lookup,
                b3unknown,
                new Assignment(after, new IdentifierExpr(x))
        ));
        // additional branes to ensure at least three per layer
        SingleBrane b2b = new SingleBrane(null, List.of(
                new SingleBrane(null, List.of(new IdentifierExpr(x))),
                new SingleBrane(null, List.of(new IdentifierExpr(x))),
                new SingleBrane(null, List.of(new IdentifierExpr(x)))
        ));
        SingleBrane b2c = new SingleBrane(null, List.of(
                new SingleBrane(null, List.of(new IdentifierExpr(x))),
                new SingleBrane(null, List.of(new IdentifierExpr(x))),
                new SingleBrane(null, List.of(new IdentifierExpr(x)))
        ));

        // Level 1 brane
        SingleBrane top = new SingleBrane(null, List.of(
                new Assignment(x, new IntegerLiteral(10)),
                b2target,
                b2b,
                b2c,
                new IdentifierExpr(x)
        ));

        Environment root = new Environment();
        Object topResult = top.execute(root);
        assertEquals(10L, topResult); // outer value unchanged by inner branes
        assertEquals(10L, root.lookup(x));

        // execute b2target separately to inspect captured values
        Environment envB2 = new Environment(root);
        b2target.execute(envB2);
        assertEquals(10L, envB2.lookup(before)); // resolved from outer brane
        assertEquals(20L, envB2.lookup(after));  // local assignment persists inside
        assertEquals(20L, envB2.lookup(x));      // inner mutation did not escape
        assertEquals(10L, root.lookup(x));       // parent unaffected

        // deep mutation does not escape
        Environment envB3 = new Environment(envB2);
        b3mut.execute(envB3);
        assertEquals(30L, envB3.lookup(x));
        assertEquals(20L, envB2.lookup(x));

        // lookup without local definition falls back to parent
        Environment envLookup = new Environment(envB2);
        Object lookupResult = b3lookup.execute(envLookup);
        assertEquals(20L, lookupResult);

        // unresolved identifier yields Unknown
        Environment envUnknown = new Environment(envB2);
        Object unknownResult = b3unknown.execute(envUnknown);
        assertSame(Unknown.INSTANCE, unknownResult);
    }
}

