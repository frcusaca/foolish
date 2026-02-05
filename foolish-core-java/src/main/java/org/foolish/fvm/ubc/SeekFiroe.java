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

    /**
     * Copy constructor for cloneConstanic.
     */
    protected SeekFiroe(SeekFiroe original, FIR newParent) {
        super(original, newParent);
        this.offset = original.offset;
    }

    @Override
    protected void initialize() {
        super.initialize();
        AST.SeekExpr seekExpr = (AST.SeekExpr) ast;
        storeExprs(seekExpr.anchor());
    }

    @Override
    protected FIR executeSearch(Cursor cursor) {
        ReadOnlyBraneMemory targetMemory = cursor.brane().getBraneMemory();
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

    @Override
    protected FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes) {
        if (!isConstanic()) {
            throw new IllegalStateException(
                formatErrorMessage("cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
                    "but this FIR is in state: " + getNyes()));
        }

        if (isConstant()) {
            return this;  // Share CONSTANT seeks
        }

        SeekFiroe copy = new SeekFiroe(this, newParent);

        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }

        return copy;
    }
}
