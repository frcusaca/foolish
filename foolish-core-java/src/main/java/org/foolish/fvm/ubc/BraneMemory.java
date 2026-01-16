package org.foolish.fvm.ubc;

import org.apache.commons.lang3.NotImplementedException;
import org.apache.commons.lang3.tuple.Pair;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.stream.Stream;

import static java.lang.Math.min;

public class BraneMemory implements Iterable<FIR> {
    private BraneMemory parent;
    private Optional<Integer> myPos = Optional.empty();
    private final List<FIR> memory;
    private final List<Query> blockedIdentifiers;

    public BraneMemory(BraneMemory parent) {
        this.parent = parent;
        this.memory = new ArrayList<>();
        this.blockedIdentifiers = new ArrayList<>();
    }

    public BraneMemory(BraneMemory parent, int myPos) {
        this(parent);
        setMyPos(myPos);
    }

    /**
     * Sets the list of identifiers that should be blocked from parent resolution.
     * This is used by detachment branes to prevent identifier resolution in parent scopes.
     * <p>
     * IMPORTANT: Blocking set at this level cascades to all child branes that search
     * upward through this brane. See {@link #get(Query, int)} for details on how
     * blocking propagates through the brane hierarchy.
     *
     * @param blockedQueries The list of identifier queries that should be blocked
     */
    public void setBlockedIdentifiers(List<Query> blockedQueries) {
        this.blockedIdentifiers.clear();
        this.blockedIdentifiers.addAll(blockedQueries);
    }

    /**
     * Returns an unmodifiable view of the blocked identifiers at this level.
     * Used for debugging and testing to verify detachment brane blocking.
     *
     * @return List of blocked queries at this brane level
     */
    public List<Query> getBlockedIdentifiers() {
        return List.copyOf(blockedIdentifiers);
    }

    /**
     * Checks if a query is blocked from parent resolution at this level.
     * <p>
     * IMPORTANT: Blocking cascades to all child branes. When a child brane searches upward
     * for an identifier, the search will traverse through parent branes. If any parent brane
     * has blocked that identifier, the search stops at that level and returns empty.
     * This ensures that detachment branes effectively block identifier resolution for
     * the entire brane subtree below them.
     *
     * @param query The query to check
     * @return true if the query is blocked at this level, false otherwise
     */
    private boolean isBlocked(Query query) {
        return blockedIdentifiers.stream().anyMatch(blocked -> {
            if (blocked instanceof StrictlyMatchingQuery blockedMatch &&
                query instanceof StrictlyMatchingQuery queryMatch) {
                return blockedMatch.equals(queryMatch);
            }
            return false;
        });
    }

    public void setMyPos(int pos) {
        if (myPos.isEmpty()) {
            this.myPos = Optional.of(pos);
        } else {
            throw new RuntimeException("Cannot recoordinate a BraneMemory.");
        }
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

    /**
     * Search for a query in this brane and its ancestors.
     * <p>
     * This implements backward/upward identifier resolution with detachment blocking.
     * The search proceeds as follows:
     * 1. Search locally in this brane's memory (backward from fromLine)
     * 2. If not found locally, check if the query is blocked at this level
     * 3. If not blocked, recursively search parent branes
     * <p>
     * The blocking check at step 2 ensures that detachment branes effectively
     * block identifier resolution for the entire brane subtree. When a child brane
     * searches for an identifier, it will traverse upward through parent branes,
     * and any blocking set at an ancestor level will stop the search.
     *
     * @param query The query to search for
     * @param fromLine The line number to search backward from (inclusive)
     * @return Optional containing (line number, FIR) if found, empty otherwise
     */
    public Optional<Pair<Integer, FIR>> get(Query query, int fromLine) {
        for (int line = min(fromLine, memory.size() - 1); line >= 0; line--) {
            var lineMemory = memory.get(line);
            if (query.matches(lineMemory)) {
                return Optional.of(Pair.of(line, lineMemory));
            }
        }

        // Check if this query is blocked from parent resolution at this level.
        // This ensures detachment blocking cascades to all child branes.
        if (parent != null && !isBlocked(query)) {
            return parent.get(query, myPos.get());
        }

        return Optional.empty(); // Not found or blocked
    }

    /**
     * Search for a query locally within this brane only, without searching parent branes.
     * Used for localized regex search (? operator).
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
}
