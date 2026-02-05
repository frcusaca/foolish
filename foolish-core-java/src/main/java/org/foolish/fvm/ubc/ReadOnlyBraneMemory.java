package org.foolish.fvm.ubc;

import org.apache.commons.lang3.tuple.Pair;
import java.util.Iterator;
import java.util.Optional;

/**
 * Read-only view of BraneMemory for external diagnostic access.
 * <p>
 * This interface exposes only query and iteration operations, preventing
 * external code from modifying the brane's memory structure.
 * <p>
 * <b>Usage:</b>
 * <pre>
 * ReadOnlyBraneMemory memory = fir.getBraneMemory();
 * int size = memory.size();
 * FIR first = memory.get(0);
 * for (FIR fir : memory) {
 *     // Read-only iteration
 * }
 * </pre>
 */
public interface ReadOnlyBraneMemory extends Iterable<FIR> {
    /**
     * Gets a FIR by index.
     * @param index the 0-based index
     * @return the FIR at the given index
     * @throws IndexOutOfBoundsException if index is out of range
     */
    FIR get(int index);

    /**
     * Gets the last FIR in memory.
     * @return the last FIR
     * @throws java.util.NoSuchElementException if memory is empty
     */
    FIR getLast();

    /**
     * Returns the number of FIRs in memory.
     */
    int size();

    /**
     * Checks if memory is empty.
     */
    boolean isEmpty();

    /**
     * Gets the parent BraneMemory (read-only view).
     * @return parent memory, or null if this is the root
     */
    ReadOnlyBraneMemory getParent();

    /**
     * Searches for an identifier matching the query.
     * @param query the search query
     * @param fromLine the starting line for the search
     * @return pair of (index, FIR) if found, empty otherwise
     */
    Optional<Pair<Integer, FIR>> get(Query query, int fromLine);

    /**
     * Gets the statement index of a FIR within this memory.
     * @param fir the FIR to find
     * @return the 0-based index, or -1 if not found
     */
    int getStatementIndex(FIR fir);

    /**
     * Returns an iterator over FIRs in memory.
     */
    @Override
    Iterator<FIR> iterator();

    /**
     * Searches locally (backward) within this brane only, without searching parent branes.
     * @param query the search query
     * @param fromLine the starting line for the search
     * @return pair of (index, FIR) if found, empty otherwise
     */
    Optional<Pair<Integer, FIR>> getLocal(Query query, int fromLine);

    /**
     * Searches locally (forward) within this brane only, without searching parent branes.
     * @param query the search query
     * @param fromLine the starting line for the search
     * @return pair of (index, FIR) if found, empty otherwise
     */
    Optional<Pair<Integer, FIR>> getLocalForward(Query query, int fromLine);

    /**
     * Searches for an identifier matching the query starting from the cursor position.
     * @param cursor the cursor indicating start position
     * @param query the search query
     * @return pair of (index, FIR) if found, empty otherwise
     */
    Optional<Pair<Integer, FIR>> get(Cursor cursor, Query query);

    /**
     * Searches locally (backward) starting from the cursor position.
     * @param cursor the cursor indicating start position
     * @param query the search query
     * @return pair of (index, FIR) if found, empty otherwise
     */
    Optional<Pair<Integer, FIR>> getLocal(Cursor cursor, Query query);

    /**
     * Searches locally (forward) starting from the cursor position.
     * @param cursor the cursor indicating start position
     * @param query the search query
     * @return pair of (index, FIR) if found, empty otherwise
     */
    Optional<Pair<Integer, FIR>> getLocalForward(Cursor cursor, Query query);
}
