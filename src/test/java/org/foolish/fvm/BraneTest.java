package org.foolish.fvm;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import java.util.List;

public class BraneTest {
    @Test
    void executesStatements() {
        Characterizable x = new Characterizable("x");
        List<Instruction> stmts = List.of(
                new Assignment(x, new IntegerLiteral(42)),
                new IdentifierExpr(x)
        );
        SingleBrane brane = new SingleBrane(null, stmts);
        Environment env = new Environment();
        Object result = brane.execute(env);
        assertEquals(42L, result);
        assertEquals(42L, env.lookup(x));
    }

    @Test
    void branesAggregateStatements() {
        Characterizable x = new Characterizable("x");
        SingleBrane b1 = new SingleBrane(null, List.of(new Assignment(x, new IntegerLiteral(1))));
        SingleBrane b2 = new SingleBrane(null, List.of(new Assignment(x, new IntegerLiteral(2))));
        Branes branes = new Branes(List.of(b1, b2));
        Environment env = new Environment();
        branes.execute(env);
        assertEquals(2L, env.lookup(x));
    }
}
