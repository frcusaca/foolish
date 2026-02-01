package org.foolish.fvm.ubc;

import java.util.ArrayList;
import java.util.LinkedList;


/**
 * Builder for constructing {@link FoolishIndex} instances.
 * <p>
 * <b>Usage:</b>
 * <pre>
 * // Build index [0, 1, 2] by prepending (walking up tree)
 * FoolishIndex index = new FoolishIndexBuilder()
 *     .prepend(2)  // deepest level first
 *     .prepend(1)
 *     .prepend(0)  // root level last
 *     .build();
 *
 * // Build index [0, 1, 2] by appending (walking down tree)
 * FoolishIndex index = new FoolishIndexBuilder()
 *     .append(0)   // root level first
 *     .append(1)
 *     .append(2)   // deepest level last
 *     .build();
 *
 * // Default separator is '#', producing seek-like paths:
 * // [0#1#2] - roughly a seek sequence from root
 *
 * // Custom separator
 * FoolishIndex index = new FoolishIndexBuilder()
 *     .separator(",")
 *     .append(0).append(1).append(2)
 *     .build();
 * // toString() returns "[0,1,2]"
 * </pre>
 * <p>
 * The builder uses a {@link LinkedList} internally to support efficient
 * prepend operations, which is the common case when walking up the FIR tree.
 *
 * @see FoolishIndex
 * @see FIR#getMyIndex()
 */
public class FoolishIndexBuilder {
    private final LinkedList<Integer> indices = new LinkedList<>();
    private String separator = "#";

    public FoolishIndexBuilder append(int index) {
        indices.addLast(index);
        return this;
    }

    public FoolishIndexBuilder prepend(int index) {
        indices.addFirst(index);
        return this;
    }

    public FoolishIndexBuilder separator(String separator) {
        this.separator = java.util.Objects.requireNonNull(separator);
        return this;
    }

    public FoolishIndex build() {
        // Create a copy of the list to ensure the FoolishIndex is truly immutable/independent
        return new FoolishIndex(new ArrayList<>(indices), separator);
    }
}
