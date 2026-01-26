package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

public class DetachmentFiroe extends FiroeWithoutBraneMind {
    public DetachmentFiroe(AST.DetachmentBrane ast) {
        super(ast);
    }

    @Override
    public String toString() {
        return ast.toString();
    }

    @Override
    public boolean isAbstract() {
        return false;
    }
}
