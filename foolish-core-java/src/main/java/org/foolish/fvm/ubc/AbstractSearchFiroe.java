package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.ast.SearchOperator;

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
    protected FIR searchResult = null;
    protected FIR unwrapAnchor = null;
    protected boolean searchPerformed = false;
    protected boolean found = false;

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
        // Reset search state for re-evaluation
        this.searchResult = null;
        this.unwrapAnchor = null;
        this.searchPerformed = false;
        this.found = false;
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
                if (stepNonBranesUntilState(Nyes.CONSTANT)) {
                    if (isAnchorReady()) {
                        performSearchStep();
                        switch (searchResult) {
                            case null -> {
                                return 1;
                            }
                            case NKFiroe nk -> {
                                found = false;
                                setNyes(Nyes.CONSTANT);
                                return 1;
                            }
                            default -> {
                                found = true;
                                if (searchResult.isNye()) {
                                    searchResult.step();
                                    return 1;
                                }
                                if (searchResult.atConstanic()) {
                                    setNyes(Nyes.CONSTANIC);
                                    return 1;
                                }
                                setNyes(Nyes.CONSTANT);
                                return 1;
                            }
                        }
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

    protected boolean stepNonBranesUntilState(Nyes targetState) {
        if (isBrainEmpty()) {
            return true;
        }

        FIR current = brainDequeue();
        current.step();

        if (current.isNye()) {
            brainEnqueue(current);
        }

        return current.getNyes().ordinal() >= targetState.ordinal();
    }

    protected boolean isAnchorReady() {
        if (isMemoryEmpty()) {
            return false;
        }

        FIR anchor = memoryGetLast();

        if (anchor.atConstanic()) {
            return true;
        }

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
            if (isMemoryEmpty()) {
                searchResult = new NKFiroe();
                return;
            }
            unwrapAnchor = memoryGetLast();
        }

        if (searchResult != null) return;

        if (unwrapAnchor instanceof IdentifierFiroe identifierFiroe) {
            if (identifierFiroe.value == null) {
                if (identifierFiroe.atConstanic()) {
                    // Identifier is constanic but not resolved - propagate as search result
                    // This happens when the search finds a constanic identifier (e.g., forward ref)
                    searchResult = identifierFiroe;
                } else {
                    searchResult = new NKFiroe();
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
            if (unwrapAnchor == null) searchResult = new NKFiroe();
            return;
        }

        if (unwrapAnchor instanceof AbstractSearchFiroe abstractSearchFiroe) {
            if (abstractSearchFiroe.isNye()) {
                abstractSearchFiroe.step();
                return;
            }
            unwrapAnchor = abstractSearchFiroe.getResult();
            if (unwrapAnchor == null) searchResult = new NKFiroe();
            return;
        }

        if (unwrapAnchor instanceof UnanchoredSeekFiroe unanchoredSeekFiroe) {
            if (unanchoredSeekFiroe.isNye()) {
                unanchoredSeekFiroe.step();
                return;
            }
            unwrapAnchor = unanchoredSeekFiroe.getResult();
            if (unwrapAnchor == null) {
                searchResult = new NKFiroe();
            }
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
                 searchResult = new NKFiroe();
                 return;
             }

             if (result instanceof AssignmentFiroe assignment) {
                 result = assignment.getResult();
                 if (result == null) result = new NKFiroe();
             }

             if (result instanceof IdentifierFiroe || result instanceof AssignmentFiroe
                 || result instanceof AbstractSearchFiroe || result instanceof UnanchoredSeekFiroe) {
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

        if (searchPerformed) {
            searchResult = unwrapAnchor;
        } else {
            searchResult = new NKFiroe();
        }
    }

    protected abstract FIR executeSearch(BraneFiroe target);

    public FIR getResult() {
        return searchResult;
    }

    public boolean isFound() {
        return found;
    }

    public long getValue() {
        if (searchResult == null) {
            throw new IllegalStateException("Search not yet evaluated");
        }
        return searchResult.getValue();
    }

    @Override
    protected abstract FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes);
}
