package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

public class SearchFiroeUnitTest {

    @Test
    void testDotSearch() {
        // x = 10
        // y = { a = 1; b = 2; a = 3 }
        // z = y . "a"

        // Construct the AST for y
        AST.Brane yBrane = new AST.Brane(List.of(
            new AST.Assignment(new AST.Identifier("a"), new AST.IntegerLiteral(1)),
            new AST.Assignment(new AST.Identifier("b"), new AST.IntegerLiteral(2)),
            new AST.Assignment(new AST.Identifier("a"), new AST.IntegerLiteral(3))
        ));

        AssignmentFiroe yAss = new AssignmentFiroe(new AST.Assignment(new AST.Identifier("y"), yBrane));

        // Construct AST for z = y . "a"
        // AST.BraneRegexpSearch(Characterizable brane, String operator, String pattern)
        // brane is Identifier("y")

        AST.BraneRegexpSearch search = new AST.BraneRegexpSearch(
             new AST.Identifier("y"), ".", "a"
        );

        SearchFiroe searchFiroe = new SearchFiroe(search);

        // We need to run this in a context where y is defined.
        // FiroeWithBraneMind test wrapper?

        // Create a root BraneFiroe to hold everything.
        // But SearchFiroe needs to run.

        // Let's create a BraneFiroe that contains y assignment and the search as an expression (maybe wrapped in assignment to z).

        AST.Assignment zAss = new AST.Assignment(new AST.Identifier("z"), search);

        AST.Brane rootBraneAst = new AST.Brane(List.of(
             (AST.Expr) yAss.ast(), // Cast to Expr to satisfy type inference
             (AST.Expr) zAss
        ));

        BraneFiroe rootBrane = new BraneFiroe(rootBraneAst);

        // Step until z is resolved.
        while (rootBrane.isNye()) {
            rootBrane.step();
        }

        // Get z
        BraneMemory mem = rootBrane.getMemory();
        // z should be the last one
        FIR zVal = mem.getLast();
        assertTrue(zVal instanceof AssignmentFiroe);
        assertEquals("z", ((AssignmentFiroe) zVal).getId());

        // The value of z should be the result of the search.
        // The search result is an AssignmentFiroe (the matching line in y).
        // It should match the LAST "a = 3".

        long result = zVal.getValue();
        assertEquals(3, result);
    }

    @Test
    void testDotSearchWithRegex() {
        // y = { a1 = 10; b = 20; a2 = 30 }
        // z = y . "a.*"

        AST.Brane yBrane = new AST.Brane(List.of(
            new AST.Assignment(new AST.Identifier("a1"), new AST.IntegerLiteral(10)),
            new AST.Assignment(new AST.Identifier("b"), new AST.IntegerLiteral(20)),
            new AST.Assignment(new AST.Identifier("a2"), new AST.IntegerLiteral(30))
        ));

        AST.BraneRegexpSearch search = new AST.BraneRegexpSearch(
             new AST.Identifier("y"), ".", "a.*"
        );

        AST.Brane rootBraneAst = new AST.Brane(List.of(
             (AST.Expr) new AST.Assignment(new AST.Identifier("y"), yBrane),
             (AST.Expr) new AST.Assignment(new AST.Identifier("z"), search)
        ));

        BraneFiroe rootBrane = new BraneFiroe(rootBraneAst);
        while (rootBrane.isNye()) {
            rootBrane.step();
        }

        // Expect a2 = 30 because it's the last one matching a.*
        FIR zVal = rootBrane.getMemory().getLast();
        assertEquals(30, zVal.getValue());
    }

    @Test
    void testCharacterizedSearch() {
        // y = { type'a = 10; other'a = 20 }
        // z = y . "type'a"

        AST.Brane yBrane = new AST.Brane(List.of(
            new AST.Assignment(new AST.Identifier(List.of("type"), "a"), new AST.IntegerLiteral(10)),
            new AST.Assignment(new AST.Identifier(List.of("other"), "a"), new AST.IntegerLiteral(20))
        ));

        AST.BraneRegexpSearch search = new AST.BraneRegexpSearch(
             new AST.Identifier("y"), ".", "type'a"
        );

        AST.Brane rootBraneAst = new AST.Brane(List.of(
             (AST.Expr) new AST.Assignment(new AST.Identifier("y"), yBrane),
             (AST.Expr) new AST.Assignment(new AST.Identifier("z"), search)
        ));

        BraneFiroe rootBrane = new BraneFiroe(rootBraneAst);
        while (rootBrane.isNye()) {
            rootBrane.step();
        }

        FIR zVal = rootBrane.getMemory().getLast();
        assertEquals(10, zVal.getValue());
    }

