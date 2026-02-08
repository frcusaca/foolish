package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.ast.SearchOperator;
import java.util.Optional;

public class RegexpSearchFiroe extends AbstractSearchFiroe implements Constanicable {
    private final RegexpSearcher searcher;

    public RegexpSearchFiroe(AST.RegexpSearchExpr searchExpr) {
        super(searchExpr, searchExpr.operator());
        this.searcher = new RegexpSearcher(searchExpr.pattern());
    }

    public RegexpSearchFiroe(Either<AST, String> source, String pattern, SearchOperator operator) {
        super(source, operator);
        this.searcher = new RegexpSearcher(pattern);
    }
    
    /**
     * Copy constructor for cloneConstanic.
     */
    protected RegexpSearchFiroe(RegexpSearchFiroe original, FIR newParent) {
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
        ReadOnlyBraneMemory memory = target.getBraneMemory();
        int size = memory.size();

        // Check if GLOBAL search (searches up) or LOCAL (bounded)
        boolean isGlobal = (operator == SearchOperator.REGEXP_GLOBAL || operator == SearchOperator.REGEXP_FORWARD_GLOBAL);
        boolean braneBound = !isGlobal;

        // Direction logic: FORWARD means start at 0, otherwise standard usually reverse
        boolean forward = (operator == SearchOperator.REGEXP_FORWARD_LOCAL || operator == SearchOperator.REGEXP_FORWARD_GLOBAL);
        
        int startIndex = forward ? 0 : Math.max(0, size - 1);
        
        return new SearchCursor(new FoolishCursor(target, startIndex), forward, true, braneBound);
    }

    @Override
    protected FIR executeSearch(SearchCursor cursor) {
         return searcher.search(cursor).orElse(new MissingFiroe());
    }
    
    @Override
    public String toString() {
        if (ast != null) return ast.toString();
        return searcher.getRawPattern();
    }
    
    public String getPattern() {
       return searcher.getPattern();
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
        
        RegexpSearchFiroe copy = new RegexpSearchFiroe(this, newParent);
        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }
        return copy;
    }
}
