package org.foolish.fvm.ubc;

import org.apache.commons.lang3.NotImplementedException;
import org.apache.commons.lang3.tuple.Pair;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.stream.Stream;

import static java.lang.Math.min;

public class BraneMemory {
    public sealed interface Query permits StrictlyMatchingQuery {
        public abstract boolean matches(FIR brane_line);
    }

    public static final class StrictlyMatchingQuery extends CharacterizedIdentifier implements Query {
        public StrictlyMatchingQuery(String name, String characterization) {
            super(name, characterization);
        }

        public boolean matches(FIR brane_line) {
            return switch (brane_line) {
                case AssignmentFiroe ass -> {
                    CharacterizedIdentifier lhs = ass.getLhs();
                    yield lhs.equals(this);
                }
                default ->
                    // mostly here is unamed lines in brane.
                        false;
            };
        }
    }

    BraneMemory parent;
    int myPos;
    private final List<FIR> memory;

    public BraneMemory(BraneMemory parent, int myPos) {
        this.parent = parent;
        this.myPos = myPos;
        this.memory = new ArrayList<>();
    }

    public Optional<FIR> get(int idx){
        if(idx>0 && idx<memory.size()){
            return Optional.of(memory.get(idx));
        }
        return Optional.empty();
    }
    public Optional<Pair<Integer, FIR>> get(Query query, int fromLine) {
        for (int line = min(fromLine, memory.size() - 1); line >= 0; line--) {
            var lineMemory = memory.get(line);
            if (query.matches(lineMemory)) {
                return Optional.of(Pair.of(line, lineMemory));
            }
        }
        if (parent != null) {
            return parent.get(query, myPos);
        }
        return Optional.empty(); // Not found
    }

    public void put(FIR line) {
        memory.add(line);
    }

    public boolean isEmpty() {
        return memory.isEmpty();
    }

    public Stream<FIR> stream(){
        return memory.stream();
    }
}