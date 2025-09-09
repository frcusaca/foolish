package org.foolish.fvm;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import java.util.List;

public class BraneUnitTest {
    @Test
    void executesStatements() {
        Characterizable x = new Characterizable("x");
        List<Targoe> stmts = List.of(
                new Midoe(new Assignment(x, Finer.of(42))),
                new Midoe(new IdentifierExpr(x))
        );
        SingleBrane brane = new SingleBrane(null, stmts);
        Environment env = new Environment();
        Finer result = new Midoe(brane).evaluate(env);
        assertEquals(42L, result.value());
        assertEquals(42L, env.lookup(x).value());
    }

    @Test
    void branesAggregateStatements() {
        Characterizable x = new Characterizable("x");
        SingleBrane b1 = new SingleBrane(null, List.of(new Midoe(new Assignment(x, Finer.of(1)))));
        SingleBrane b2 = new SingleBrane(null, List.of(new Midoe(new Assignment(x, Finer.of(2)))));
        Branes branes = new Branes(List.of(b1, b2));
        Environment env = new Environment();
        new Midoe(branes).evaluate(env);
        assertEquals(2L, env.lookup(x).value());
    }
}
