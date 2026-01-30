package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.junit.jupiter.api.Test;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Minimal test to debug PRIMED state transition issue.
 */
class PrimedStateDebugTest {

    @Test
    void testSimpleBraneWithAssignment() {
        // Create a brane with one assignment: a = 1
        AST.Brane braneAst = new AST.Brane(
            List.of(),
            List.of(new AST.Assignment(
                new AST.Identifier(List.of(), "a"),
                new AST.IntegerLiteral(1)
            ))
        );

        BraneFiroe brane = new BraneFiroe(braneAst);

        System.out.println("=== Simple Brane Test ===");
        System.out.println("Initial: state=" + brane.getNyes() +
                         ", memory=" + brane.braneMemory.size() +
                         ", mind=" + brane.braneMind.size());

        // Step through evaluation
        int steps = 0;
        while (brane.isNye() && steps < 20) {
            Nyes before = brane.getNyes();
            brane.step();
            Nyes after = brane.getNyes();

            System.out.println("Step " + steps + ": " + before + " -> " + after +
                             ", memory=" + brane.braneMemory.size() +
                             ", mind=" + brane.braneMind.size());

            steps++;
        }

        System.out.println("Final: state=" + brane.getNyes() + ", steps=" + steps);

        assertTrue(brane.isConstant(), "Brane should reach CONSTANT state");
        // Branes don't have getValue() - they contain assignments
        // Check that the assignment 'a' is present and has value 1
        assertEquals(1, brane.braneMemory.size(), "Should have 1 assignment");
        FIR assignment = brane.braneMemory.get(0);
        assertTrue(assignment instanceof AssignmentFiroe, "Should be an assignment");
        assertEquals(1, ((AssignmentFiroe)assignment).getResult().getValue(), "Assignment result should be 1");
    }

    @Test
    void testBraneWithConstanic() {
        // Create a brane with unresolved identifier: f = { r = a + b }
        AST.Brane innerBrane = new AST.Brane(
            List.of(),
            List.of(new AST.Assignment(
                new AST.Identifier(List.of(), "r"),
                new AST.BinaryExpr("+",
                    new AST.Identifier(List.of(), "a"),
                    new AST.Identifier(List.of(), "b"))
            ))
        );

        AST.Brane outerBrane = new AST.Brane(
            List.of(),
            List.of(new AST.Assignment(
                new AST.Identifier(List.of(), "f"),
                innerBrane
            ))
        );

        BraneFiroe brane = new BraneFiroe(outerBrane);

        System.out.println("\n=== Constanic Brane Test ===");
        System.out.println("Initial: state=" + brane.getNyes());

        int steps = 0;
        while (brane.isNye() && steps < 50) {
            Nyes before = brane.getNyes();

            // Debug: Check what's in braneMind before stepping
            if (!brane.braneMind.isEmpty() && steps > 5) {
                FIR current = brane.braneMind.getFirst();
                System.out.println("  [Step " + steps + "] braneMind.first: " + current.getClass().getSimpleName() +
                                 " state=" + current.getNyes() +
                                 (current instanceof AssignmentFiroe ? " lhs=" + ((AssignmentFiroe)current).getId() : ""));
            }

            brane.step();
            Nyes after = brane.getNyes();

            if (before != after) {
                System.out.println("Step " + steps + ": " + before + " -> " + after);
            }

            // Check if f assignment is present
            if (steps % 10 == 0) {
                System.out.println("  At step " + steps + ": memory=" + brane.braneMemory.size() +
                                 ", mind=" + brane.braneMind.size());
                if (!brane.braneMemory.isEmpty()) {
                    FIR f = brane.braneMemory.get(0);
                    System.out.println("    memory[0]: " + f.getClass().getSimpleName() + " state=" + f.getNyes());
                }
            }

            steps++;
        }

        System.out.println("Final: state=" + brane.getNyes() + ", steps=" + steps);
        System.out.println("Output: " + brane);

        assertTrue(brane.isConstanic() || brane.isConstant(),
                  "Brane should reach CONSTANIC or CONSTANT state");
    }

