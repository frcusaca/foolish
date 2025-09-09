package org.foolish.fvm;

import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

public class BraneTest {
    @Test
    void executesStatementsWithoutLeakingEnvironment() {
        Characterizable x = new Characterizable("x");
        List<Targoe> stmts = List.of(
                new Assignment(x, new IntegerLiteral(42)),
                new IdentifierExpr(x)
        );
        SingleBrane brane = new SingleBrane(null, stmts);
        Environment env = new Environment();
        EvalResult result = brane.execute(env);
        assertEquals(42L, result.value().asLong());
        assertSame(Unknown.INSTANCE, env.lookup(x));
    }

    @Test
    void nestedBranesRespectCopyOnWrite() {
        Characterizable x = new Characterizable("x");
        Characterizable y = new Characterizable("y");
        Characterizable z = new Characterizable("z");

        // level 3
        SingleBrane inner = new SingleBrane(null, List.of(
                new IdentifierExpr(x),
                new Assignment(z, new IntegerLiteral(3)),
                new IdentifierExpr(z)
        ));

        // level 2
        SingleBrane middle = new SingleBrane(null, List.of(
                new Assignment(y, new IntegerLiteral(2)),
                inner,
                new IdentifierExpr(z) // should be unknown outside inner
        ));

        // level 1
        SingleBrane outer = new SingleBrane(null, List.of(
                new Assignment(x, new IntegerLiteral(1)),
                middle,
                new IdentifierExpr(y) // should be unknown outside middle
        ));

        Environment env = new Environment();
        EvalResult r = outer.execute(env);
        assertSame(Unknown.INSTANCE, r.value());
        assertSame(Unknown.INSTANCE, env.lookup(x));
        assertSame(Unknown.INSTANCE, env.lookup(y));
        assertSame(Unknown.INSTANCE, env.lookup(z));
    }
}

