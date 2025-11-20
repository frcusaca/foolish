package org.foolish.fvm.ubc;

import org.foolish.fvm.BraneMemoryInterface;

import java.util.regex.Pattern;

public class BraneMemory implements BraneMemoryInterface<BraneMemory.Query, FIR> {
    public interface Query{}
    final public class IdentifierQuery extends CharacterizedIdentifier implements Query {
        public IdentifierQuery(String name, String characterization) {
            super(name, characterization);
        }
    }
    final public class RegExpQuery implements Query {
        private final Pattern pattern;
        public RegExpQuery(String regex) {
            this.pattern = Pattern.compile(regex);
        }
    }

    BraneMemory parent;
    private final java.util.Map<Integer, java.util.Map<CharacterizedIdentifier, FIR>> memory;

    public BraneMemory(BraneMemory parent) {
        this.parent=parent;
        this.memory = new java.util.HashMap<>();
    }

    @Override
    public FIR get(Query id, int fromLine) {
        for (int line = fromLine; line >= 0; line--) {
            java.util.Map<CharacterizedIdentifier, FIR> lineMemory = memory.get(line);
            if (lineMemory != null && lineMemory.containsKey(id)) {
                return lineMemory.get(id);
            }
        }
        return null; // Not found
    }

    @Override
    public void put(Query id, FIR value, int byLine) {
        if (id instanceof CharacterizedIdentifier charId) {
            memory.computeIfAbsent(byLine, k -> new java.util.HashMap<>()).put(charId, value);
        } else {
            throw new IllegalArgumentException("Only IdentifierQuery is supported for put operations");
        }
    }}