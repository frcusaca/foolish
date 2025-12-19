package org.foolish.fvm.ubc;

import org.apache.commons.lang3.tuple.Pair;
import org.foolish.ast.AST;

import java.util.Optional;

/**
 * RegexpSearchFiroe performs a regular expression search on a brane.
 * It evaluates an anchor expression (expected to be a brane), then searches
 * within that brane's memory for an identifier matching the regexp pattern.
 *
 * Syntax: brane ? pattern or brane ?? pattern or brane ?* pattern
 *
 * The ? operator performs localized search (only within the brane, no parent search).
 * The ?? operator performs globalized search (cursor-based search upward through parents).
 * The ?* operator performs multi-search (returns a brane with all matching results).
 */
public class RegexpSearchFiroe extends FiroeWithBraneMind {
    /**
     * Specifies what kind of result a search should return.
     */
    public enum SearchResultType {
        /** Return the identifier name that matched (e.g., "alice") */
        NAME,
        /** Return the value bound to the identifier (e.g., the brane or number) */
        VALUE,
        /** Return contextual information about the match (line number, characterizations, etc.) */
        CONTEXT,
        /** Return the full assignment (identifier + value pair) */
        ASSIGNMENT
    }

    private final String operator;
    private final String pattern;
    private final SearchResultType resultType;
    FIR searchResult = null; // Package-private for chained searches; null means search incomplete
    private FIR unwrapAnchor = null; // Current anchor being unwrapped during search
    private boolean searchPerformed = false; // Track whether we've performed the actual brane search

    public RegexpSearchFiroe(AST.RegexpSearchExpr regexpSearch) {
        super(regexpSearch);
        this.operator = regexpSearch.operator();
        this.pattern = regexpSearch.pattern();
        // For now, all searches return VALUE (the actual value, not the name/assignment)
        // Future: operator mapping might determine result type (e.g., ?* for multi-results)
        this.resultType = SearchResultType.VALUE;
    }

    @Override
    protected void initialize() {
        setInitialized();
        // Enqueue the anchor expression to be evaluated first
        AST.RegexpSearchExpr searchExpr = (AST.RegexpSearchExpr) ast;
        AST.Expr anchorExpr = searchExpr.anchor();
        switch (anchorExpr) {
            case AST.Brane _, AST.Branes _, AST.RegexpSearchExpr _, AST.Identifier _:
                enqueueExprs(anchorExpr);
                break;
            default:
                throw new IllegalArgumentException("regular expression search must anchor onto a brane context.");
        }
    }

    @Override
    public void step() {
        switch (getNyes()) {
            case INITIALIZED -> {
                // Wait for anchor expression to be evaluated
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
                // Wait for anchor brane to be fully CONSTANT before starting search
                if (stepNonBranesUntilState(Nyes.CONSTANT)) {
                    // Check if the anchor is ready (unwrapping identifiers and assignments)
                    if (isAnchorReady()) {
                        // Perform one step of the search unwrapping
                        performSearchStep();
                        // Only advance to CONSTANT once search is complete (searchResult != null)
                        if (searchResult != null) {
                            setNyes(Nyes.CONSTANT);
                        }
                    }
                }
            }
            default -> super.step();
        }
    }

