package org.foolish.fvm;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import java.util.List;

public class BraneTest {
    @Test
    void executesStatements() {
        // Foolish: { x = 42; x }
        Characterizable x = new Characterizable("x");
        List<Targoe> stmts = List.of(
                new Midoe(new Assignment(x, new IntegerLiteral(42))),
                new Midoe(new IdentifierExpr(x))
        );
        SingleBrane brane = new SingleBrane(null, stmts);
        Environment env = new Environment();
        Finer result = Evaluator.eval(brane, env);
        assertEquals(42L, result.value());
        assertEquals(42L, ((Finer) env.lookup(x)).value());
    }

    @Test
    void branesAggregateStatements() {
        // Foolish: { x = 1 } { x = 2 }
        Characterizable x = new Characterizable("x");
        SingleBrane b1 = new SingleBrane(null, List.of(new Midoe(new Assignment(x, new IntegerLiteral(1)))));
        SingleBrane b2 = new SingleBrane(null, List.of(new Midoe(new Assignment(x, new IntegerLiteral(2)))));
        Branes branes = new Branes(List.of(b1, b2));
        Environment env = new Environment();
        Evaluator.eval(branes, env);
        assertEquals(2L, ((Finer) env.lookup(x)).value());
    }
}

