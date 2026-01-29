package org.foolish.fvm.ubc;

import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

public class FoolishIndexTest {

    @Test
    public void testFoolishIndex() {
        // Corrected code from user example (added closing brace for b)
        String code = """
            {
              top={
                   a={a=1,b=2,c=3};
                   b={4,5,c={1,2,3,z=1}};
              }
            }
            """;

        BraneFiroe rootBrane = parseAndRun(code);
        assertNotNull(rootBrane);

        // Root Index: "0"
        assertEquals("0", rootBrane.getMyIndex().toString());

        // Find top
        AssignmentFiroe topAssign = findAssignment(rootBrane, "top");
        assertNotNull(topAssign);
        // top={...} is statement 0 in root.
        // Index of topAssign: 0, 0
        assertEquals("0,0", topAssign.getMyIndex().toString());

        BraneFiroe topBrane = (BraneFiroe) topAssign.getResult();
        assertNotNull(topBrane);
        // topBrane is part of topAssign. Should have same index as topAssign
        assertEquals("0,0", topBrane.getMyIndex().toString());

        // Find top.a
        AssignmentFiroe aAssign = findAssignment(topBrane, "a");
        assertNotNull(aAssign);
        // a={...} is statement 0 in top.
        // top is 0 in root.
        // Index: 0 (root), 0 (top), 0 (a).
        assertEquals("0,0,0", aAssign.getMyIndex().toString());

        BraneFiroe aBrane = (BraneFiroe) aAssign.getResult();

        // Find top.a.a (a=1 inside a)
        AssignmentFiroe aaAssign = findAssignment(aBrane, "a");
        assertNotNull(aaAssign);
        // a=1 is statement 0 in a.
        // Index: 0, 0, 0, 0.
        assertEquals("0,0,0,0", aaAssign.getMyIndex().toString());

        // Find top.b
        AssignmentFiroe bAssign = findAssignment(topBrane, "b");
        assertNotNull(bAssign);
        // b={...} is statement 1 in top.
        // Index: 0, 0, 1.
        assertEquals("0,0,1", bAssign.getMyIndex().toString());

        BraneFiroe bBrane = (BraneFiroe) bAssign.getResult();

        // Find top.b.c
        AssignmentFiroe cAssign = findAssignment(bBrane, "c");
        assertNotNull(cAssign);
        // c={...} is statement 2 in b (4, 5, c=...).
        // Index: 0, 0, 1, 2.
        assertEquals("0,0,1,2", cAssign.getMyIndex().toString());

        BraneFiroe cBrane = (BraneFiroe) cAssign.getResult();

        // Find top.b.c.z (z=1 inside c)
        AssignmentFiroe zAssign = findAssignment(cBrane, "z");
        assertNotNull(zAssign);
        // z=1 is statement 3 in c (1, 2, 3, z=1).
        // Index: 0, 0, 1, 2, 3.
        assertEquals("0,0,1,2,3", zAssign.getMyIndex().toString());
    }

    private BraneFiroe parseAndRun(String code) {
        FoolishLexer lexer = new FoolishLexer(CharStreams.fromString(code));
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        AST.Program program = (AST.Program) new ASTBuilder().visit(parser.program());
        AST.Brane brane = (AST.Brane) program.branes().branes().get(0);

        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(brane);
        ubc.runToCompletion();
        return ubc.getRootBrane();
    }

    private AssignmentFiroe findAssignment(BraneFiroe brane, String id) {
        for (FIR fir : brane.getExpressionFiroes()) {
            if (fir instanceof AssignmentFiroe assign) {
                if (assign.getId().equals(id)) {
                    return assign;
                }
            }
        }
        return null;
    }
}
