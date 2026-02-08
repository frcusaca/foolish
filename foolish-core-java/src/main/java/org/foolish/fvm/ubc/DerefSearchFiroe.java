package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.ast.SearchOperator;
import java.util.Optional;

/**
 * DerefSearchFiroe handles dereferencing operations (e.g. obj.prop or anchored exact searches).
 * It searches strictly within the anchor's brane (braneBound=true).
 */
public class DerefSearchFiroe extends AbstractSearchFiroe implements Constanicable {
    private final RegexpSearcher searcher;

    public DerefSearchFiroe(AST.RegexpSearchExpr regexpSearch) {
        super(regexpSearch, regexpSearch.operator());
        this.searcher = new RegexpSearcher(regexpSearch.pattern());
    }

    public DerefSearchFiroe(AST.RegexpSearchExpr syntheticExpr, AST.DereferenceExpr originalExpr) {
        super(syntheticExpr, syntheticExpr.operator());
        this.searcher = new RegexpSearcher(syntheticExpr.pattern());
    }

    /**
     * Copy constructor for cloneConstanic.
     */
    protected DerefSearchFiroe(DerefSearchFiroe original, FIR newParent) {
        super(original, newParent);
        this.searcher = original.searcher;
    }

    @Override
    protected void initialize() {
        super.initialize();
        if (ast instanceof AST.RegexpSearchExpr searchExpr) {
            storeExprs(searchExpr.anchor());
        }
    }

    @Override
    protected SearchCursor createCursor(BraneFiroe target) {
        // Dereference search is always bounded to the target brane
        ReadOnlyBraneMemory memory = target.getBraneMemory();
        int size = memory.size();
        
        // Determine start index based on operator direction
        boolean forward = (operator == SearchOperator.REGEXP_FORWARD_LOCAL || operator == SearchOperator.REGEXP_FORWARD_GLOBAL);
        int startIndex = forward ? 0 : Math.max(0, size - 1);

        // Always braneBound=true for dereference
        return new SearchCursor(new FoolishCursor(target, startIndex), forward, true, true);
    }

    @Override
    protected FIR executeSearch(SearchCursor cursor) {
         return searcher.search(cursor).orElse(new MissingFiroe());
    }
    
    public String getPattern() {
        return searcher.getPattern();
    }
    
    @Override
    public String toString() {
        if (ast != null) return ast.toString();
        return searcher.getRawPattern();
    }

    @Override
    protected FIR cloneConstanic(FIR newParent, Optional<Nyes> targetNyes) {
        if (!isConstanic()) {
            throw new IllegalStateException(
                formatErrorMessage("cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
                    "but this FIR is in state: " + getNyes()));
        }

        if (isConstant()) {
            return this;
        }

        DerefSearchFiroe copy = new DerefSearchFiroe(this, newParent);

        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }

        return copy;
    }
    
    /**
     * Checks if the pattern contains any regex wildcards or special characters.
     */
    public static boolean isExactMatch(String pattern) {
        return RegexpSearcher.isExactMatch(pattern);
    }
}
