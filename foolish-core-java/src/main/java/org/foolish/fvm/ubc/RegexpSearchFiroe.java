package org.foolish.fvm.ubc;

import org.apache.commons.lang3.tuple.Pair;
import org.foolish.ast.AST;

import java.util.Optional;

/**
 * RegexpSearchFiroe performs a regular expression search on a brane.
 */
public class RegexpSearchFiroe
        extends AbstractSearchFiroe
        implements Constanicable {
    private final String pattern;

    public RegexpSearchFiroe(AST.RegexpSearchExpr regexpSearch) {
        super(regexpSearch, regexpSearch.operator());
        this.pattern = regexpSearch.pattern();
    }

    /**
     * Copy constructor for cloneConstanic.
     */
    protected RegexpSearchFiroe(RegexpSearchFiroe original, FIR newParent) {
        super(original, newParent);
        this.pattern = original.pattern;
    }

    @Override
    protected void initialize() {
        super.initialize();
        AST.RegexpSearchExpr searchExpr = (AST.RegexpSearchExpr) ast;
        storeExprs(searchExpr.anchor());
    }

    @Override
    protected FIR executeSearch(Cursor cursor) {
        Query.RegexpQuery query = new Query.RegexpQuery(pattern);
        ReadOnlyBraneMemory targetMemory = cursor.brane().getBraneMemory();

        Optional<Pair<Integer, FIR>> result = switch (operator) {
            case REGEXP_LOCAL -> targetMemory.getLocal(cursor, query);
            case REGEXP_FORWARD_LOCAL -> targetMemory.getLocalForward(cursor, query);
            case REGEXP_GLOBAL -> targetMemory.get(cursor, query);
            default -> throw new IllegalStateException("Unknown regexp operator: " + operator);
        };

        return result.map(Pair::getValue).orElse(new NKFiroe());
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

        RegexpSearchFiroe copy = new RegexpSearchFiroe(this, newParent);

        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }

        return copy;
    }
}
