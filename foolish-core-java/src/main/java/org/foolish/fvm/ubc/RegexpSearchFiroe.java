package org.foolish.fvm.ubc;

import org.apache.commons.lang3.tuple.Pair;
import org.foolish.ast.AST;

import java.util.Optional;

/**
 * RegexpSearchFiroe performs a regular expression search on a brane.
 * It evaluates a base expression (expected to be a brane), then searches
 * within that brane's memory for an identifier matching the regexp pattern.
 *
 * Syntax: brane ? pattern or brane ?? pattern or brane ?* pattern
 *
 * The ? operator performs localized search (only within the brane, no parent search).
 * The ?? operator performs globalized search (cursor-based search upward through parents).
 * The ?* operator performs multi-search (returns a brane with all matching results).
 */
public class RegexpSearchFiroe extends FiroeWithBraneMind {
    private final String operator;
    private final String pattern;
    private FIR searchResult = null;

    public RegexpSearchFiroe(AST.RegexpSearchExpr regexpSearch) {
        super(regexpSearch);
        this.operator = regexpSearch.operator();
        this.pattern = regexpSearch.pattern();
    }

    @Override
    protected void initialize() {
        setInitialized();
        // Enqueue the base expression to be evaluated first
        AST.RegexpSearchExpr searchExpr = (AST.RegexpSearchExpr) ast;
        enqueueExprs(searchExpr.base());
    }

    @Override
    public void step() {
        switch (getNyes()) {
            case INITIALIZED -> {
                // Wait for base expression to be evaluated
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
                // Wait for base brane to be fully CONSTANT before searching
                if (stepNonBranesUntilState(Nyes.CONSTANT)) {
                    // Perform the search
                    performSearch();
                    setNyes(Nyes.CONSTANT);
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

    private void performSearch() {
        // Get the base brane from braneMind (the evaluated base expression)
        if (braneMemory.isEmpty()) {
            searchResult = new NKFiroe(); // Not found
            return;
        }

        FIR base = braneMemory.getLast();

        // Base must be a brane
        if (!(base instanceof BraneFiroe braneFiroe)) {
            searchResult = new NKFiroe(); // Can only search branes
            return;
        }

        // Create a regexp query
        Query.RegexpQuery query = new Query.RegexpQuery(pattern);

        // Search from the last line of the brane backward
        BraneMemory targetMemory = braneFiroe.braneMemory;
        int searchFrom = targetMemory.size() - 1;

        // Use getLocal() for the ? operator (localized search, no parent search)
        // Use get() for ?? operator (globalized search with parent search)
        Optional<Pair<Integer, FIR>> result = operator.equals("?")
            ? targetMemory.getLocal(query, searchFrom)
            : targetMemory.get(query, searchFrom);

        if (result.isPresent()) {
            searchResult = result.get().getValue();
        } else {
            searchResult = new NKFiroe(); // Not found
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