    @Test
    void testCMFirBasic() {
        // Create f = { r = a + b } (will be CONSTANIC)
        AST.Brane innerBrane = new AST.Brane(
            List.of(),
            List.of(new AST.Assignment(
                new AST.Identifier(List.of(), "r"),
                new AST.BinaryExpr("+",
                    new AST.Identifier(List.of(), "a"),
                    new AST.Identifier(List.of(), "b"))
            ))
        );

        BraneFiroe innerFir = new BraneFiroe(innerBrane);

        System.out.println("\n=== CMFir Basic Test ===");

        // Evaluate inner brane to CONSTANIC
        System.out.println("Evaluating inner brane...");
        int steps = 0;
        while (innerFir.isNye() && steps < 50) {
            innerFir.step();
            steps++;
        }

        System.out.println("Inner brane state: " + innerFir.getNyes() + " after " + steps + " steps");
        assertTrue(innerFir.isConstanic(), "Inner brane should be CONSTANIC");

        // Create outer brane with a=1, b=2
        AST.Brane outerBrane = new AST.Brane(
            List.of(),
            List.of(
                new AST.Assignment(new AST.Identifier(List.of(), "a"), new AST.IntegerLiteral(1)),
                new AST.Assignment(new AST.Identifier(List.of(), "b"), new AST.IntegerLiteral(2))
            )
        );
        BraneFiroe outerFir = new BraneFiroe(outerBrane);

        // Evaluate outer brane
        System.out.println("Evaluating outer brane...");
        steps = 0;
        while (outerFir.isNye() && steps < 50) {
            outerFir.step();
            steps++;
        }
        System.out.println("Outer brane state: " + outerFir.getNyes() + " after " + steps + " steps");

        // Now create CMFir wrapping the inner brane
        System.out.println("\nCreating CMFir...");
        CMFir cmFir = new CMFir(innerBrane, innerFir);
        cmFir.setParentFir(outerFir);

        System.out.println("CMFir initial state: " + cmFir.getNyes());
        System.out.println("CMFir.o state: " + cmFir.getO().getNyes());
        System.out.println("CMFir parent isConstanic: " + cmFir.getParentFir().isConstanic());
        System.out.println("CMFir parent state: " + cmFir.getParentFir().getNyes());

        // Step CMFir
        System.out.println("\nStepping CMFir...");
        steps = 0;
        while (cmFir.isNye() && steps < 100) {
            Nyes before = cmFir.getNyes();
            cmFir.step();
            Nyes after = cmFir.getNyes();

            if (before != after) {
                System.out.println("Step " + steps + ": " + before + " -> " + after);
                if (cmFir.phaseBStarted) {
                    System.out.println("  Phase B started, o2 state: " +
                                     (cmFir.o2 != null ? cmFir.o2.getNyes() : "null"));
                }
            }

            steps++;

            if (steps > 50) {
                System.out.println("WARNING: More than 50 steps, breaking");
                break;
            }
        }

        System.out.println("Final CMFir state: " + cmFir.getNyes() + " after " + steps + " steps");
        assertTrue(cmFir.isConstant(), "CMFir should reach CONSTANT state");

        // CMFir wraps a brane, check the assignment result within it
        FIR result = cmFir.getResult();  // Gets o2 which is the re-evaluated brane
        assertTrue(result instanceof BraneFiroe, "Result should be a BraneFiroe");
        BraneFiroe resultBrane = (BraneFiroe) result;

        // Check that the brane has the assignment 'r = a + b' evaluated to 3
        assertEquals(1, resultBrane.braneMemory.size(), "Should have 1 assignment");
        FIR assignment = resultBrane.braneMemory.get(0);
        assertTrue(assignment instanceof AssignmentFiroe, "Should be an assignment");
        AssignmentFiroe assignFir = (AssignmentFiroe) assignment;
        assertEquals("r", assignFir.getId(), "Assignment should be for 'r'");
        assertEquals(3, assignFir.getResult().getValue(), "r should evaluate to 3 (1 + 2)");
        System.out.println("SUCCESS: CMFir correctly evaluated inner brane with r=3");
    }
}
