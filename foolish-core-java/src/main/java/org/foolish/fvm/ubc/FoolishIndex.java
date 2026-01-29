package org.foolish.fvm.ubc;

import java.util.Collections;
import java.util.List;
import java.util.stream.Collectors;

/**
 * Represents the hierarchical index of a statement in a Foolish program.
 * The index is a sequence of integers starting with 0 (root).
 * This class is immutable.
 */
public class FoolishIndex {
    private final List<Integer> indices;

    public FoolishIndex(List<Integer> indices) {
        this.indices = Collections.unmodifiableList(indices);
    }

    public List<Integer> getIndices() {
        return indices;
    }

    @Override
    public String toString() {
        // "The printing of index consists of characters [0-9] separated by characters that are not [0-9]."
        return indices.stream()
                .map(String::valueOf)
                .collect(Collectors.joining(","));
    }
}
