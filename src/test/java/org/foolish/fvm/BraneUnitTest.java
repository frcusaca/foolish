package org.foolish.fvm;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import java.util.List;

public class BraneUnitTest {
    @Test
    void executesStatements() {
        Characterizable x = new Characterizable("x");
        Insoe fortyTwo = Finear.of(42);
        List<Insoe> stmts = List.of(
                new Assignment(x, fortyTwo),
                new IdentifierExpr(x)
        );
        SingleBrane brane = new SingleBrane(null, stmts);
        Environment env = new Environment();
        Finear result = MidoeVm.wrap(brane).evaluate(env);
        assertEquals(42L, result.value());
        assertEquals(42L, env.lookup(x).value());
    }

    @Test
    void branesAggregateStatements() {
        Characterizable x = new Characterizable("x");
        Insoe one = Finear.of(1);
        Insoe two = Finear.of(2);
        SingleBrane b1 = new SingleBrane(null, List.of(new Assignment(x, one)));
        SingleBrane b2 = new SingleBrane(null, List.of(new Assignment(x, two)));
        Branes branes = new Branes(List.of(b1, b2));
        Environment env = new Environment();
        MidoeVm.wrap(branes).evaluate(env);
        assertEquals(2L, env.lookup(x).value());
    }
}
