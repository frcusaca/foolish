package org.foolish.fvm;

import org.foolish.ast.AST;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

class FinearVmSimpleUnitTest {
    private FinearVmSimple vm;
    private Env env;

    @BeforeEach
    void setup() {
        vm = new FinearVmSimple();
        env = new Env();
    }

    @Test
    void evaluateSimpleArithmetic() {
        Finear left = Finear.of(5);
        Finear right = Finear.of(3);
        BinaryFiroe add = new BinaryFiroe(null, "+", left, right);

        Firoe result = vm.evaluate(add, env);

        assertInstanceOf(Finear.class, result);
        assertEquals(8L, ((Finear) result).value());
    }

    @Test
    void evaluateVariableAssignment() {
        Finear value = Finear.of(42);
        IdentifierFiroe id = new IdentifierFiroe("x");
        AssignmentFiroe assignment = new AssignmentFiroe(null, id.id(), value);

        vm.evaluate(assignment, env);

        Targoe stored = env.get("x");
        assertInstanceOf(Finear.class, stored);
        assertEquals(42L, ((Finear) stored).longValue());
    }

    @Test
    void evaluateIfCondition() {
        Finear condition = Finear.of(1);
        Finear thenValue = Finear.of(42);
        Finear elseValue = Finear.of(24);
        IfFiroe ifFiroe = new IfFiroe(null, condition, thenValue, elseValue, List.of());

        Firoe result = vm.evaluate(ifFiroe, env);

        assertInstanceOf(Finear.class, result);
        assertEquals(42L, ((Finear) result).value());
    }

    @Test
    void evaluateUnaryOperation() {
        Finear value = Finear.of(5);
        UnaryFiroe unary = new UnaryFiroe("-", value);

        Firoe result = vm.evaluate(unary, env);

        assertInstanceOf(Finear.class, result);
        assertEquals(-5L, ((Finear) result).value());
    }

    @Test
    void evaluateChainedOperations() {
        // Equivalent to (x = 5) + (y = 3)
        Finear five = Finear.of(5);
        Finear three = Finear.of(3);
        IdentifierFiroe xId = new IdentifierFiroe("x");
        IdentifierFiroe yId = new IdentifierFiroe("y");
        AssignmentFiroe xAssign = new AssignmentFiroe(null, xId.id(), five);
        AssignmentFiroe yAssign = new AssignmentFiroe(null, yId.id(), three);
        BinaryFiroe add = new BinaryFiroe(null,"+", xAssign, yAssign);

        vm.evaluate(add, env);

        assertEquals(5L, ((Finear) env.get("x")).value());
        assertEquals(3L, ((Finear) env.get("y")).value());
    }

    @Test
    void evaluateUnknownReturnsUnknown() {
        Firoe unknown = new Firoe();
        Firoe result = vm.evaluate(unknown, env);
        assertEquals(Finear.UNKNOWN, result);
    }

    @Test
    void evaluateBraneWithMultipleStatements() {
        Finear first = Finear.of(1);
        Finear second = Finear.of(2);
        BraneFiroe brane = new BraneFiroe(null, List.of(first, second));

        Firoe result = vm.evaluate(brane, env);

        assertNotEquals(Finear.UNKNOWN, result);
        assertTrue(result instanceof BraneFiroe);
    }
}