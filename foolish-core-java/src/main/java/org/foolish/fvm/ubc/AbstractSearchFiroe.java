package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.ast.SearchOperator;

/**
 * AbstractSearchFiroe implements the common logic for search operations:
 * - Managing the state transitions (INITIALIZED -> CONSTANT)
 * - Waiting for the anchor expression to be evaluated
 * - Unwrapping the anchor (Identifier -> Value, Assignment -> Result, etc.)
 * - Calling executeSearch() when ready
 */
public abstract class AbstractSearchFiroe extends FiroeWithBraneMind {
    protected final SearchOperator operator;
    protected FIR searchResult = null;
    protected FIR unwrapAnchor = null;
    protected boolean searchPerformed = false;

    protected AbstractSearchFiroe(AST.Expr ast, SearchOperator operator) {
        super(ast);
        this.operator = operator;
    }

    @Override
    protected void initialize() {
        setInitialized();
        // Derived classes must ensure they call enqueueExprs(anchor) during initialization
        // We can't do it here easily because getting the anchor depends on the specific AST type
    }

    @Override
    public void step() {
        switch (getNyes()) {
            case INITIALIZED -> {
                if (stepNonBranesUntilState(Nyes.REFERENCES_IDENTIFIED)) {
                    setNyes(Nyes.REFERENCES_IDENTIFIED);
                }
            }
            case REFERENCES_IDENTIFIED -> {
                if (stepNonBranesUntilState(Nyes.ALLOCATED)) {
                    setNyes(Nyes.ALLOCATED);
                }
            }
            case ALLOCATED -> {
                if (stepNonBranesUntilState(Nyes.RESOLVED)) {
                    setNyes(Nyes.RESOLVED);
                }
            }
            case RESOLVED -> {
                if (stepNonBranesUntilState(Nyes.CONSTANT)) {
                    if (isAnchorReady()) {
                        performSearchStep();
                        if (searchResult != null) {
                            setNyes(Nyes.CONSTANT);
                        }
                    }
                }
            }
            default -> super.step();
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

        // Unwrap identifier to get the actual value
        FIR resolvedAnchor = anchor;
        if (anchor instanceof IdentifierFiroe identifierFiroe) {
            if (identifierFiroe.value == null) return false;
            resolvedAnchor = identifierFiroe.value;
        }

        if (resolvedAnchor == null) return false;
        anchor = resolvedAnchor;

        // Unwrap assignment
        if (anchor instanceof AssignmentFiroe assignmentFiroe) {
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

        // Unwrapping Loop
        if (unwrapAnchor instanceof IdentifierFiroe identifierFiroe) {
            unwrapAnchor = identifierFiroe.value;
            if (unwrapAnchor == null) searchResult = new NKFiroe();
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
            // If result is null (shouldn't be if not NYE, but check anyway)
            if (unwrapAnchor == null) searchResult = new NKFiroe();
            return;
        }

        if (unwrapAnchor instanceof ConstanticFiroe constantic) {
            if (constantic.isNye()) {
                constantic.step();
                return;
            }
            unwrapAnchor = constantic.getResult();
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

             // If the search result itself needs unwrapping (e.g. it's an assignment or identifier found in the brane)
             // We need to unwrap it too.
             if (result instanceof AssignmentFiroe assignment) {
                 result = assignment.getResult();
                 if (result == null) result = new NKFiroe();
             }

             if (result instanceof IdentifierFiroe || result instanceof AssignmentFiroe || result instanceof AbstractSearchFiroe) {
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

    @Override
    public boolean isAbstract() {
        if (searchResult == null) return true;
        return searchResult.isAbstract();
    }

    @Override
    public long getValue() {
        if (searchResult == null) {
            throw new IllegalStateException("Search not yet evaluated");
        }
        return searchResult.getValue();
    }
}
