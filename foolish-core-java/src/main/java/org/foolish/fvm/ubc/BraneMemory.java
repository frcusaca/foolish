package org.foolish.fvm.ubc;

import org.apache.commons.lang3.tuple.Pair;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.stream.Stream;

import static java.lang.Math.min;
import static java.lang.Math.max;

public class BraneMemory implements ReadOnlyBraneMemory {
    private BraneMemory parent;
    private final List<FIR> memory;
    private FiroeWithBraneMind owningBrane = null; // The Firoe with braneMind that owns this memory

    public BraneMemory(BraneMemory parent) {
        this.parent = parent;
        this.memory = new ArrayList<>();
    }


    public BraneMemory getParent() {
        return parent;
    }

    public void setParent(BraneMemory parent) {
        this.parent = parent;
    }

    public FIR get(int idx) {
        if (idx >= 0 && idx < memory.size()) {
            return memory.get(idx);
        }
        throw new IndexOutOfBoundsException("Index: " + idx + ", Size: " + memory.size());
    }

    public Optional<Pair<Integer, FIR>> get(Query query, int fromLine) {
        for (int line = min(fromLine, memory.size() - 1); line >= 0; line--) {
            var lineMemory = memory.get(line);
            if (query.matches(lineMemory)) {
                // Check if we need to filter this result due to detachment
                if (shouldFilterMatch(query)) {
                    continue;  // Skip this match due to detachment filter
                }
                return Optional.of(Pair.of(line, lineMemory));
            }
        }
        if (parent != null) {
            // Compute position using owningBrane's getMyBraneStatementNumber()
            // which queries the parent's indexLookup for this brane's index.
            // Default to searching from end of parent if position cannot be determined.
            int parentPos = parent.size() - 1;  // Default fallback
            if (owningBrane != null) {
                int idx = owningBrane.getMyBraneStatementNumber();
                if (idx >= 0) {
                    parentPos = idx;
                }
            }
            return parent.get(query, parentPos);
        }
        return Optional.empty(); // Not found
    }

    /**
     * Checks if a match should be filtered due to detachment brane.
     * Walks up the parent chain looking for active detachment filters.
     *
     * @param query the query being searched for
     * @return true if this match should be filtered (skipped)
     */
    private boolean shouldFilterMatch(Query query) {
        String queryName = extractIdentifierName(query);
        if (queryName == null) {
            return false;  // Can't filter non-identifier queries
        }

        // Check this brane's owning brane for detachment filtering
        if (owningBrane instanceof DetachmentBraneFiroe detach) {
            if (detach.shouldFilter(queryName)) {
                return true;
            }
        }

        // Could also check parent branes, but for now just check local
        return false;
    }

    /**
     * Extracts the identifier name from a Query for detachment filtering.
     * Returns null if the query doesn't have a simple identifier name.
     */
    private String extractIdentifierName(Query query) {
        return switch (query) {
            case Query.StrictlyMatchingQuery smq -> smq.getId();
            case Query.RegexpQuery rq -> null;  // Regexp queries don't have simple names
        };
    }

    /**
     * Search for a query locally within this brane only, without searching parent branes.
     * Searches backward from fromLine to 0 (finds last match).
     * Used for localized backward regex search (? operator).
     */
    public Optional<Pair<Integer, FIR>> getLocal(Query query, int fromLine) {
        for (int line = min(fromLine, memory.size() - 1); line >= 0; line--) {
            var lineMemory = memory.get(line);
            if (query.matches(lineMemory)) {
                return Optional.of(Pair.of(line, lineMemory));
            }
        }
        return Optional.empty(); // Not found, don't search parents
    }

    /**
     * Search for a query locally within this brane only, without searching parent branes.
     * Searches forward from fromLine to end (finds first match).
     * Used for localized forward regex search (/ operator).
     */
    public Optional<Pair<Integer, FIR>> getLocalForward(Query query, int fromLine) {
        for (int line = max(fromLine, 0); line < memory.size(); line++) {
            var lineMemory = memory.get(line);
            if (query.matches(lineMemory)) {
                return Optional.of(Pair.of(line, lineMemory));
            }
        }
        return Optional.empty(); // Not found, don't search parents
    }

    @Override
    public Optional<Pair<Integer, FIR>> get(Cursor cursor, Query query) {
        return get(query, cursor.statementIndex());
    }

    @Override
    public Optional<Pair<Integer, FIR>> getLocal(Cursor cursor, Query query) {
        return getLocal(query, cursor.statementIndex());
    }

    @Override
    public Optional<Pair<Integer, FIR>> getLocalForward(Cursor cursor, Query query) {
        return getLocalForward(query, cursor.statementIndex());
    }

    public void put(FIR line) {
        memory.add(line);
    }

    public boolean isEmpty() {
        return memory.isEmpty();
    }

    public Stream<FIR> stream() {
        return memory.stream();
    }

    public int size() {
        return memory.size();
    }

    public FIR getLast() {
        if (memory.isEmpty()) {
            throw new java.util.NoSuchElementException("BraneMemory is empty");
        }
        return memory.get(memory.size() - 1);
    }

    public FIR removeFirst() {
        if (memory.isEmpty()) {
            throw new java.util.NoSuchElementException("BraneMemory is empty");
        }
        return memory.remove(0);
    }

    @Override
    public java.util.Iterator<FIR> iterator() {
        return memory.iterator();
    }

    /**
     * Sets the FiroeWithBraneMind that owns this BraneMemory.
     * Should only be called once, typically by BraneFiroe or DetachmentBraneFiroe during construction.
     */
    public void setOwningBrane(FiroeWithBraneMind brane) {
        if (this.owningBrane == null) {
            this.owningBrane = brane;
        } else if (this.owningBrane != brane) {
            throw new RuntimeException("Cannot reassign owning brane of BraneMemory.");
        }
        // If same brane, allow (idempotent)
    }

    /**
     * Gets the FiroeWithBraneMind that owns this BraneMemory.
     * Returns null if this is not a brane's memory (e.g., expression evaluation memory).
     */
    public FiroeWithBraneMind getOwningBrane() {
        return owningBrane;
    }

    /**
     * Gets the index of a FIR within this brane's queue.
     * Returns -1 if not found.
     *
     * @param fir the FIR to find
     * @return 0-based index, or -1 if not found
     */
    public int getStatementIndex(FIR fir) {
        for (int i = 0; i < memory.size(); i++) {
            if (memory.get(i) == fir) {
                return i;
            }
        }
        return -1;
    }
}
