package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * RegexpSearchFiroe represents a regexp search operation stub.
 * For now, all regexp searches evaluate to ??? (unknown) in one step.
 * This is a placeholder until full regexp search evaluation is implemented.
 */
public class RegexpSearchFiroe extends FiroeWithoutBraneMind {

    public RegexpSearchFiroe(AST.RegexpSearchExpr regexpSearch) {
        super(regexpSearch);
    }

    /**
     * RegexpSearchFiroe is always abstract since we're stubbing the evaluation.
     * Once full regexp search is implemented, this should check if the search
     * has been resolved.
     */
    @Override
    public boolean isAbstract() {
        return true;
    }

    @Override
    public String toString() {
        return "???";
    }
}
