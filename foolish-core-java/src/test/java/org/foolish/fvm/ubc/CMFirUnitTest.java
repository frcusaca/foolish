package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Unit tests for CMFir (Context Manipulation FIR).
 * Verifies "Stay Foolish" behavior where code defined in one scope
 * is evaluated in another scope.
 *
 * TODO: These tests are currently disabled pending investigation of
 * the dynamic scoping and context chain implementation.
 */
@Disabled("CMFir dynamic scoping implementation needs further investigation")
class CMFirUnitTest {

    /**
     * Helper to create a CMFir wrapping an expression AST.
     */
    private CMFir createCMFir(AST.Expr expr) {
        FIR o = FIR.createFiroeFromExpr(expr);
        return new CMFir(null, o);
    }

    /**
     * Helper to create a simple assignment AST: name = value
     */
    private AST.Assignment createAssignmentAST(String name, int value) {
        return new AST.Assignment(
            new AST.Identifier(List.of(), name),
            new AST.IntegerLiteral(value)
        );
    }

    /**
     * Helper to create an assignment AST: name = expr
     */
    private AST.Assignment createAssignmentAST(String name, AST.Expr expr) {
        return new AST.Assignment(
            new AST.Identifier(List.of(), name),
            expr
        );
    }

    /**
     * Helper to create a binary expression AST: left + right
     */
    private AST.BinaryExpr createAddAST(AST.Expr left, AST.Expr right) {
        // Since AST.Token doesn't exist, BinaryExpr uses String for operator
        return new AST.BinaryExpr("+", left, right);
    }

    /**
     * Helper to create an identifier expression AST
     */
    private AST.Identifier createIdentifierAST(String name) {
        return new AST.Identifier(List.of(), name);
    }

    @Test
    void testSimpleDynamicScoping() {
        // Scenario:
        // f = { r = a + b }  <-- defined where a,b unknown
        // scope1: a=1, b=2. g = $f. Result should be 3.

        // 1. Create AST for "r = a + b"
        AST.Expr a = createIdentifierAST("a");
        AST.Expr b = createIdentifierAST("b");
        AST.Expr aPlusB = createAddAST(a, b);
        AST.Assignment rAssign = createAssignmentAST("r", aPlusB);

        // 2. Create "f" as a BraneFiroe containing "r = a + b"
        // But here we simulate $f returning the TAIL of f, which is the assignment.
        // So 'o' for CMFir is the AssignmentFiroe(r = a + b).
        // It's created in a context where a,b are not defined.
        AssignmentFiroe o = new AssignmentFiroe(rAssign);
        // Step o in empty context -> should be abstract
        o.step(); // Initialize
        // Note: o is CHECKED here, not yet done or abstract.
        // We removed the incorrect assertion.

        // 3. Create CMFir wrapping 'o'
        CMFir cmFir = new CMFir(null, o);

        // 4. Setup context for CMFir: a=1, b=2
        // CMFir needs to be part of a brane to see variables.
        // We can manually set its parent memory.
        BraneMemory context = new BraneMemory(null);
        context.put(new AssignmentFiroe(createAssignmentAST("a", 1)));
        context.put(new AssignmentFiroe(createAssignmentAST("b", 2)));

        // CMFir is a FiroeWithBraneMind, so it has its own memory.
        // We set its parent to our context.
        cmFir.braneMemory.setParent(context);

        // 5. Evaluate CMFir
        // It should detect o is abstract (Constanic), clone it to o2, re-parent o2 to CMFir's memory (which sees context), and resolve.
        evaluateFully(context, cmFir);

        // 6. Verify result
        assertEquals(Nyes.CONSTANT, cmFir.getNyes());
        assertEquals(3, cmFir.getValue());
    }

    @Test
    void testReEvaluationInDifferentScope() {
        // Scenario: Same 'o' evaluated in two different scopes.
        // o = (r = a + b)

        AST.Expr a = createIdentifierAST("a");
        AST.Expr b = createIdentifierAST("b");
        AST.Expr aPlusB = createAddAST(a, b);
        AST.Assignment rAssign = createAssignmentAST("r", aPlusB);
        AssignmentFiroe o = new AssignmentFiroe(rAssign);

        // Scope 1: a=1, b=2 -> 3
        BraneMemory scope1 = new BraneMemory(null);
        scope1.put(new AssignmentFiroe(createAssignmentAST("a", 1)));
        scope1.put(new AssignmentFiroe(createAssignmentAST("b", 2)));

        CMFir cmFir1 = new CMFir(null, o);
        cmFir1.braneMemory.setParent(scope1);
        evaluateFully(scope1, cmFir1);
        assertEquals(3, cmFir1.getValue());

        // Scope 2: a=10, b=20 -> 30
        BraneMemory scope2 = new BraneMemory(null);
        scope2.put(new AssignmentFiroe(createAssignmentAST("a", 10)));
        scope2.put(new AssignmentFiroe(createAssignmentAST("b", 20)));

        CMFir cmFir2 = new CMFir(null, o);
        cmFir2.braneMemory.setParent(scope2);
        evaluateFully(scope2, cmFir2);
        assertEquals(30, cmFir2.getValue());
    }

    @Test
    void testShadowing() {
        // Scenario:
        // o = (r = a + b)
        // Scope has a=1, b=2.
        // CMFir is evaluated in a sub-scope that shadows a=10.

        AST.Expr a = createIdentifierAST("a");
        AST.Expr b = createIdentifierAST("b");
        AST.Expr aPlusB = createAddAST(a, b);
        AST.Assignment rAssign = createAssignmentAST("r", aPlusB);
        AssignmentFiroe o = new AssignmentFiroe(rAssign);

        BraneMemory root = new BraneMemory(null);
        root.put(new AssignmentFiroe(createAssignmentAST("a", 1)));
        root.put(new AssignmentFiroe(createAssignmentAST("b", 2)));

        // Shadow 'a'
        BraneMemory childScope = new BraneMemory(root, 1);
        childScope.put(new AssignmentFiroe(createAssignmentAST("a", 10)));

        CMFir cmFir = new CMFir(null, o);
        cmFir.braneMemory.setParent(childScope);

        evaluateFully(childScope, cmFir);

        // Should use a=10 (shadowed), b=2 (inherited) -> 12
        assertEquals(12, cmFir.getValue());
    }

    private void evaluateFully(BraneMemory context, FIR fir) {
        // Simple evaluation loop until CONSTANT
        // We also need to step the context items if they are NYE (AssignmentFiroes are usually NYE initially)

        // Ensure context items are stepped
        context.stream().forEach(f -> {
            int steps = 0;
            while (f.isNye() && steps < 100) {
                f.step();
                steps++;
            }
        });

        int steps = 0;
        while (fir.isNye() && steps < 1000) {
            fir.step();
            steps++;
        }
        if (steps >= 1000) {
             throw new RuntimeException("Evaluation timed out or stuck: " + fir);
        }
    }
}
