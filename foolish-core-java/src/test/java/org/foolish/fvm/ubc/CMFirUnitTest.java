package org.foolish.fvm.ubc;

import org.antlr.v4.runtime.*;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Unit tests for CMFir (Context Manipulation FIR).
 * Verifies "Stay Foolish" behavior where code defined in one scope
 * is evaluated in another scope.
 */
class CMFirUnitTest {

    /**
     * Parse Foolish code and return the root brane FIR.
     * This allows tests to be written using actual Foolish syntax.
     */
    private BraneFiroe parseFoolish(String code) {
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);

        AST.Program program = (AST.Program) new ASTBuilder().visit(parser.program());

        // Get the first (and typically only) brane from the program
        // Program.branes() returns Branes, which has branes() returning List<Characterizable>
        List<AST.Characterizable> braneList = program.branes().branes();
        if (braneList.isEmpty()) {
            throw new IllegalArgumentException("No branes found in parsed code");
        }

        AST.Expr firstBrane = (AST.Expr) braneList.get(0);
        FIR fir = FIR.createFiroeFromExpr(firstBrane);

        if (!(fir instanceof BraneFiroe)) {
            throw new IllegalArgumentException("Expected BraneFiroe, got " + fir.getClass());
        }

        return (BraneFiroe) fir;
    }

    /**
     * Evaluate a FIR fully until it reaches CONSTANT or CONSTANIC.
     * This version is for tests using parseFoolish() where the FIR handles its own context.
     */
    private void evaluateFully(FIR fir) {
        int steps = 0;
        while (fir.isNye() && steps < 10000) {
            fir.step();
            steps++;
        }
        if (steps >= 10000) {
            throw new RuntimeException("Evaluation timed out or stuck after " + steps + " steps: " + fir);
        }
    }

    /**
     * Evaluate a FIR with explicit context management.
     * This version is for tests that manually construct FIRs and set up parent relationships.
     * Ensures all context items in the entire parent chain are evaluated before stepping the target FIR.
     */
    private void evaluateFully(BraneMemory context, FIR fir) {
        // Ensure all context items in the entire parent chain are stepped
        BraneMemory current = context;
        while (current != null) {
            current.stream().forEach(f -> {
                int steps = 0;
                while (f.isNye() && steps < 1000) {
                    f.step();
                    steps++;
                }
            });
            current = current.getParent();
        }

        int steps = 0;
        while (fir.isNye() && steps < 10000) {
            fir.step();
            steps++;
        }
        if (steps >= 10000) {
             throw new RuntimeException("Evaluation timed out or stuck: " + fir);
        }
    }

    /**
     * Helper to look up a value from a brane's memory by identifier name.
     */
    private long lookupValue(BraneFiroe brane, String identifier) {
        Query query = new Query.StrictlyMatchingQuery(identifier, "");
        Optional<org.apache.commons.lang3.tuple.Pair<Integer, FIR>> result =
            brane.braneMemory.get(query, brane.braneMemory.size() - 1);

        if (result.isEmpty()) {
            throw new IllegalStateException("Identifier '" + identifier + "' not found");
        }

        FIR fir = result.get().getRight();

        if (fir instanceof AssignmentFiroe assignment) {
            return assignment.getResult().getValue();
        }

        return fir.getValue();
    }

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
        // f = { r = a + b }  <-- defined where a,b unknown (becomes CONSTANIC)
        // In new scope: a=1, b=2, g = $f
        // The $f reference triggers CMFir wrapping, re-evaluating f with a=1, b=2
        // Result: r should be 3

        String foolishCode = """
            {
                f = { r = a + b };
                a = 1;
                b = 2;
                g = $f
            }
            """;

        BraneFiroe rootBrane = parseFoolish(foolishCode);
        evaluateFully(rootBrane);

        // Verify the brane evaluated successfully
        assertTrue(rootBrane.isConstanic(), "Root brane should be CONSTANIC or CONSTANT");

        // Verify that g (which is $f evaluated in context with a=1, b=2) has value 3
        // Note: g will be the brane {r = a + b} evaluated in the context
        // We need to check the value of r within that brane
        long gValue = lookupValue(rootBrane, "g");
        assertEquals(3, gValue, "g = $f should evaluate to 3 (a + b where a=1, b=2)");
    }

    @Test
    void testReEvaluationInDifferentScope() {
        // Scenario: Same brane f referenced in two different scopes
        // f = { r = a + b } becomes CONSTANIC (a,b unbound)
        // Scope 1: a=1, b=2, g1 = $f  -> should give r=3
        // Scope 2: a=10, b=20, g2 = $f -> should give r=30

        String foolishCode = """
            {
                f = { r = a + b };
                scope1 = {
                    a = 1;
                    b = 2;
                    g1 = $f
                };
                scope2 = {
                    a = 10;
                    b = 20;
                    g2 = $f
                }
            }
            """;

        BraneFiroe rootBrane = parseFoolish(foolishCode);
        evaluateFully(rootBrane);

        assertTrue(rootBrane.isConstanic(), "Root brane should be CONSTANIC or CONSTANT");

        // Verify scope1 result
        long scope1Value = lookupValue(rootBrane, "scope1");
        assertEquals(3, scope1Value, "scope1 should evaluate g1 = $f to 3");

        // Verify scope2 result
        long scope2Value = lookupValue(rootBrane, "scope2");
        assertEquals(30, scope2Value, "scope2 should evaluate g2 = $f to 30");
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

        // Create root brane with a=1, b=2
        AST.Brane rootAst = new AST.Brane(
            List.of(),
            List.of(
                createAssignmentAST("a", 1),
                createAssignmentAST("b", 2)
            )
        );
        BraneFiroe rootBrane = new BraneFiroe(rootAst);
        evaluateFully(rootBrane.braneMemory, rootBrane);

        // Create child brane that shadows 'a' with 10
        AST.Brane childAst = new AST.Brane(
            List.of(),
            List.of(
                createAssignmentAST("a", 10)
            )
        );
        BraneFiroe childBrane = new BraneFiroe(childAst);
        childBrane.braneMemory.setParent(rootBrane.braneMemory);
        childBrane.setParentFir(rootBrane);
        evaluateFully(childBrane.braneMemory, childBrane);

        CMFir cmFir = new CMFir(null, o);
        cmFir.setParentFir(childBrane);

        evaluateFully(childBrane.braneMemory, cmFir);

        // Should use a=10 (shadowed), b=2 (inherited) -> 12
        assertEquals(12, cmFir.getValue());
    }
}
