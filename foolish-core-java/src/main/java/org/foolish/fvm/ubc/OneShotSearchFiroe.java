package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * OneShotSearchFiroe performs a one-shot search on a brane (head/tail).
 */
public class OneShotSearchFiroe extends AbstractSearchFiroe {

    public OneShotSearchFiroe(AST.OneShotSearchExpr oneShotSearch) {
        super(oneShotSearch, oneShotSearch.operator());
    }

    @Override
    protected void initialize() {
        super.initialize();
        AST.OneShotSearchExpr searchExpr = (AST.OneShotSearchExpr) ast;
        storeExprs(searchExpr.anchor());
    }

    @Override
    protected FIR executeSearch(BraneFiroe target) {
        BraneMemory targetMemory = target.braneMemory;
        if (targetMemory.isEmpty()) {
             return new NKFiroe();
        }

        return switch (operator) {
            case HEAD -> targetMemory.get(0);
            case TAIL -> targetMemory.getLast();
            default -> throw new IllegalStateException("Unknown one-shot operator: " + operator);
        };
    }

    @Override
    public String toString() {
        return ast.toString();
    }
}
