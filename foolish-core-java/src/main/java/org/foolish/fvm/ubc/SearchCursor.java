package org.foolish.fvm.ubc;

import org.apache.commons.lang3.tuple.Pair;
import java.util.stream.Stream;
import java.util.Spliterators;
import java.util.Spliterator;
import java.util.stream.StreamSupport;
import java.util.Iterator;

public final class SearchCursor {
    private final Cursor start;
    private final boolean forward;
    private final boolean inclusive;
    private final boolean braneBound;

    public SearchCursor(Cursor start, boolean forward, boolean inclusive, boolean braneBound) {
        this.start = start;
        this.forward = forward;
        this.inclusive = inclusive;
        this.braneBound = braneBound;
    }

    public Cursor getStart() {
        return start;
    }

    public Stream<Pair<Cursor, FIR>> streamCandidates() {
        return StreamSupport.stream(
            Spliterators.spliteratorUnknownSize(new CandidateIterator(), Spliterator.ORDERED),
            false);
    }

    private class CandidateIterator implements Iterator<Pair<Cursor, FIR>> {
        private Cursor current;
        private boolean hasNext;

        public CandidateIterator() {
            this.current = start;
            // logic to determine initial hasNext based on start position and direction
            // If inclusive: start check from 'current'.
            // If !inclusive: start check from next/prev depending on direction.
            // But Cursor is just a pointer.
            // If inclusive && forward: start at current.statementIndex
            // If !inclusive && forward: start at current.statementIndex + 1
            
            // Adjust start position if not inclusive
            if (!inclusive) {
                int nextIdx = current.statementIndex() + (forward ? 1 : -1);
                this.current = new FoolishCursor(current.brane(), nextIdx);
            }
            this.hasNext = isValid(this.current);
        }

        private boolean isValid(Cursor c) {
            // Check bounds in current brane
            if (c.statementIndex() < 0) return false;
            
            // Note: BraneMemory access
            if (c.statementIndex() >= c.brane().getBraneMemory().size()) return false;
            
            return true;
        }

        @Override
        public boolean hasNext() {
            return hasNext;
        }

        @Override
        public Pair<Cursor, FIR> next() {
            if (!hasNext) throw new java.util.NoSuchElementException();

            Cursor retCursor = current;
            FIR retFir = retCursor.brane().getMemoryItem(retCursor.statementIndex());
            Pair<Cursor, FIR> result = Pair.of(retCursor, retFir);

            // Advance
            int nextIdx = retCursor.statementIndex() + (forward ? 1 : -1);
            
            // Check bounds
            if (nextIdx < 0 || nextIdx >= retCursor.brane().getBraneMemory().size()) {
                 // End of this brane
                 // TODO: If not braneBound, move to parent?
                 // For now, implementing braneBound=true logic implicitly
                 // If !braneBound, we would set 'current' to parent's cursor?
                 hasNext = false; 
            } else {
                 current = new FoolishCursor(retCursor.brane(), nextIdx);
                 hasNext = true;
            }

            return result;
        }
    }
}
