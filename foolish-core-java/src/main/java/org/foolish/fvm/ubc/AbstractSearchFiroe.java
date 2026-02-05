package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.ast.SearchOperator;
import java.util.Optional;

/**
 * Common logic for search operations (head/tail, pattern matching, etc.).
 *
 * Anchored searches (B^, B$, B/pattern) look within a specific brane only.
 * Level-skipping searches traverse parent branes.
 *
 * Constanic branes can still have searchable constant elements (e.g., tail of {aa=⎵⎵; cc=9} is 9).
 */
public abstract class AbstractSearchFiroe extends FiroeWithBraneMind implements Constanicable {
    protected final SearchOperator operator;
    protected Optional<FIR> searchResult = null;
    protected FIR unwrapAnchor = null;

    protected AbstractSearchFiroe(AST.Expr ast, SearchOperator operator) {
        super(ast);
        this.operator = operator;
    }

    /**
     * Copy constructor for cloneConstanic.
     * Resets search state so the search can be re-executed in a new context.
     */
    protected AbstractSearchFiroe(AbstractSearchFiroe original, FIR newParent) {
        super(original, newParent);
        this.operator = original.operator;
        this.unwrapAnchor = null;
        
        // C4: If search resulted in "Not Found" (Empty), reset it to null so we try again.
        // If it was found (Present), we could conceptually keep it, but typically
        // constanic cloning implies re-evaluation might be needed or context changed.
        // However, per user request: reset ONLY if Optional.Empty.
        if (original.searchResult != null && original.searchResult.isEmpty()) {
            this.searchResult = null;
        } else {
             // If original was null (not run) or Present (found), keep it?
             // If original was null, new one is null.
             // If original was Present, we copy it? But wait, if Present, it might be
             // a specific FIR instance that needs cloning if we want to be safe?
             // Actually, usually we want to re-evaluate if we are being cloned into a new context.
             // But adhering strictly to "reset to null ONLY WHEN Optional.Empty":
             this.searchResult = original.searchResult;
        }
    }

    protected void initialize() {
        setInitialized();
    }

    public int step() {
        switch (getNyes()) {
            case UNINITIALIZED -> {
                initialize();
                setNyes(Nyes.INITIALIZED);
                return 1;
            }
            case INITIALIZED -> {
                prime();
                setNyes(Nyes.CHECKED);
                return 1;
            }
            case CHECKED -> {
                boolean nonBranesReady = stepNonBranesUntilState(Nyes.CONSTANIC);
                if (nonBranesReady) {
                    // Initialize unwrapAnchor if needed to check if it's a Brane
                    if (unwrapAnchor == null && !isMemoryEmpty()) {
                        unwrapAnchor = memoryGetLast();
                    }

                    // Special handling for Brane anchors which are skipped by stepNonBranesUntilState
                    // We must step them explicitly so they can progress (and step their children)
                    if (unwrapAnchor != null && isBrane(unwrapAnchor) && unwrapAnchor.isNye()) {
                         unwrapAnchor.step();
                         return 1;
                    }
                    
                    if (isAnchorReady()) {
                        int work = performSearchStep();
                        if (work == 0) {
                             // Waiting for dependency
                             return 0;
                        }
                        
                        if (searchResult == null) {
                            // Search not completed (internal stepping)
                            return 1;
                        }
                        
                        if (searchResult.isEmpty()) {
                            // Search completed and found nothing (or Constanic)
                            setNyes(Nyes.CONSTANIC); 
                            return 1;
                        }

                        // Search found a result
                        FIR result = searchResult.get();
                        
                        // Wait for result to be ready
                        if (result.isNye()) {
                             return 1; 
                        }
                        if (result.atConstanic()) {
                            setNyes(Nyes.CONSTANIC);
                            return 1;
                        }
                        setNyes(Nyes.CONSTANT);
                        return 1;
                    }
                }
                return 1;
            }
            case CONSTANIC, CONSTANT -> {
                return 0;
            }
            default -> {
                return super.step();
            }
        }
    }

    /**
     * Takes one step on one brane not yet at target state (by numerical value).
     * This method will not loop forever. But it is possible to call this method
     * forever due to children branes not progressing.
     * 
     * @returns true when all branes are at target state
     */
    protected boolean stepNonBranesUntilState(Nyes targetState) {
        if (isBraneEmpty()) {
            return true;
        }

        int size = braneSize();
        for(int i = 0; i < size; ++i){
            FIR current = braneDequeue();
            
            if (current.getNyes().ordinal() < targetState.ordinal()) {
                current.step();
                if (current.getNyes().ordinal() < targetState.ordinal()){
                    braneEnqueue(current);
                }
                return isBraneEmpty();
            }
        }

        return isBraneEmpty();
    }

    protected boolean isAnchorReady() {
        // stepNonBranesUntilState(PRIMED) ensures general readiness.
        // We just need to know if we have something to anchor to.
        return !isMemoryEmpty();
    }

