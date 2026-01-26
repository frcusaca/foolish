package org.foolish.fvm.ubc;

import org.apache.commons.lang3.NotImplementedException;
import org.apache.commons.lang3.tuple.Pair;
import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.stream.Stream;

import static java.lang.Math.min;

public class BraneMemory implements Iterable<FIR> {
    private BraneMemory parent;
    private Optional<Integer> myPos = Optional.empty();
    private final List<FIR> memory;
    private final List<QueryModification> filters;

    public BraneMemory(BraneMemory parent) {
        this.parent = parent;
        this.memory = new ArrayList<>();
        this.filters = new ArrayList<>();
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

    public void addFilter(Query query, Modification type) {
        filters.add(new QueryModification(query, type));
    }

    /**
     * Copies content and filters from another memory.
     * Used for merging branes (concatenation).
     */
    public void copyFrom(BraneMemory other) {
        this.memory.addAll(other.memory);
        this.filters.addAll(other.filters);
    }

    public enum Modification {
        BLOCK, ALLOW
    }

    public record QueryModification(Query query, Modification modification) {}

    public FIR get(int idx) {
        if (idx >= 0 && idx < memory.size()) {
            return memory.get(idx);
        }
        throw new IndexOutOfBoundsException("Index: " + idx + ", Size: " + memory.size());
    }

    public Optional<Pair<Integer, FIR>> get(Query query, int fromLine) {
        // 1. Check local memory first
        for (int line = min(fromLine, memory.size() - 1); line >= 0; line--) {
            var lineMemory = memory.get(line);
            if (query.matches(lineMemory)) {
                return Optional.of(Pair.of(line, lineMemory));
            }
        }

        // 2. Check filters before delegating to parent
        Modification mod = Modification.ALLOW;
        boolean modified = false;

        for (QueryModification filter : filters) {
            // For now, only support StrictIdentifier matching
            if (query instanceof Query.StrictlyMatchingQuery idQuery && filter.query instanceof Query.StrictlyMatchingQuery filterIdQuery) {
                // If names match, apply filter
                if (idQuery.equals(filterIdQuery)) {
                    mod = filter.modification;
                    modified = true;
                    // Left-most overrides right-most (assuming filters added in order d1, d2...)
                    // "d1 (left-most) has the final say."
                    // If d1 added first, we found it first. Break.
                    break;
                }
            }
        }

        if (modified && mod == Modification.BLOCK) {
            return Optional.empty(); // Blocked, do not search parent
        }

        if (parent != null) {
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
