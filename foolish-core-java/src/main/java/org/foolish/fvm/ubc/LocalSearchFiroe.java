package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.ast.SearchOperator;
import java.util.Optional;

/**
 * LocalSearchFiroe represents a variable/identifier lookup in the current context.
 * It searches from the current brane outwards/upwards (unanchored search).
 */
public class LocalSearchFiroe extends AbstractSearchFiroe implements Constanicable {
    private final RegexpSearcher searcher;

    public LocalSearchFiroe(Either<AST, String> source) {
        super(source, SearchOperator.REGEXP_GLOBAL);
        String pattern = source.fold(a -> ((AST.Identifier)a).id(), s -> s);
        this.searcher = new RegexpSearcher(pattern);
    }
    
    /**
     * Copy constructor for cloneConstanic.
     */
    protected LocalSearchFiroe(LocalSearchFiroe original, FIR newParent) {
        super(original, newParent);
        this.searcher = original.searcher;
    }

    @Override
    protected void initialize() {
        super.initialize();
        if (unwrapAnchor == null) {
            // Implicit anchor is current brane
            this.unwrapAnchor = getMyBrane();
        }
    }

    @Override
    protected SearchCursor createCursor(BraneFiroe target) {
        // Unanchored search (Search Up)
        // Start from end of memory (most recent definition shadows earlier ones)
        ReadOnlyBraneMemory memory = target.getBraneMemory();
        int size = memory.size();
        int lastIndex = Math.max(0, size - 1);
        
        // braneBound=false allows traversing up to parent
        return new SearchCursor(new FoolishCursor(target, lastIndex), false, true, false);
    }

    @Override
    protected FIR executeSearch(SearchCursor cursor) {
        return searcher.search(cursor).orElse(new MissingFiroe());
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

        LocalSearchFiroe copy = new LocalSearchFiroe(this, newParent);
        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }
        return copy;
    }

    @Override
    public String toString() {
        // Use AST if available
        if (ast != null) return ast.toString();
        return searcher.getRawPattern();
    }
    
    public String getPattern() {
        return searcher.getPattern();
    }
}
