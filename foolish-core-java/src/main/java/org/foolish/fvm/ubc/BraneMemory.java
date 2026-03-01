package org.foolish.fvm.ubc;

import org.apache.commons.lang3.tuple.Pair;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.stream.Stream;

import static java.lang.Math.min;
import static java.lang.Math.max;

/**
 * BraneMemory: append-only storage for FIRs within a brane.
 * <p>
 * Supports backward search from a position with parent chain traversal for identifier resolution.
 * See {@code projects/FIR-Invariances.md#C7: BraneMemory Persistence}
 */
public class BraneMemory implements ReadOnlyBraneMemory {
    private FiroeWithBraneMind parentBrane;
    private final List<FIR> memory;
    private FiroeWithBraneMind owningBrane = null;

    public BraneMemory(FiroeWithBraneMind parentBrane) {
        this.parentBrane = parentBrane;
        this.memory = new ArrayList<>();
    }

    public FiroeWithBraneMind getParentBrane() {
        return parentBrane;
    }

    public void setParentBrane(FiroeWithBraneMind parentBrane) {
        this.parentBrane = parentBrane;
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
                if (shouldFilterMatch(query)) {
                    continue;
                }
                return Optional.of(Pair.of(line, lineMemory));
            }
        }
        if (parentBrane != null) {
            int parentPos = parentBrane.memorySize() - 1;
            if (owningBrane != null) {
                int idx = owningBrane.getMyBraneStatementNumber();
                if (idx >= 0) {
                    parentPos = idx;
                }
            }
            return parentBrane.memoryGet(query, parentPos);
        }
        return Optional.empty();
    }

    private boolean shouldFilterMatch(Query query) {
        String queryName = extractIdentifierName(query);
        if (queryName == null) {
            return false;
        }
        if (owningBrane instanceof DetachmentBraneFiroe detach) {
            if (detach.shouldFilter(queryName)) {
                return true;
            }
        }
        return false;
    }

    private String extractIdentifierName(Query query) {
        return switch (query) {
            case Query.StrictlyMatchingQuery smq -> smq.getId();
            case Query.RegexpQuery rq -> null;
        };
    }

    public Optional<Pair<Integer, FIR>> getLocal(Query query, int fromLine) {
        for (int line = min(fromLine, memory.size() - 1); line >= 0; line--) {
            var lineMemory = memory.get(line);
            if (query.matches(lineMemory)) {
                return Optional.of(Pair.of(line, lineMemory));
            }
        }
        return Optional.empty();
    }

    public Optional<Pair<Integer, FIR>> getLocalForward(Query query, int fromLine) {
        for (int line = max(fromLine, 0); line < memory.size(); line++) {
            var lineMemory = memory.get(line);
            if (query.matches(lineMemory)) {
                return Optional.of(Pair.of(line, lineMemory));
            }
        }
        return Optional.empty();
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

    public void setOwningBrane(FiroeWithBraneMind brane) {
        if (this.owningBrane == null) {
            this.owningBrane = brane;
        } else if (this.owningBrane != brane) {
            throw new RuntimeException("Cannot reassign owning brane of BraneMemory.");
        }
    }

    public FiroeWithBraneMind getOwningBrane() {
        return owningBrane;
    }

    public int getStatementIndex(FIR fir) {
        for (int i = 0; i < memory.size(); i++) {
            if (memory.get(i) == fir) {
                return i;
            }
        }
        return -1;
    }
}
