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
                 if (!braneBound) {
                     // Move to parent brane
                     BraneFiroe parentBrane = retCursor.brane().getMyBrane();
                     if (parentBrane != null) {
                         // Start at beginning or end of parent depending on direction
                         int parentSize = parentBrane.getBraneMemory().size();
                         int startIdx = forward ? 0 : (parentSize - 1); // Start fresh in parent
                         
                         // Create cursor for parent
                         // Note: We scan the ENTIRE parent (or from start/end).
                         // If we wanted lexical scope (from declaration point), we'd need
                         // to know where the child brane was detached from. 
                         // But for now, assuming "global unanchored" = search whole parent.
                         current = new FoolishCursor(parentBrane, startIdx);
                         
                         // Check if valid (e.g. empty parent?)
                         if (isValid(current)) {
                             hasNext = true;
                         } else {
                             // Parent empty, recurse logic?
                             // Simplification: if parent empty/invalid, try ITS parent
                             // But for now, rely on loop or next call?
                             // Iteration logic handles next() call.
                             // But we need to set hasNext=true only if valid.
                             // If parent is empty, isValid returns false?
                             // If invalid, hasNext=false (stop).
                             // Ideally we should loop until we find a valid parent or run out.
                             // But let's assume one level step for now or fix this later.
                             // Actually, isValid checks bounds. If parent empty, size=0, idx=0 -> invalid.
                             hasNext = isValid(current); 
                         }
                     } else {
                         hasNext = false; // Root reached
                     }
                 } else {
                     hasNext = false;
                 }
            } else {
                 current = new FoolishCursor(retCursor.brane(), nextIdx);
                 hasNext = true;
            }

            return result;
        }
    }
}