    private boolean stepNonBranesUntilState(Nyes targetState) {
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

    private boolean isAnchorReady() {
        if (braneMemory.isEmpty()) {
            return false;
        }

        // assert braneMemory.size() == 1 : "RegexpSearchFiroe expects exactly one anchor expression in braneMemory";
        FIR anchor = braneMemory.getLast();

        // Unwrap identifier to get the actual value
        anchor = switch (anchor) {  
            case IdentifierFiroe identifierFiroe -> {
                if (identifierFiroe.value == null) {
                    yield null; // Identifier not yet resolved
                }
                yield identifierFiroe.value;
            }
            default -> anchor;
        };

        if (anchor == null) {
            return false;
        }

        // Unwrap assignment to get the assigned value
        anchor = switch (anchor) {
            case AssignmentFiroe assignmentFiroe -> {
                // Check if the assignment has been fully evaluated
                if (assignmentFiroe.isNye()) {
                    // Assignment not yet complete, step it
                    assignmentFiroe.step();
                    yield null;
                }
                if (assignmentFiroe.getResult() == null) {
                    yield null; // Assignment not yet evaluated
                }
                yield assignmentFiroe.getResult();
            }
            default -> anchor;
        };

        if (anchor == null) {
            return false;
        }

        // Check if it's a chained RegexpSearchFiroe
        return switch (anchor) {
            case RegexpSearchFiroe regexpSearchFiroe -> {
                // Wait for the chained search to complete
                if (regexpSearchFiroe.isNye()) {
                    regexpSearchFiroe.step();
                    yield false;
                }
                yield true;
            }
            default -> true; // Anchor is ready
        };
    }

    /**
     * Perform one step of the search unwrapping process.
     * This method is called repeatedly while in RESOLVED state until searchResult != null.
     * Each call unwraps one layer or performs the brane search.
     */
    private void performSearchStep() {
        // Check which result type is requested (only on first call)
        if (unwrapAnchor == null && !searchPerformed) {
            switch (resultType) {
                case VALUE:
                    // Implemented below - return the actual value
                    break;
                case NAME:
                    throw new UnsupportedOperationException("Search result type NAME not yet implemented");
                case CONTEXT:
                    throw new UnsupportedOperationException("Search result type CONTEXT not yet implemented");
                case ASSIGNMENT:
                    throw new UnsupportedOperationException("Search result type ASSIGNMENT not yet implemented");
            }

            // Get the anchor brane from braneMemory (the evaluated anchor expression)
            if (braneMemory.isEmpty()) {
                searchResult = new NKFiroe(); // Not found
                return;
            }

            unwrapAnchor = braneMemory.getLast();
        }

        // If we've already found the result, nothing to do
        if (searchResult != null) {
            return;
        }

        // Unwrap one layer at a time
        // NOTE: Current FIR architecture has IdentifierFiroe.value point to AssignmentFiroe,
        // which then contains the actual value. This means we must unwrap both:
        //   brn (IdentifierFiroe) -> brn=... (AssignmentFiroe) -> {...} (BraneFiroe)
        switch (unwrapAnchor) {
            case IdentifierFiroe identifierFiroe:
                unwrapAnchor = identifierFiroe.value;
                if (unwrapAnchor == null) {
                    searchResult = new NKFiroe();
                }
                return; // Check the unwrapped value on next step

            case AssignmentFiroe assignmentFiroe:
                // Unwrap assignment - this is needed because identifiers point to assignments
                // Step the assignment if it's not yet evaluated
                if (assignmentFiroe.isNye()) {
                    assignmentFiroe.step();
                    return; // Wait for assignment to complete
                }

                unwrapAnchor = assignmentFiroe.getResult();
                if (unwrapAnchor == null) {
                    searchResult = new NKFiroe();
                }
                return; // Check the unwrapped value on next step

            case RegexpSearchFiroe regexpSearchFiroe:
                // Step the chained search if it's not complete
                if (regexpSearchFiroe.searchResult == null) {
                    regexpSearchFiroe.step();
                    return; // Wait for chained search to complete
                }
                unwrapAnchor = regexpSearchFiroe.searchResult;
                return; // Check the unwrapped value on next step

            case BraneFiroe braneFiroe:
                // If we've already performed a search, this brane is the result - return it
                if (searchPerformed) {
                    searchResult = braneFiroe;
                    return;
                }

                // Found a brane - perform the search
                Query.RegexpQuery query = new Query.RegexpQuery(pattern);
                BraneMemory targetMemory = braneFiroe.braneMemory;
                int searchFrom = targetMemory.size() - 1;

                Optional<Pair<Integer, FIR>> result = operator.equals("?")
                    ? targetMemory.getLocal(query, searchFrom)
                    : targetMemory.get(query, searchFrom);

                searchPerformed = true; // Mark that we've performed the search

                FIR foundValue = result.map(pair -> pair.getValue()).orElse(new NKFiroe());

                // For VALUE result type: Unwrap assignments to get the actual value
                // When we find "name=value" in a brane, we want to return the value, not the assignment
                if (foundValue instanceof AssignmentFiroe assignment) {
                    foundValue = assignment.getResult();
                    if (foundValue == null) {
                        foundValue = new NKFiroe();
                    }
                }

                // If the result is still a wrapper type (Identifier/Assignment/RegexpSearch),
                // put it back through the unwrapping loop to fully resolve it to a concrete value
                if (foundValue instanceof IdentifierFiroe || foundValue instanceof AssignmentFiroe || foundValue instanceof RegexpSearchFiroe) {
                    unwrapAnchor = foundValue;
                    return; // Continue unwrapping on next step
                }

                // Fully unwrapped - return the concrete value
                searchResult = foundValue;
                return;

            case NKFiroe _:
                // Searching ??? returns ???
                searchResult = new NKFiroe();
                return;

            default:
                // Can only search branes or ???
                searchResult = new NKFiroe();
                return;
        }
    }

    @Override
    public boolean isAbstract() {
        if (searchResult == null) {
            return true;
        }
        return searchResult.isAbstract();
    }

    @Override
    public long getValue() {
        if (searchResult == null) {
            throw new IllegalStateException("RegexpSearch not yet evaluated");
        }
        // searchResult is the FIR that was found, which should be fully evaluated by now
        return searchResult.getValue();
    }

    @Override
    public String toString() {
        AST.RegexpSearchExpr searchExpr = (AST.RegexpSearchExpr) ast;
        return searchExpr.toString();
    }
}
