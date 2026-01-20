package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class GetMyIdentifiersTest {

    private FIR createFir(String code) {
        // Since we don't have easy access to the parser here without complex setup,
        // we will manually construct FIRs or use mocks if possible.
        // But manual construction requires AST nodes.
        // Let's assume we can construct basic AST nodes.
        // If not, we might need a different approach or rely on the parser being available.
        // The codebase seems to have `org.foolish.ast.AST` available.
        return null;
    }

    // Since we cannot easily parse in a unit test without the full parser setup (which might be heavy),
    // and manually creating AST nodes can be verbose, let's try to construct simple cases.

    @Test
    void testIdentifier() {
        AST.Identifier idAst = new AST.Identifier(java.util.List.of(), "x");
        IdentifierFiroe fir = new IdentifierFiroe(idAst);

        Set<String> ids = fir.getMyIdentifiers();
        assertEquals(Set.of("x"), ids);
    }

    @Test
    void testValue() {
        ValueFiroe fir = new ValueFiroe(42);
        assertTrue(fir.getMyIdentifiers().isEmpty());
    }

    @Test
    void testAssignment() {
        // x = y
        // AssignmentFiroe needs an AST.Assignment
        // AST.Assignment needs an AST.Identifier (LHS) and AST.Expr (RHS)

        AST.Identifier lhs = new AST.Identifier(java.util.List.of(), "x");
        AST.Identifier rhs = new AST.Identifier(java.util.List.of(), "y");

        AST.Assignment assignmentAst = new AST.Assignment(lhs, rhs);

        AssignmentFiroe fir = new AssignmentFiroe(assignmentAst);
        // We need to initialize it? No, getMyIdentifiers constructs a temporary FIR from the expr.

        Set<String> ids = fir.getMyIdentifiers();
        assertEquals(Set.of("y"), ids);
    }

    @Test
    void testAssignmentWithBrane() {
        // x = { y = z }
        // RHS is a Brane. BraneFiroe.getMyIdentifiers should return empty.

        AST.Identifier lhs = new AST.Identifier(java.util.List.of(), "x");

        // Brane AST
        // Brane needs a list of Statements (Exprs)
        AST.Identifier z = new AST.Identifier(java.util.List.of(), "z");
        AST.Identifier y = new AST.Identifier(java.util.List.of(), "y");
        AST.Assignment innerAssign = new AST.Assignment(y, z);

        AST.Brane braneAst = new AST.Brane(java.util.List.of(), java.util.List.of(innerAssign));

        AST.Assignment assignmentAst = new AST.Assignment(lhs, braneAst);

        AssignmentFiroe fir = new AssignmentFiroe(assignmentAst);

        Set<String> ids = fir.getMyIdentifiers();
        assertTrue(ids.isEmpty(), "Should not collect identifiers from nested brane");
    }
}
