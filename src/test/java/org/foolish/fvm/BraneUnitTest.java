package org.foolish.fvm;

import org.foolish.ast.AST;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

import java.util.List;

public class BraneUnitTest {
    @Test
    void executesStatements() {
        AST.Expr assign = new AST.Assignment("x", new AST.IntegerLiteral(42));
        AST.Expr ident = new AST.Identifier("x");
        AST.Brane brane = new AST.Brane(List.of(assign, ident));
        AST.Branes branes = new AST.Branes(List.of(brane));
        AST.Program program = new AST.Program(branes);

        Environment env = new Environment();
        Insoe in = new TargoeVm().translate(program);
        Finear result = MidoeVm.wrap(in).evaluate(env);
        assertEquals(42L, result.value());
        assertEquals(42L, env.lookup(new Characterizable("x")).value());
    }

    @Test
    void branesAggregateStatements() {
        AST.Expr oneAssign = new AST.Assignment("x", new AST.IntegerLiteral(1));
        AST.Expr twoAssign = new AST.Assignment("x", new AST.IntegerLiteral(2));
        AST.Brane b1 = new AST.Brane(List.of(oneAssign));
        AST.Brane b2 = new AST.Brane(List.of(twoAssign));
        AST.Branes branes = new AST.Branes(List.of(b1, b2));
        AST.Program program = new AST.Program(branes);

        Environment env = new Environment();
        Insoe in = new TargoeVm().translate(program);
        MidoeVm.wrap(in).evaluate(env);
        assertEquals(2L, env.lookup(new Characterizable("x")).value());
    }
}
