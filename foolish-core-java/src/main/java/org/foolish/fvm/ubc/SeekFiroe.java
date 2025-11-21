package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * SeekFiroe represents a seek operation to search by statement number.
 * For now, all seek operations evaluate to ??? (unknown) in one step.
 * This is a placeholder until full seek evaluation is implemented.
 *
 * Examples: myBrane#5 (5th statement), data#-2 (2nd from end)
 */
public class SeekFiroe extends FiroeWithoutBraneMind {

    public SeekFiroe(AST.SeekExpr seekExpr) {
        super(seekExpr);
    }

    /**
     * SeekFiroe is always abstract since we're stubbing the evaluation.
     * Once full seek is implemented, this should check if the seek
     * has been resolved to a specific statement.
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
