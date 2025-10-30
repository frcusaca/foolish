package org.foolish.fvm;

import org.foolish.ast.AST;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

public class FiroeVmUnitTest {
    @Test
    void wrapHandlesNullInput() {
        Firoe result = FiroeVm.wrap(null);
        assertNotNull(result);
        assertTrue(result instanceof Firoe);
    }

    @Test
    void wrapHandlesProgram() {
        AST.Program program = new AST.Program(new AST.Branes(List.of()));
        Insoe input = new Insoe(program);
        Firoe result = FiroeVm.wrap(input);

        assertTrue(result instanceof ProgramFiroe);
    }

    @Test
    void wrapHandlesBinaryExpression() {
        AST.Identifier left = new AST.Identifier("x");
        AST.IntegerLiteral right = new AST.IntegerLiteral(42);
        AST.BinaryExpr binExpr = new AST.BinaryExpr("+", left, right);
        Insoe input = new Insoe(binExpr);

        Firoe result = FiroeVm.wrap(input);

        assertTrue(result instanceof BinaryFiroe);
        BinaryFiroe binary = (BinaryFiroe) result;
        assertInstanceOf(IdentifierFiroe.class, binary.left());
        assertInstanceOf(Finear.class, binary.right());
    }

    @Test
    void wrapHandlesIntegerLiteral() {
        AST.IntegerLiteral literal = new AST.IntegerLiteral(42);
        Insoe input = new Insoe(literal);

        Firoe result = FiroeVm.wrap(input);

        assertInstanceOf(Finear.class, result);
        assertEquals (42 , ((Finear) result).longValue());
    }

    @Test
    void wrapHandlesIfExpression() {
        AST.Identifier condition = new AST.Identifier("condition");
        AST.IntegerLiteral thenExpr = new AST.IntegerLiteral(1);
        AST.IntegerLiteral elseExpr = new AST.IntegerLiteral(0);
        AST.IfExpr ifExpr = new AST.IfExpr(condition, thenExpr, elseExpr, List.of());
        Insoe input = new Insoe(ifExpr);

        Firoe result = FiroeVm.wrap(input);

        assertInstanceOf(IfFiroe.class, result);
        IfFiroe ifFiroe = (IfFiroe) result;
        assertInstanceOf(IdentifierFiroe.class, ifFiroe.condition());
        assertInstanceOf(Finear.class, ifFiroe.thenExpr());
        assertInstanceOf(Finear.class, ifFiroe.elseExpr());
        assertTrue(ifFiroe.elseIfs().isEmpty());
    }
}