package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.ast.SearchOperator;

/**
 * AbstractSearchFiroe implements the common logic for search operations:
 * - Managing the state transitions (INITIALIZED -> CONSTANT)
 * - Waiting for the anchor expression to be evaluated
 * - Unwrapping the anchor (Identifier -> Value, Assignment -> Result, etc.)
 * - Calling executeSearch() when ready
 *
 * <h2>Search Semantics</h2>
 *
 * <h3>Anchored Searches (B^, B$, B#123, B/pattern, B?pattern, B.pattern)</h3>
 * <p>Anchored searches look within a specific brane only (do not traverse parent branes).</p>
 * <p>Anchored searches return ??? (NK) when:</p>
 * <ul>
 *   <li>The anchor brane is empty or the search fails</li>
 *   <li>The anchor itself evaluates to ???</li>
 *   <li>The anchor brane is CONSTANT but the identifier is not found</li>
 *   <li>Searching on non-brane values</li>
 * </ul>
 *
 * <p><b>IMPORTANT:</b> For anchored searches, the result can be CONSTANT ??? even when the anchor
 * brane B is still Constanic. This happens when the brane's own identifiers have been accumulated
 * (all idFIRs created) by the time the search is performed.</p>
 *
 * <p><b>Search Directions:</b></p>
 * <ul>
 *   <li>B/x - forward search (finds first match from start to end)</li>
 *   <li>B?x - backward search (finds last match from end to start)</li>
 *   <li>B.x - alias for B?x (behaves like object member access in Java/C++/Python)</li>
 * </ul>
 *
 * <p><b>Note:</b> Unanchored forward search /x is NOT in syntax (conflicts with division).
 * Find-all operators (B??x, B//x) are reserved for future implementation.</p>
 *
 * <h3>Level-Skipping Brane-Boundary Insensitive Searches (identifier references, ?pattern)</h3>
 * <p>Level-skipping searches traverse parent branes (unanchored searches).</p>
 * <p>Level-skipping searches use CONSTANIC when:</p>
 * <ul>
 *   <li>Identifier not found in any parent brane (CONSTANIC state signals missing attachment/detachment)</li>
 *   <li>Found value is itself CONSTANIC (wrapped in CMFir)</li>
 *   <li>Found value becomes CONSTANT (search completes normally)</li>
 * </ul>
 *
 * <p>See docs/search-semantics.md for detailed documentation.</p>
 *
 * <h3>TODO: Test Cases Needed</h3>
 * <ul>
 *   <li>Partial parameter specification: Test where we partially specify parameters in a brane
 *       and then query for their values while the brane is still Constanic. This should demonstrate
 *       that searches can return CONSTANT results even when the anchor brane is Constanic.</li>
 *   <li>Recursive search behavior: Searches that reference themselves may need special handling.</li>
 * </ul>
 */
public abstract class AbstractSearchFiroe extends FiroeWithBraneMind {
    protected final SearchOperator operator;
    protected FIR searchResult = null;
    protected FIR unwrapAnchor = null;
    protected boolean searchPerformed = false;
    protected boolean found = false;

    protected AbstractSearchFiroe(AST.Expr ast, SearchOperator operator) {
        super(ast);
        this.operator = operator;
    }

    protected void initialize() {
        setInitialized();
        // Derived classes must ensure they call enqueueExprs(anchor) during initialization
        // We can't do it here easily because getting the anchor depends on the specific AST type
    }

