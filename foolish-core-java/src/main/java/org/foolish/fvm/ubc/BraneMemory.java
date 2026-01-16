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
     *
     * @param blockedQueries The list of identifier queries that should be blocked
     */
    public void setBlockedIdentifiers(List<Query> blockedQueries) {
        this.blockedIdentifiers.clear();
        this.blockedIdentifiers.addAll(blockedQueries);
    }

    /**
     * Checks if a query is blocked from parent resolution.
     *
     * @param query The query to check
     * @return true if the query is blocked, false otherwise
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

    public Optional<Pair<Integer, FIR>> get(Query query, int fromLine) {
        for (int line = min(fromLine, memory.size() - 1); line >= 0; line--) {
            var lineMemory = memory.get(line);
            if (query.matches(lineMemory)) {
                return Optional.of(Pair.of(line, lineMemory));
            }
        }

        // Check if this query is blocked from parent resolution
        if (parent != null && !isBlocked(query)) {
            return parent.get(query, myPos.get());
        }

        return Optional.empty(); // Not found
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
