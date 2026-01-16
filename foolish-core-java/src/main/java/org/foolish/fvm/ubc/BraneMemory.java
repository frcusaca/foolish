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

    // Detachment filter chain: list of detachment stages applied right-to-left
    // Each stage is a list of queries representing one detachment brane
    // Example: [d1][d2][d3]{B} → chain has 3 stages: [[d3 queries], [d2 queries], [d1 queries]]
    private final List<List<Query>> detachmentFilterChain;

    public BraneMemory(BraneMemory parent) {
        this.parent = parent;
        this.memory = new ArrayList<>();
        this.detachmentFilterChain = new ArrayList<>();
    }

    public BraneMemory(BraneMemory parent, int myPos) {
        this(parent);
        setMyPos(myPos);
    }

    /**
     * Sets the detachment filter chain for this brane.
     * <p>
     * <b>Filter Chain Semantics:</b> When searching for identifier `v` in `[d1][d2][d3]{B}`:
     * <ol>
     * <li>Search finds match in parent scope</li>
     * <li>Apply filter d3 (rightmost): does d3 block v?</li>
     * <li>Apply filter d2: does d2 override d3's decision?</li>
     * <li>Apply filter d1 (leftmost): d1 has final say</li>
     * <li>If undetached after all filters → use match</li>
     * <li>If detached → identifier is blocked</li>
     * </ol>
     * <p>
     * Each filter stage can:
     * <ul>
     * <li>Block the identifier (if it's in that stage's list)</li>
     * <li>Pass through (if identifier not mentioned in that stage)</li>
     * </ul>
     * <p>
     * IMPORTANT: Cannot merge filters unless they're identical exact matches.
     * The sequence must be preserved for correct semantics.
     * <p>
     * Blocking cascades to child branes - see {@link #get(Query, int)}.
     *
     * @param filterChain List of detachment stages (ordered left-to-right as they appear in code)
     */
    public void setDetachmentFilterChain(List<List<Query>> filterChain) {
        this.detachmentFilterChain.clear();
        this.detachmentFilterChain.addAll(filterChain);
    }

    /**
     * Legacy method for backward compatibility. Adds a single filter stage.
     * @deprecated Use {@link #setDetachmentFilterChain(List)} for proper chain support
     */
    @Deprecated
    public void setBlockedIdentifiers(List<Query> blockedQueries) {
        this.detachmentFilterChain.clear();
        this.detachmentFilterChain.add(new ArrayList<>(blockedQueries));
    }

    /**
     * Returns an unmodifiable view of the detachment filter chain.
     * Used for debugging and testing to verify detachment brane filtering.
     *
     * @return List of filter stages (each stage is a list of queries)
     */
    public List<List<Query>> getDetachmentFilterChain() {
        return detachmentFilterChain.stream()
                .map(List::copyOf)
                .toList();
    }

    /**
     * Checks if a query is blocked by the detachment filter chain.
     * <p>
     * <b>Filter Chain Evaluation (Right-to-Left):</b>
     * <p>
     * For chain `[d1][d2][d3]{B}` searching for `v`:
     * <ol>
     * <li>Check d3 (rightmost): if d3 blocks v, mark as MAYBE_BLOCKED</li>
     * <li>Check d2: if d2 blocks v, override to MAYBE_BLOCKED; if d2 doesn't mention v, keep previous state</li>
     * <li>Check d1 (leftmost): d1 has final say - if blocks v, result is BLOCKED; if doesn't mention, keep previous</li>
     * </ol>
     * <p>
     * Currently simplified: if ANY filter in chain blocks the identifier, it's blocked.
     * The leftmost filter that mentions it determines the default value (handled elsewhere).
     * <p>
     * IMPORTANT: Blocking cascades to all child branes. When a child brane searches upward,
     * any blocking at an ancestor level stops the search.
     *
     * @param query The query to check
     * @return true if the query is blocked after applying all filters, false otherwise
     */
    private boolean isBlocked(Query query) {
        // Apply filters right-to-left (reverse iteration through chain)
        // For now: if any filter blocks it, it's blocked
        // Future: track block/unblock decisions through the chain for p-branes
        for (int i = detachmentFilterChain.size() - 1; i >= 0; i--) {
            List<Query> filterStage = detachmentFilterChain.get(i);
            for (Query blocked : filterStage) {
                if (blocked instanceof Query.StrictlyMatchingQuery blockedMatch &&
                    query instanceof Query.StrictlyMatchingQuery queryMatch) {
                    if (blockedMatch.equals(queryMatch)) {
                        return true; // Blocked by this filter
                    }
                }
            }
        }
        return false; // Not blocked by any filter
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

    /**
     * Search for a query in this brane and its ancestors.
     * <p>
     * This implements backward/upward identifier resolution with detachment blocking.
     * The search proceeds as follows:
     * 1. Search locally in this brane's memory (backward from fromLine)
     * 2. If not found locally, check if the query is blocked at this level
     * 3. If not blocked, recursively search parent branes
     * <p>
     * The blocking check at step 2 ensures that detachment branes effectively
     * block identifier resolution for the entire brane subtree. When a child brane
     * searches for an identifier, it will traverse upward through parent branes,
     * and any blocking set at an ancestor level will stop the search.
     *
     * @param query The query to search for
     * @param fromLine The line number to search backward from (inclusive)
     * @return Optional containing (line number, FIR) if found, empty otherwise
     */
    public Optional<Pair<Integer, FIR>> get(Query query, int fromLine) {
        for (int line = min(fromLine, memory.size() - 1); line >= 0; line--) {
            var lineMemory = memory.get(line);
            if (query.matches(lineMemory)) {
                return Optional.of(Pair.of(line, lineMemory));
            }
        }

        // Check if this query is blocked from parent resolution at this level.
        // This ensures detachment blocking cascades to all child branes.
        if (parent != null && !isBlocked(query)) {
            return parent.get(query, myPos.get());
        }

        return Optional.empty(); // Not found or blocked
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
