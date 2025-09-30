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
        BinaryMidoe add = new BinaryMidoe(null, "+", left, right);

        Midoe result = vm.evaluate(add, env);

        assertInstanceOf(Finear.class, result);
        assertEquals(8L, ((Finear) result).value());
    }

    @Test
    void evaluateVariableAssignment() {
        Finear value = Finear.of(42);
        IdentifierMidoe id = new IdentifierMidoe("x");
        AssignmentMidoe assignment = new AssignmentMidoe(null, id.id(), value);

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
        IfMidoe ifMidoe = new IfMidoe(null, condition, thenValue, elseValue, List.of());

        Midoe result = vm.evaluate(ifMidoe, env);

        assertInstanceOf(Finear.class, result);
        assertEquals(42L, ((Finear) result).value());
    }

    @Test
    void evaluateUnaryOperation() {
        Finear value = Finear.of(5);
        UnaryMidoe unary = new UnaryMidoe("-", value);

        Midoe result = vm.evaluate(unary, env);

        assertInstanceOf(Finear.class, result);
        assertEquals(-5L, ((Finear) result).value());
    }

    @Test
    void evaluateChainedOperations() {
        // Equivalent to (x = 5) + (y = 3)
        Finear five = Finear.of(5);
        Finear three = Finear.of(3);
        IdentifierMidoe xId = new IdentifierMidoe("x");
        IdentifierMidoe yId = new IdentifierMidoe("y");
        AssignmentMidoe xAssign = new AssignmentMidoe(null, xId.id(), five);
        AssignmentMidoe yAssign = new AssignmentMidoe(null, yId.id(), three);
        BinaryMidoe add = new BinaryMidoe(null,"+", xAssign, yAssign);

        vm.evaluate(add, env);

        assertEquals(5L, ((Finear) env.get("x")).value());
        assertEquals(3L, ((Finear) env.get("y")).value());
    }

    @Test
    void evaluateUnknownReturnsUnknown() {
        Midoe unknown = new Midoe();
        Midoe result = vm.evaluate(unknown, env);
        assertEquals(Finear.UNKNOWN, result);
    }

    @Test
    void evaluateBraneWithMultipleStatements() {
        Finear first = Finear.of(1);
        Finear second = Finear.of(2);
        BraneMidoe brane = new BraneMidoe(null, List.of(first, second));

        Midoe result = vm.evaluate(brane, env);

        assertNotEquals(Finear.UNKNOWN, result);
        assertTrue(result instanceof BraneMidoe);
    }
}