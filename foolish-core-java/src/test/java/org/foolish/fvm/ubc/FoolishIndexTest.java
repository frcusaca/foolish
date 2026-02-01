package org.foolish.fvm.ubc;

import org.junit.jupiter.api.Test;
import java.util.List;
import static org.junit.jupiter.api.Assertions.*;

public class FoolishIndexTest {

    @Test
    public void testFoolishIndexToString() {
        FoolishIndex index = new FoolishIndex(List.of(0, 1, 2, 3, 4), ",");
        assertEquals("[0,1,2,3,4]", index.toString());
    }

    @Test
    public void testFoolishIndexBuilder() {
        FoolishIndex index = new FoolishIndexBuilder()
                .append(1)
                .append(2)
                .prepend(0)
                .build();
        assertEquals("[0,1,2]", index.toString());
        assertEquals(List.of(0, 1, 2), index.getIndices());
    }

    @Test
    public void testCustomSeparator() {
        FoolishIndex index = new FoolishIndexBuilder()
                .separator("-")
                .append(0)
                .append(1)
                .build();
        assertEquals("[0-1]", index.toString());
    }

    @Test
    public void testFIRGetMyIndex() {
        // Construct hierarchy:
        // root
        //   childBrane (index 0)
        //     stmt1 (index 0)
        //     stmt2 (index 1)
        //     grandChildBrane (index 2)
        //       stmt3 (index 0)

        BraneFiroe root = new BraneFiroe(null);
        BraneFiroe childBrane = new BraneFiroe(null);
        FIR stmt1 = new ValueFiroe(null, 1);
        FIR stmt2 = new ValueFiroe(null, 2);
        BraneFiroe grandChildBrane = new BraneFiroe(null);
        FIR stmt3 = new ValueFiroe(null, 3);

        // root contains childBrane
        root.enqueueFirs(childBrane);

        // childBrane contains stmt1, stmt2, grandChildBrane
        childBrane.enqueueFirs(stmt1, stmt2, grandChildBrane);

        // grandChildBrane contains stmt3
        grandChildBrane.enqueueFirs(stmt3);

        // Assertions

        // Root: [0]
        assertEquals("[0]", root.getMyIndex().toString());

        // ChildBrane: [0, 0] (Root(0) -> childBrane(0))
        assertEquals("[0,0]", childBrane.getMyIndex().toString());

        // stmt1: [0, 0, 0] (Root(0) -> childBrane(0) -> stmt1(0))
        assertEquals("[0,0,0]", stmt1.getMyIndex().toString());

        // stmt2: [0, 0, 1] (Root(0) -> childBrane(0) -> stmt2(1))
        assertEquals("[0,0,1]", stmt2.getMyIndex().toString());

        // grandChildBrane: [0, 0, 2]
        assertEquals("[0,0,2]", grandChildBrane.getMyIndex().toString());

        // stmt3: [0, 0, 2, 0]
        assertEquals("[0,0,2,0]", stmt3.getMyIndex().toString());
    }

    @Test
    public void testFIRGetMyIndexWithExpressions() {
        // root
        //   assignment (index 0): a = 1 + 2

        BraneFiroe root = new BraneFiroe(null);
        FIR one = new ValueFiroe(null, 1);
        FIR two = new ValueFiroe(null, 2);
        // We can't easily create BinaryFiroe without AST.BinaryExpr, but we can simulate parent relationship.
        // Or simply create an anonymous class or just use ValueFiroe as a placeholder.
        // Let's create a dummy FIR that acts as the assignment

        FIR assignment = new ValueFiroe(null, 0); // Represents a = ...

        // Enqueue assignment into root
        root.enqueueFirs(assignment);

        // Now simulate sub-expressions.
        // We need to set parentFir manually or use enqueueFirs if the parent supports it.
        // ValueFiroe doesn't support enqueueFirs (it's not FiroeWithBraneMind).
        // But BinaryFiroe extends FiroeWithBraneMind?
        // Let's check BinaryFiroe inheritance.
        // BinaryFiroe extends FIR directly? No, usually FiroeWithBraneMind or similar if it evaluates sub-expressions.
        // Let's assume we have a parent FIR 'exprParent' that is FiroeWithBraneMind.

        FiroeWithBraneMind exprParent = new FiroeWithBraneMind(null) {
            @Override
            protected void initialize() {}
        };

        // Put exprParent in root
        root.enqueueFirs(exprParent);

        // Put child expression in exprParent
        FIR childExpr = new ValueFiroe(null, 1);
        exprParent.enqueueFirs(childExpr);

        // exprParent index: [0, 1] (root has assignment at 0, exprParent at 1)
        assertEquals("[0,1]", exprParent.getMyIndex().toString());

        // childExpr should share the same statement index [0, 1]
        // Because childExpr.getMyBraneIndex() calls parentFir.getMyBraneIndex().
        // parentFir is exprParent.
        // exprParent is in root (BraneFiroe). So it returns its index in root.
        assertEquals("[0,1]", childExpr.getMyIndex().toString());
    }
}
