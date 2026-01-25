package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.ast.SearchOperator;

/**
 * SeekFiroe implements offset-based access to a brane.
 * Syntax: b#0 (first), b#-1 (last).
 */
public class SeekFiroe extends AbstractSearchFiroe {

    private final int offset;

    public SeekFiroe(AST.SeekExpr seekExpr) {
        super(seekExpr, SearchOperator.SEEK);
        this.offset = seekExpr.offset();
    }

    @Override
    protected void initialize() {
        super.initialize();
        AST.SeekExpr seekExpr = (AST.SeekExpr) ast;
        enqueueExprs(seekExpr.anchor());
    }

    @Override
    protected FIR executeSearch(BraneFiroe target) {
        BraneMemory targetMemory = target.braneMemory;
        int size = targetMemory.size();
        int idx = offset;

        // Handle negative indexing
        if (idx < 0) {
            idx = size + idx;
        }

        // Check bounds
        if (idx >= 0 && idx < size) {
             return targetMemory.get(idx);
        }

        return new NKFiroe();
    }

    @Override
    public String toString() {
        return ast.toString();
    }
}
