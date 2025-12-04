package org.foolish.fvm.ubc;

import org.apache.commons.lang3.tuple.Pair;
import org.foolish.ast.AST;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Unit tests for BraneMemory identifier resolution.
 *
 * Tests verify that identifier lookup finds the nearest binding above
 * the reference point, following lexical scoping rules.
 */
class BraneMemoryUnitTest {

    /**
     * Helper to create a simple assignment FIR for testing.
     */
    private AssignmentFiroe createAssignment(String name, long value) {
        return createAssignment(name, "", value);
    }

    /**
     * Helper to create a characterized assignment FIR for testing.
     */
    private AssignmentFiroe createAssignment(String name, String characterization, long value) {
        List<String> chars = (characterization == null || characterization.isEmpty())
            ? List.of()
            : List.of(characterization);
        AST.Assignment ast = new AST.Assignment(
            new AST.Identifier(chars, name),
            new AST.IntegerLiteral(value)
        );
        return new AssignmentFiroe(ast);
    }

    @Test
    void testSimpleLookup() {
        // Create a brane with one assignment: x = 42 at line 0
        BraneMemory brane = new BraneMemory(null);
        AssignmentFiroe assignment = createAssignment("x", 42);
        brane.put(assignment);

        // Query for x from line 0
        BraneMemory.Query query = new BraneMemory.StrictlyMatchingQuery("x", "");
        Optional<Pair<Integer, FIR>> result = brane.get(query, 0);

        assertTrue(result.isPresent());
        assertEquals(0, result.get().getLeft());
        assertSame(assignment, result.get().getRight());
    }

    @Test
    void testLookupFromLaterLine() {
        // Create a brane with: x = 42 at line 0
        BraneMemory brane = new BraneMemory(null);
        AssignmentFiroe assignment = createAssignment("x", 42);
        brane.put(assignment);

        // Add some other lines
        brane.put(createAssignment("y", 10));
        brane.put(createAssignment("z", 20));

        // Query for x from line 2 - should find line 0
        BraneMemory.Query query = new BraneMemory.StrictlyMatchingQuery("x", "");
        Optional<Pair<Integer, FIR>> result = brane.get(query, 2);

        assertTrue(result.isPresent());
        assertEquals(0, result.get().getLeft());
        assertSame(assignment, result.get().getRight());
    }

    @Test
    void testShadowing() {
        // Create a brane with two assignments to x:
        // Line 0: x = 10
        // Line 1: y = 20
        // Line 2: x = 30
        BraneMemory brane = new BraneMemory(null);
        AssignmentFiroe firstX = createAssignment("x", 10);
        AssignmentFiroe secondX = createAssignment("x", 30);

        brane.put(firstX);
        brane.put(createAssignment("y", 20));
        brane.put(secondX);

        // Query from line 1 should find first x (line 0)
        BraneMemory.Query query = new BraneMemory.StrictlyMatchingQuery("x", "");
        Optional<Pair<Integer, FIR>> result = brane.get(query, 1);
        assertTrue(result.isPresent());
        assertEquals(0, result.get().getLeft());
        assertSame(firstX, result.get().getRight());

        // Query from line 2 should find second x (line 2)
        result = brane.get(query, 2);
        assertTrue(result.isPresent());
        assertEquals(2, result.get().getLeft());
        assertSame(secondX, result.get().getRight());

        // Query from line 3 should find second x (line 2)
        result = brane.get(query, 3);
        assertTrue(result.isPresent());
        assertEquals(2, result.get().getLeft());
        assertSame(secondX, result.get().getRight());
    }

    @Test
    void testNotFound() {
        // Create a brane with x = 42
        BraneMemory brane = new BraneMemory(null);
        brane.put(createAssignment("x", 42));

        // Query for y - should not be found
        BraneMemory.Query query = new BraneMemory.StrictlyMatchingQuery("y", "");
        Optional<Pair<Integer, FIR>> result = brane.get(query, 0);

        assertFalse(result.isPresent());
    }

    @Test
    void testParentScope() {
        // Create parent brane with x = 100 at line 0
        BraneMemory parent = new BraneMemory(null);
        AssignmentFiroe parentX = createAssignment("x", 100);
        parent.put(parentX);

        // Create child brane at position 1 in parent
        BraneMemory child = new BraneMemory(parent, 1);
        child.put(createAssignment("y", 200));

        // Query for x from child - should find it in parent
        BraneMemory.Query query = new BraneMemory.StrictlyMatchingQuery("x", "");
        Optional<Pair<Integer, FIR>> result = child.get(query, 0);

        assertTrue(result.isPresent());
        assertEquals(0, result.get().getLeft());
        assertSame(parentX, result.get().getRight());
    }

    @Test
    void testParentScopeShadowing() {
        // Create parent brane with x = 100 at line 0
        BraneMemory parent = new BraneMemory(null);
        parent.put(createAssignment("x", 100));
        parent.put(createAssignment("y", 50));

        // Create child brane at position 2 in parent
        BraneMemory child = new BraneMemory(parent, 2);
        AssignmentFiroe childX = createAssignment("x", 200);
        child.put(childX);

        // Query for x from child - should find child's x
        BraneMemory.Query query = new BraneMemory.StrictlyMatchingQuery("x", "");
        Optional<Pair<Integer, FIR>> result = child.get(query, 0);

        assertTrue(result.isPresent());
        assertEquals(0, result.get().getLeft());
        assertSame(childX, result.get().getRight());
    }

