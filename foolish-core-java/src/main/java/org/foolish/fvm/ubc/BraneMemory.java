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
    private final List<QueryModification> queryModifications;

    public enum Modification { BLOCK, ALLOW }
    /**
     * Represents a modification to query resolution logic, such as blocking or allowing a query.
     * Supports Brane and Branes scope control.
     */
    public record QueryModification(Query query, Modification modification) {}

    public BraneMemory(BraneMemory parent) {
        this.parent = parent;
        this.memory = new ArrayList<>();
        this.queryModifications = new ArrayList<>();
    }

    public BraneMemory(BraneMemory parent, int myPos) {
        this(parent);
        setMyPos(myPos);
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

        if (parent != null && !isBlocked(query)) {
             return parent.get(query, myPos.get());
        }
        return Optional.empty(); // Not found
    }

    public void addQueryModification(Query query, Modification modification) {
        // Prepend rule to enforce priority.
        // Left-most detachment overrides right-side detachments.
        // Since processing happens Right-to-Left, we prepend each new rule to the list.
        // This ensures the Left-most (last processed) rule appears first in the list.
        queryModifications.add(0, new QueryModification(query, modification));
    }

    public void addBlocker(Query blocker) {
        addQueryModification(blocker, Modification.BLOCK);
    }

    private boolean isBlocked(Query query) {
        for (QueryModification mod : queryModifications) {
             if (mod.query.blocks(query)) {
                 return mod.modification == Modification.BLOCK;
             }
        }
        return false;
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
