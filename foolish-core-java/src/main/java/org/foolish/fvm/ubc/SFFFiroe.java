package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

public class SFFFiroe extends FiroeWithoutBraneMind {
    public SFFFiroe(AST.SFFExpr ast) {
        super(ast);
    }

    // SFF holds the AST expression as a value without evaluating it.
    // It's already CONSTANT.

    @Override
    public String toString() {
        if (ast instanceof AST.SFFExpr sff) {
            return sff.expr().toString();
        }
        return super.toString();
    }

    @Override
    public boolean isAbstract() {
        return false;
    }
}