    @Test
    void testParentScopeOnlyVisibleUpToChildPosition() {
        // Create parent brane with:
        // Line 0: x = 100
        // Line 1: (child brane conceptually here)
        // Line 1 actual: y = 200 (added after child created)
        // Line 2: z = 300
        BraneMemory parent = new BraneMemory(null);
        AssignmentFiroe xAssignment = createAssignment("x", 100);
        parent.put(xAssignment);  // line 0

        // Create child at position 1 - can see up to and including position 1 in parent
        BraneMemory child = new BraneMemory(parent, 1);

        // Add y and z to parent AFTER child created
        AssignmentFiroe yAssignment = createAssignment("y", 200);
        parent.put(yAssignment);  // line 1
        parent.put(createAssignment("z", 300));  // line 2

        // Query for x from child - should find it (line 0 <= child position 1)
        BraneMemory.Query queryX = new BraneMemory.StrictlyMatchingQuery("x", "");
        Optional<Pair<Integer, FIR>> resultX = child.get(queryX, 0);
        assertTrue(resultX.isPresent());
        assertSame(xAssignment, resultX.get().getRight());

        // Query for y from child - should find it (line 1 <= child position 1)
        BraneMemory.Query queryY = new BraneMemory.StrictlyMatchingQuery("y", "");
        Optional<Pair<Integer, FIR>> resultY = child.get(queryY, 0);
        assertTrue(resultY.isPresent());
        assertSame(yAssignment, resultY.get().getRight());

        // Query for z from child - should NOT find it (line 2 > child position 1)
        BraneMemory.Query queryZ = new BraneMemory.StrictlyMatchingQuery("z", "");
        Optional<Pair<Integer, FIR>> resultZ = child.get(queryZ, 0);
        assertFalse(resultZ.isPresent());
    }

    @Test
    void testNestedScopes() {
        // Create three-level nested scopes
        BraneMemory grandparent = new BraneMemory(null);
        AssignmentFiroe grandparentX = createAssignment("x", 100);
        grandparent.put(grandparentX);

        BraneMemory parent = new BraneMemory(grandparent, 1);
        AssignmentFiroe parentY = createAssignment("y", 200);
        parent.put(parentY);

        BraneMemory child = new BraneMemory(parent, 1);
        AssignmentFiroe childZ = createAssignment("z", 300);
        child.put(childZ);

        // Query for x from child - should traverse up to grandparent
        BraneMemory.Query queryX = new BraneMemory.StrictlyMatchingQuery("x", "");
        Optional<Pair<Integer, FIR>> result = child.get(queryX, 0);
        assertTrue(result.isPresent());
        assertSame(grandparentX, result.get().getRight());

        // Query for y from child - should find in parent
        BraneMemory.Query queryY = new BraneMemory.StrictlyMatchingQuery("y", "");
        result = child.get(queryY, 0);
        assertTrue(result.isPresent());
        assertSame(parentY, result.get().getRight());

        // Query for z from child - should find in child
        BraneMemory.Query queryZ = new BraneMemory.StrictlyMatchingQuery("z", "");
        result = child.get(queryZ, 0);
        assertTrue(result.isPresent());
        assertSame(childZ, result.get().getRight());
    }

    @Test
    void testCharacterizedIdentifier() {
        // Create a brane with characterized assignments
        BraneMemory brane = new BraneMemory(null);
        AssignmentFiroe characterizedAssignment = createAssignment("x", "type", 42);
        AssignmentFiroe plainAssignment = createAssignment("x", "", 100);

        brane.put(characterizedAssignment);  // line 0: type'x = 42
        brane.put(plainAssignment);          // line 1: x = 100

        // Query for type'x - canonicalCharacterization adds trailing '
        BraneMemory.Query query = new BraneMemory.StrictlyMatchingQuery("x", "type'");
        Optional<Pair<Integer, FIR>> result = brane.get(query, 1);
        assertTrue(result.isPresent());
        assertEquals(0, result.get().getLeft());
        assertSame(characterizedAssignment, result.get().getRight());

        // Query for uncharacterized x from line 1 - should find line 1
        BraneMemory.Query queryPlain = new BraneMemory.StrictlyMatchingQuery("x", "");
        result = brane.get(queryPlain, 1);
        assertTrue(result.isPresent());
        assertEquals(1, result.get().getLeft());
        assertSame(plainAssignment, result.get().getRight());
    }

    @Test
    void testBackwardsSearch() {
        // Create a brane with multiple assignments at different lines
        BraneMemory brane = new BraneMemory(null);
        AssignmentFiroe x1 = createAssignment("x", 10);
        AssignmentFiroe x2 = createAssignment("x", 20);
        AssignmentFiroe x3 = createAssignment("x", 30);

        brane.put(x1);              // line 0: x = 10
        brane.put(x2);              // line 1: x = 20
        brane.put(createAssignment("y", 5));   // line 2: y = 5
        brane.put(x3);              // line 3: x = 30

        BraneMemory.Query query = new BraneMemory.StrictlyMatchingQuery("x", "");

        // From line 0: finds line 0
        Optional<Pair<Integer, FIR>> result = brane.get(query, 0);
        assertTrue(result.isPresent());
        assertEquals(0, result.get().getLeft());

        // From line 1: finds line 1 (not line 0)
        result = brane.get(query, 1);
        assertTrue(result.isPresent());
        assertEquals(1, result.get().getLeft());

        // From line 2: finds line 1 (nearest above)
        result = brane.get(query, 2);
        assertTrue(result.isPresent());
        assertEquals(1, result.get().getLeft());

        // From line 3: finds line 3 (not earlier ones)
        result = brane.get(query, 3);
        assertTrue(result.isPresent());
        assertEquals(3, result.get().getLeft());

        // From line 4 (beyond end): finds line 3
        result = brane.get(query, 4);
        assertTrue(result.isPresent());
        assertEquals(3, result.get().getLeft());
    }
}