    public int step() {
        switch (getNyes()) {
            case INITIALIZED -> {
                if (stepNonBranesUntilState(Nyes.CHECKED)) {
                    setNyes(Nyes.CHECKED);
                }
                return 1;
            }
            case CHECKED -> {
                if (stepNonBranesUntilState(Nyes.CONSTANT)) {
                    if (isAnchorReady()) {
                        performSearchStep();
                        switch (searchResult) {
                            case null -> {
                                // Keep searching
                                return 1;
                            }
                            case NKFiroe nk -> {
                                // Search failed - result is NK (not found)
                                found = false;
                                setNyes(Nyes.CONSTANT);
                                return 1;
                            }
                            default -> {
                                // Search succeeded - step the result until it's fully evaluated
                                found = true;
                                if (searchResult.isNye()) {
                                    searchResult.step();
                                    return 1;
                                }
                                // Check if result is Constanic (not NK)
                                if (searchResult.atConstanic()) {
                                    setNyes(Nyes.CONSTANIC);
                                    return 1;
                                }
                                // Result is CONSTANT - we're done
                                setNyes(Nyes.CONSTANT);
                                return 1;
                            }
                        }
                    }
                }
                return 1;
            }
            default -> {
                return super.step();
            }
        }
    }

    protected boolean stepNonBranesUntilState(Nyes targetState) {
        if (braneMind.isEmpty()) {
            return true;
        }

        FIR current = braneMind.removeFirst();
        current.step();

        if (current.isNye()) {
            braneMind.addLast(current);
        }

        return current.getNyes().ordinal() >= targetState.ordinal();
    }

    protected boolean isAnchorReady() {
        if (braneMemory.isEmpty()) {
            return false;
        }

        FIR anchor = braneMemory.getLast();

        // Check if anchor is CONSTANIC
        if (anchor.atConstanic()) {
            return true;
        }

        // Unwrap identifier to get the actual value
        FIR resolvedAnchor = anchor;
        if (anchor instanceof IdentifierFiroe identifierFiroe) {
            if (identifierFiroe.atConstanic()) {
                return true;
            }
            if (identifierFiroe.value == null) return false;
            resolvedAnchor = identifierFiroe.value;
        }

        if (resolvedAnchor == null) return false;
        anchor = resolvedAnchor;

        // Unwrap assignment
        if (anchor instanceof AssignmentFiroe assignmentFiroe) {
            if (assignmentFiroe.atConstanic()) {
                return true;
            }
            if (assignmentFiroe.isNye()) {
                assignmentFiroe.step();
                return false;
            }
            if (assignmentFiroe.getResult() == null) return false;
            anchor = assignmentFiroe.getResult();
        }

        if (anchor == null) return false;

        // Check if chained search is ready
        if (anchor instanceof AbstractSearchFiroe abstractSearchFiroe) {
             if (abstractSearchFiroe.atConstanic()) {
                 return true;
             }
             if (abstractSearchFiroe.isNye()) {
                 abstractSearchFiroe.step();
                 return false;
             }
             return true;
        }

        return true;
    }