    @Test
    void testPartialMatchWithAnchor() {
        // y = { abc = 10 }
        // z = y . "^a" -> contains ^. No wrapping.
        // If matches() was used, ^a vs abc -> False.
        // If find() is used, ^a vs abc -> True (matches prefix).

        AST.Brane yBrane = new AST.Brane(List.of(
            new AST.Assignment(new AST.Identifier("abc"), new AST.IntegerLiteral(10))
        ));

        AST.BraneRegexpSearch search = new AST.BraneRegexpSearch(
             new AST.Identifier("y"), ".", "^a"
        );

        AST.Brane rootBraneAst = new AST.Brane(List.of(
             (AST.Expr) new AST.Assignment(new AST.Identifier("y"), yBrane),
             (AST.Expr) new AST.Assignment(new AST.Identifier("z"), search)
        ));

        BraneFiroe rootBrane = new BraneFiroe(rootBraneAst);
        while (rootBrane.isNye()) {
            rootBrane.step();
        }

        FIR zVal = rootBrane.getMemory().getLast();
        assertEquals(10, zVal.getValue());
    }

    @Test
    void testRegexAnchors() {
        // y = { abc = 10; bcd = 20 }
        // z = y . "b.*"  -> matches ^b.*$ -> matches bcd (abc does not start with b)

        AST.Brane yBrane = new AST.Brane(List.of(
            new AST.Assignment(new AST.Identifier("abc"), new AST.IntegerLiteral(10)),
            new AST.Assignment(new AST.Identifier("bcd"), new AST.IntegerLiteral(20))
        ));

        AST.BraneRegexpSearch search = new AST.BraneRegexpSearch(
             new AST.Identifier("y"), ".", "b.*"
        );

        AST.Brane rootBraneAst = new AST.Brane(List.of(
             (AST.Expr) new AST.Assignment(new AST.Identifier("y"), yBrane),
             (AST.Expr) new AST.Assignment(new AST.Identifier("z"), search)
        ));

        BraneFiroe rootBrane = new BraneFiroe(rootBraneAst);
        while (rootBrane.isNye()) {
            rootBrane.step();
        }

        FIR zVal = rootBrane.getMemory().getLast();
        assertEquals(20, zVal.getValue());
    }

    @Test
    void testExplicitAnchors() {
        // y = { abc = 10; bcd = 20 }
        // z = y . ".*b.*" -> manual anchors provided? No, user said if it DOES NOT contain.
        // if user provides ".*b.*" it does not contain ^ or $. So it becomes ^.*b.*$. Matches both. Last is bcd.

        // Test user providing explicit anchor
        // z = y . "^a.*" -> contains ^. No wrapping. matches ^a.* -> abc.

        AST.Brane yBrane = new AST.Brane(List.of(
            new AST.Assignment(new AST.Identifier("abc"), new AST.IntegerLiteral(10)),
            new AST.Assignment(new AST.Identifier("bcd"), new AST.IntegerLiteral(20))
        ));

        AST.BraneRegexpSearch search = new AST.BraneRegexpSearch(
             new AST.Identifier("y"), ".", "^a.*"
        );

        AST.Brane rootBraneAst = new AST.Brane(List.of(
             (AST.Expr) new AST.Assignment(new AST.Identifier("y"), yBrane),
             (AST.Expr) new AST.Assignment(new AST.Identifier("z"), search)
        ));

        BraneFiroe rootBrane = new BraneFiroe(rootBraneAst);
        while (rootBrane.isNye()) {
            rootBrane.step();
        }

        FIR zVal = rootBrane.getMemory().getLast();
        assertEquals(10, zVal.getValue());
    }
}
