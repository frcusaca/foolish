package org.foolish.fvm.ubc;

import org.apache.commons.lang3.tuple.Pair;
import org.foolish.ast.AST;

import java.util.Optional;

/**
 * RegexpSearchFiroe performs a regular expression search on a brane.
 */
public class RegexpSearchFiroe extends AbstractSearchFiroe {
    private final String pattern;

    public RegexpSearchFiroe(AST.RegexpSearchExpr regexpSearch) {
        super(regexpSearch, regexpSearch.operator());
        this.pattern = regexpSearch.pattern();
    }

    @Override
    protected void initialize() {
        super.initialize();
        AST.RegexpSearchExpr searchExpr = (AST.RegexpSearchExpr) ast;
        storeExprs(searchExpr.anchor());
    }

    @Override
    protected FIR executeSearch(BraneFiroe target) {
        Query.RegexpQuery query = new Query.RegexpQuery(pattern);
        BraneMemory targetMemory = target.braneMemory;

        Optional<Pair<Integer, FIR>> result = switch (operator) {
            case REGEXP_LOCAL -> {
                // Backward search: search from end to start (find last match)
                int searchFrom = targetMemory.size() - 1;
                yield targetMemory.getLocal(query, searchFrom);
            }
            case REGEXP_FORWARD_LOCAL -> {
                // Forward search: search from start to end (find first match)
                int searchFrom = 0;
                yield targetMemory.getLocalForward(query, searchFrom);
            }
            case REGEXP_GLOBAL -> {
                // Global backward search (find-all, not yet fully implemented)
                int searchFrom = targetMemory.size() - 1;
                yield targetMemory.get(query, searchFrom);
            }
            default -> throw new IllegalStateException("Unknown regexp operator: " + operator);
        };

        return result.map(Pair::getValue).orElse(new NKFiroe());
    }

    @Override
    public String toString() {
        return ast.toString();
    }
}