    protected int performSearchStep() {
        if (searchResult != null) return 0; // Already finished search

        if (unwrapAnchor == null) {
            if (isMemoryEmpty()) {
                // No anchor means empty set -> Not Found
                searchResult = Optional.empty();
                return 1;
            }
            unwrapAnchor = memoryGetLast();
        }

        // Use valuableSelf() to resolve the anchor
        Optional<FIR> valuable = unwrapAnchor.valuableSelf();

        if (valuable == null) {
            // Anchor not ready (pre-PRIMED or evaluating)
            return 0; // Waiting
        }

        if (valuable.isEmpty()) {
            // Anchor is constanic/empty -> Search result is empty
            searchResult = Optional.empty();
            // By definition, valuableSelf() returns Empty when constanic.
            setNyes(Nyes.CONSTANIC); 
            return 1;
        }

        FIR resolvedAnchor = valuable.get();

        // If the resolved anchor is still something we need to process specifically
        // (like BraneFiroe for searching), handle it.
        // Note: valuableSelf() returns 'this' for BraneFiroe.

        if (resolvedAnchor instanceof BraneFiroe braneFiroe) {
             SearchCursor cursor = createCursor(braneFiroe);
             FIR result = executeSearch(cursor);

             if (result == null) {
                 // Search incomplete/failed
                 searchResult = Optional.empty();
                 return 1;
             }
             
             // Unwrap the result using recursive valuableSelf()
             Optional<FIR> val = result.valuableSelf();
             if (val == null) {
                 // Result depends on something not ready. Wait.
                 return 0; // Waiting
             }
             
             // If val is empty (e.g. Constanic/Unresolved), searchResult is empty.
             searchResult = val;
             return 1;
        }

        if (resolvedAnchor instanceof NKFiroe) {
            searchResult = Optional.empty();
            return 1;
        }

        // If we unwrapped something (e.g. Identifier -> Assignment), continue finding the anchor.
        if (resolvedAnchor != unwrapAnchor && resolvedAnchor != null) {
            unwrapAnchor = resolvedAnchor;
            return 1; // Progress made (unwrapped)
        }

        // Fallthrough: resolvedAnchor is the final result (e.g. constant value)
        searchResult = Optional.of(resolvedAnchor);
        return 1;
    }

    protected SearchCursor createCursor(BraneFiroe target) {
        ReadOnlyBraneMemory memory = target.getBraneMemory();
        int size = memory.size();

        // Handle empty memory gracefully
        int lastIndex = Math.max(0, size - 1);
        int firstIndex = 0;

        // Note: boolean defaults: inclusive=true, braneBound=true (for now)
        return switch (operator) {
            case HEAD -> new SearchCursor(new FoolishCursor(target, firstIndex), true, true, true);
            case TAIL -> new SearchCursor(new FoolishCursor(target, lastIndex), false, true, true);
            
            case REGEXP_LOCAL -> new SearchCursor(new FoolishCursor(target, lastIndex), false, true, true);
            case REGEXP_GLOBAL -> new SearchCursor(new FoolishCursor(target, lastIndex), false, true, true);
            case REGEXP_FORWARD_LOCAL -> new SearchCursor(new FoolishCursor(target, firstIndex), true, true, true);
            case REGEXP_FORWARD_GLOBAL -> new SearchCursor(new FoolishCursor(target, firstIndex), true, true, true);
            
            // Default fallbacks
            default -> new SearchCursor(new FoolishCursor(target, firstIndex), true, true, true);
        };
    }

    protected abstract FIR executeSearch(SearchCursor cursor);

    public FIR getResult() {
        if (searchResult == null) return null;
        return searchResult.orElse(null); // Or new NKFiroe()? Previously returned null if searchResult was null
    }
    
    public boolean isFound() {
        return searchResult != null && searchResult.isPresent();
    }

    public long getValue() {
        if (searchResult == null) {
            throw new IllegalStateException("Search not yet evaluated");
        }
        return searchResult.map(FIR::getValue).orElseThrow(() -> new IllegalStateException("Search result is empty/constanic"));
    }

    protected abstract FIR cloneConstanic(FIR newParent, Optional<Nyes> targetNyes);

    @Override
    public Optional<FIR> valuableSelf() {
        // GIGANTIC TODO: CHECK FOR CIRCULAR REFERENCE
        // Implementing simple recursion limit to prevent StackOverflow
        if (FIR.RECURSION_DEPTH.get() > 100) {
            return Optional.empty();
        }

        try {
            FIR.RECURSION_DEPTH.set(FIR.RECURSION_DEPTH.get() + 1);
            if (searchResult != null) {
                // Found a result (could be empty if not found/constanic)
                 if (searchResult.isEmpty()) {
                     return Optional.empty();
                 }
                 return searchResult.get().valuableSelf();
            }
            // Not ready/evaluated yet
            return null;
        } finally {
            FIR.RECURSION_DEPTH.set(FIR.RECURSION_DEPTH.get() - 1);
        }
    }
}
