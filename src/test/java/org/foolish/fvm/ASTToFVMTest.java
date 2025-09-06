package org.foolish.fvm;

import org.foolish.ast.AST;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

public class ASTToFVMTest {
    @Test
    void translatesAndExecutes() {
        // Construct AST: { x = 5; x }
        AST.Expr assign = new AST.Assignment("x", new AST.IntegerLiteral(5));
        AST.Expr ident = new AST.Identifier("x");
        AST.Brane brane = new AST.Brane(List.of(assign, ident));
        AST.Branes branes = new AST.Branes(List.of(brane));
        AST.Program program = new AST.Program(branes);

        ASTToFVM translator = new ASTToFVM();
        Program prog = translator.translate(program);
        Object result = prog.execute(new Environment());
        assertEquals(5L, result);
    }
}
