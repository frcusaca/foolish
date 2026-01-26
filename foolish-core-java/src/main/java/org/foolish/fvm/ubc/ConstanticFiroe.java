package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

public class ConstanticFiroe extends FiroeWithBraneMind {
    public ConstanticFiroe(AST.ConstanticExpr ast) {
        super(ast);
    }

    @Override
    protected void initialize() {
        if (isInitialized()) return;
        setInitialized();
        if (ast instanceof AST.ConstanticExpr expr) {
            enqueueFirs(createFiroeFromExpr(expr.expr()));
        }
    }

    // Logic: Once evaluated, it should probably capture the value/brane.
    // The "Constantic" aspect means it captures state.
    // Default FiroeWithBraneMind evaluates.
    // Maybe we need to clone the result?
}
