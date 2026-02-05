package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * OneShotSearchFiroe performs a one-shot search on a brane (head/tail).
 */
public class OneShotSearchFiroe extends AbstractSearchFiroe {

    public OneShotSearchFiroe(AST.OneShotSearchExpr oneShotSearch) {
        super(oneShotSearch, oneShotSearch.operator());
    }

    /**
     * Copy constructor for cloneConstanic.
     */
    protected OneShotSearchFiroe(OneShotSearchFiroe original, FIR newParent) {
        super(original, newParent);
    }

    @Override
    protected void initialize() {
        super.initialize();
        AST.OneShotSearchExpr searchExpr = (AST.OneShotSearchExpr) ast;
        storeExprs(searchExpr.anchor());
    }

    @Override
    protected FIR executeSearch(Cursor cursor) {
        ReadOnlyBraneMemory targetMemory = cursor.brane().getBraneMemory();
        if (targetMemory.isEmpty()) {
             return new NKFiroe();
        }

        try {
            return targetMemory.get(cursor.statementIndex());
        } catch (IndexOutOfBoundsException e) {
            return new NKFiroe();
        }
    }

    @Override
    public String toString() {
        return ast.toString();
    }

    @Override
    protected FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes) {
        if (!isConstanic()) {
            throw new IllegalStateException(
                formatErrorMessage("cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
                    "but this FIR is in state: " + getNyes()));
        }

        if (isConstant()) {
            return this;  // Share CONSTANT searches
        }

        OneShotSearchFiroe copy = new OneShotSearchFiroe(this, newParent);

        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }

        return copy;
    }
}
