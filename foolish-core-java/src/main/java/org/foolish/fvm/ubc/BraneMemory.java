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
    private final List<Rule> rules;

    public enum Action { BLOCK, ALLOW }
    public record Rule(Query query, Action action) {}

    public BraneMemory(BraneMemory parent) {
        this.parent = parent;
        this.memory = new ArrayList<>();
        this.rules = new ArrayList<>();
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

    public void addRule(Query query, Action action) {
        // Prepend rule to enforce priority (last added wins? No, we iterate R-to-L and prepend)
        // If createFiroeFromBranes iterates R-to-L and calls addRule (which should be prepend):
        // List: [LastAdded, ... FirstAdded].
        // Lookup iterates 0..N.
        // So LastAdded wins.
        // Correct for "Left overrides Right" if we iterate R-to-L.
        // Wait, if R-to-L: {x} [b] [a].
        // 1. Process [a]. Add `a`.
        // 2. Process [b]. Add `b`.
        // List: [b, a].
        // Lookup finds `b` first.
        // So Left (`b`) overrides Right (`a`). Correct.
        rules.add(0, new Rule(query, action));
    }

    public void addBlocker(Query blocker) {
        addRule(blocker, Action.BLOCK);
    }

    private boolean isBlocked(Query query) {
        for (Rule rule : rules) {
             if (rule.query.blocks(query)) {
                 return rule.action == Action.BLOCK;
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
