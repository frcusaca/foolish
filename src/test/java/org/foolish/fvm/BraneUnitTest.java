package org.foolish.fvm;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import java.util.List;

public class BraneUnitTest {
    @Test
    void executesStatements() {
        Characterizable x = new Characterizable("x");
        Midoe fortyTwo = new Midoe();
        fortyTwo.progress_heap().add(Finear.of(42));
        List<Midoe> stmts = List.of(
                new Midoe(new Assignment(x, fortyTwo)),
                new Midoe(new IdentifierExpr(x))
        );
        SingleBrane brane = new SingleBrane(null, stmts);
        Environment env = new Environment();
        Finear result = new Midoe(brane).evaluate(env);
        assertEquals(42L, result.value());
        assertEquals(42L, env.lookup(x).value());
    }

    @Test
    void branesAggregateStatements() {
        Characterizable x = new Characterizable("x");
        Midoe one = new Midoe();
        one.progress_heap().add(Finear.of(1));
        Midoe two = new Midoe();
        two.progress_heap().add(Finear.of(2));
        SingleBrane b1 = new SingleBrane(null, List.of(new Midoe(new Assignment(x, one))));
        SingleBrane b2 = new SingleBrane(null, List.of(new Midoe(new Assignment(x, two))));
        Branes branes = new Branes(List.of(b1, b2));
        Environment env = new Environment();
        new Midoe(branes).evaluate(env);
        assertEquals(2L, env.lookup(x).value());
    }
}
