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
                if (stepNonBranesUntilState(Nyes.PRIMED)) {
                    if (isAnchorReady()) {
                        performSearchStep();
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
                        
                        if (result.isNye()) {
                            result.step();
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

    protected void performSearchStep() {
        if (searchResult != null) return; // Already finished search

        if (unwrapAnchor == null) {
            if (isMemoryEmpty()) {
                // No anchor means empty set -> Not Found
                searchResult = Optional.empty();
                return;
            }
            unwrapAnchor = memoryGetLast();
        }

        if (unwrapAnchor instanceof IdentifierFiroe identifierFiroe) {
            if (identifierFiroe.isNye()) {
                identifierFiroe.step();
                return;
            }
            if (identifierFiroe.value == null) {
                if (identifierFiroe.atConstanic()) {
                    // Identifier is constanic but not resolved - propagate as search result
                    // This happens when the search finds a constanic identifier (e.g., forward ref)
                    searchResult = Optional.of(identifierFiroe);
                } else {
                    searchResult = Optional.empty();
                }
                return;
            }
            unwrapAnchor = identifierFiroe.value;
            return;
        }

        if (unwrapAnchor instanceof AssignmentFiroe assignmentFiroe) {
            if (assignmentFiroe.isNye()) {
                assignmentFiroe.step();
                return;
            }
            unwrapAnchor = assignmentFiroe.getResult();
            if (unwrapAnchor == null) {
                 searchResult = Optional.empty();
            }
            return;
        }

        if (unwrapAnchor instanceof AbstractSearchFiroe abstractSearchFiroe) {
            if (abstractSearchFiroe.isNye()) {
                abstractSearchFiroe.step();
                return;
            }
            unwrapAnchor = abstractSearchFiroe.getResult();
            if (unwrapAnchor == null) {
                searchResult = Optional.empty(); // Was new NKFiroe()
            }
            return;
        }

        if (unwrapAnchor instanceof UnanchoredSeekFiroe unanchoredSeekFiroe) {
            if (unanchoredSeekFiroe.isNye()) {
                unanchoredSeekFiroe.step();
                return;
            }
            unwrapAnchor = unanchoredSeekFiroe.getResult();
            if (unwrapAnchor == null) {
                searchResult = Optional.empty();
            }
            return;
        }

        if (unwrapAnchor instanceof BraneFiroe braneFiroe) {
             SearchCursor cursor = createCursor(braneFiroe);
             FIR result = executeSearch(cursor);

             if (result == null) {
                 // Should ideally trigger re-step? 
                 // Assuming executeSearch returns null if needed to wait? 
                 // Or does it return NK/null for not found?
                 // Existing code wrapped null in NKFiroe.
                 searchResult = Optional.empty();
                 return;
             }

             if (result instanceof AssignmentFiroe assignment) {
                 // Don't unwrap assignment result eagerly here, let next loop handle it?
                 // Wait, original logic did:
                 // result = assignment.getResult(); if null -> NK
                 
                 // If we return the assignment itself, the caller might use it as result.
                 // But wait, performSearchStep sets searchResult.
                 // If we set searchResult = Optional.of(assignment), the main loop will step it (if NYE).
                 // AbstractSearchFiroe.step() says: FIR result = searchResult.get(); if (result.isNye()) step();
                 // So we CAN return the AssignmentFiroe itself!
                 searchResult = Optional.of(result);
                 return;
             }
             
             // Same for UnanchoredSeekFiroe etc?
             // Original logic unwrapped anchors, but for the *result* of the search?
             // "if (result instanceof IdentifierFiroe ... unwrapAnchor = result; return;"
             
             if (result instanceof IdentifierFiroe || result instanceof AssignmentFiroe
                 || result instanceof AbstractSearchFiroe || result instanceof UnanchoredSeekFiroe) {
                 unwrapAnchor = result;
                 return;
             }

             searchResult = Optional.of(result);
             return;
        }

        if (unwrapAnchor instanceof NKFiroe) {
            searchResult = Optional.empty();
            return;
        }

        // Fallthrough: unwrapAnchor is the result (e.g. constant value)
        searchResult = Optional.of(unwrapAnchor);
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

    @Override
    protected abstract FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes);
}