    protected void performSearchStep() {
        if (unwrapAnchor == null && !searchPerformed) {
            if (braneMemory.isEmpty()) {
                searchResult = new NKFiroe();
                return;
            }
            unwrapAnchor = braneMemory.getLast();
        }

        if (searchResult != null) return;

        // Check for constanic anchor
        if (unwrapAnchor.atConstanic()) {
            searchResult = new NKFiroe(); // Search on constanic -> NK? Or constanic result?
            // Usually if resource is missing, search fails -> NK.
            // Or maybe it should propagate constanic? "Refactor identifier 'NOT FOUND' state to CONSTANIC state"
            // But a search result that fails is usually NK.
            // Let's stick to NK for now unless specified otherwise.
            return;
        }

        // Unwrapping Loop
        if (unwrapAnchor instanceof IdentifierFiroe identifierFiroe) {
            if (identifierFiroe.atConstanic()) {
                searchResult = new NKFiroe();
                return;
            }
            unwrapAnchor = identifierFiroe.value;
            if (unwrapAnchor == null) searchResult = new NKFiroe();
            return;
        }

        if (unwrapAnchor instanceof AssignmentFiroe assignmentFiroe) {
            if (assignmentFiroe.atConstanic()) {
                searchResult = new NKFiroe();
                return;
            }
            if (assignmentFiroe.isNye()) {
                assignmentFiroe.step();
                return;
            }
            unwrapAnchor = assignmentFiroe.getResult();
            if (unwrapAnchor == null) searchResult = new NKFiroe();
            return;
        }

        if (unwrapAnchor instanceof AbstractSearchFiroe abstractSearchFiroe) {
            if (abstractSearchFiroe.atConstanic()) {
                searchResult = new NKFiroe();
                return;
            }
            if (abstractSearchFiroe.isNye()) {
                abstractSearchFiroe.step();
                return;
            }
            unwrapAnchor = abstractSearchFiroe.getResult();
            // If result is null (shouldn't be if not NYE, but check anyway)
            if (unwrapAnchor == null) searchResult = new NKFiroe();
            return;
        }

        if (unwrapAnchor instanceof UnanchoredSeekFiroe unanchoredSeekFiroe) {
            if (unanchoredSeekFiroe.atConstanic()) {
                searchResult = new NKFiroe();
                return;
            }
            if (unanchoredSeekFiroe.isNye()) {
                unanchoredSeekFiroe.step();
                return;
            }
            unwrapAnchor = unanchoredSeekFiroe.getResult();
            // If result is null (out of bounds), search fails
            if (unwrapAnchor == null) searchResult = new NKFiroe();
            return;
        }

        if (unwrapAnchor instanceof BraneFiroe braneFiroe) {
             if (searchPerformed) {
                 searchResult = braneFiroe;
                 return;
             }

             FIR result = executeSearch(braneFiroe);
             searchPerformed = true;

             if (result == null) {
                 // Should ideally return NKFiroe, but assume executeSearch handles logic
                 searchResult = new NKFiroe();
                 return;
             }

             if (result.atConstanic()) {
                 // If search result is constanic (e.g. identifier found but it was constanic)
                 // Then we treat it as found but constanic.
                 // But wait, executeSearch returns FIR.
                 // If we found an IdentifierFiroe that is Constanic, result is that IdentifierFiroe.
             }

             // If the search result itself needs unwrapping (e.g. it's an assignment or identifier found in the brane)
             // We need to unwrap it too.
             if (result instanceof AssignmentFiroe assignment) {
                 result = assignment.getResult();
                 if (result == null) result = new NKFiroe();
             }

             if (result instanceof IdentifierFiroe || result instanceof AssignmentFiroe || result instanceof AbstractSearchFiroe || result instanceof UnanchoredSeekFiroe) {
                 unwrapAnchor = result;
                 return;
             }

             searchResult = result;
             return;
        }

        if (unwrapAnchor instanceof NKFiroe) {
            searchResult = new NKFiroe();
            return;
        }

        // Default: If we have performed the search (or unwrapped a result) and reached a leaf value
        // that is not one of the wrapper types above, it IS the result.
        if (searchPerformed) {
            searchResult = unwrapAnchor;
        } else {
            // We are trying to search on something that isn't a brane
            searchResult = new NKFiroe();
        }
    }

    /**
     * Executes the specific search logic on the target brane.
     * Returns the found FIR or NKFiroe if not found.
     */
    protected abstract FIR executeSearch(BraneFiroe target);

    public FIR getResult() {
        return searchResult;
    }

    /**
     * Returns whether the search found a result.
     * A search is "found" if the result is not NKFiroe.
     *
     * Semantics:
     * - isFound() && CONSTANT: search found and result is fully evaluated
     * - isFound() && CONSTANIC: search found but result is unresolved
     * - !isFound() && CONSTANIC: search not found (only valid state for not found)
     * - !isFound() && CONSTANT: invalid - should not occur
     */
    public boolean isFound() {
        return found;
    }

    public long getValue() {
        if (searchResult == null) {
            throw new IllegalStateException("Search not yet evaluated");
        }
        return searchResult.getValue();
    }
}
