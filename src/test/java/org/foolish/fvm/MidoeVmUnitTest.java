package org.foolish.fvm;

import org.foolish.ast.AST;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

public class MidoeVmUnitTest {
    @Test
    void wrapHandlesNullInput() {
        Midoe result = MidoeVm.wrap(null);
        assertNotNull(result);
        assertTrue(result instanceof Midoe);
    }

    @Test
    void wrapHandlesProgram() {
        AST.Program program = new AST.Program(new AST.Branes(List.of()));
        Insoe input = new Insoe(program);
        Midoe result = MidoeVm.wrap(input);

        assertTrue(result instanceof ProgramMidoe);
    }

    @Test
    void wrapHandlesBinaryExpression() {
        AST.Identifier left = new AST.Identifier("x");
        AST.IntegerLiteral right = new AST.IntegerLiteral(42);
        AST.BinaryExpr binExpr = new AST.BinaryExpr("+", left, right);
        Insoe input = new Insoe(binExpr);

        Midoe result = MidoeVm.wrap(input);

        assertTrue(result instanceof BinaryMidoe);
        BinaryMidoe binary = (BinaryMidoe) result;
        assertInstanceOf(IdentifierMidoe.class, binary.left());
        assertInstanceOf(Finear.class, binary.right());
    }

    @Test
    void wrapHandlesIntegerLiteral() {
        AST.IntegerLiteral literal = new AST.IntegerLiteral(42);
        Insoe input = new Insoe(literal);

        Midoe result = MidoeVm.wrap(input);

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

        Midoe result = MidoeVm.wrap(input);

        assertInstanceOf(IfMidoe.class, result);
        IfMidoe ifMidoe = (IfMidoe) result;
        assertInstanceOf(IdentifierMidoe.class, ifMidoe.condition());
        assertInstanceOf(Finear.class, ifMidoe.thenExpr());
        assertInstanceOf(Finear.class, ifMidoe.elseExpr());
        assertTrue(ifMidoe.elseIfs().isEmpty());
    }
}