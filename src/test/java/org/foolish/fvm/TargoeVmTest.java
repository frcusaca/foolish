package org.foolish.fvm;

import org.foolish.ast.AST;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

<<<<<<<< HEAD:src/test/java/org/foolish/fvm/ASTToFVMUnitTest.java
public class ASTToFVMUnitTest {
========
public class TargoeVmTest {
>>>>>>>> 127c9e5 (Consolidate VM logic by evaluation level):src/test/java/org/foolish/fvm/TargoeVmTest.java
    @Test
    void translatesAndExecutes() {
        // Construct AST: { x = 5; x }
        AST.Expr assign = new AST.Assignment("x", new AST.IntegerLiteral(5));
        AST.Expr ident = new AST.Identifier("x");
        AST.Brane brane = new AST.Brane(List.of(assign, ident));
        AST.Branes branes = new AST.Branes(List.of(brane));
        AST.Program program = new AST.Program(branes);

        TargoeVm translator = new TargoeVm();
        Program prog = translator.translate(program);
        Finear result = new Midoe(prog).evaluate(new Environment());
        assertEquals(5L, result.value());
    }

    @Test
    void translatesIfWithoutElse() {
        AST.IfExpr ifExpr = new AST.IfExpr(
                new AST.IntegerLiteral(1),
                new AST.IntegerLiteral(42),
                null,
                List.of()
        );
        AST.Brane brane = new AST.Brane(List.of(ifExpr));
        AST.Branes branes = new AST.Branes(List.of(brane));
        AST.Program program = new AST.Program(branes);

        TargoeVm translator = new TargoeVm();
        Program prog = translator.translate(program);
        Finear result = new Midoe(prog).evaluate(new Environment());
        assertEquals(42L, result.value());
    }

    @Test
    void translatesIfElseIfElse() {
        AST.IfExpr elif = new AST.IfExpr(
                new AST.IntegerLiteral(1),
                new AST.IntegerLiteral(2),
                null,
                List.of()
        );
        AST.IfExpr ifExpr = new AST.IfExpr(
                new AST.IntegerLiteral(0),
                new AST.IntegerLiteral(1),
                new AST.IntegerLiteral(3),
                List.of(elif)
        );
        AST.Brane brane = new AST.Brane(List.of(ifExpr));
        AST.Branes branes = new AST.Branes(List.of(brane));
        AST.Program program = new AST.Program(branes);

        TargoeVm translator = new TargoeVm();
        Program prog = translator.translate(program);
        Finear result = new Midoe(prog).evaluate(new Environment());
        assertEquals(2L, result.value());
    }
}
