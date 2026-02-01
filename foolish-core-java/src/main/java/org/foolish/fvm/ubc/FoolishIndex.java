package org.foolish.fvm.ubc;

import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

/**
 * Multi-level hierarchical index for coordinate access in brane trees.
 * <p>
 * A FoolishIndex represents a path from the root brane to a specific FIR.
 * <p>
 * <b>Format:</b> {@code [i0, i1, i2, ..., iN]}
 * <ul>
 *   <li>i0: Always 0 (root brane marker)</li>
 *   <li>i1: Statement index in root brane (0-based)</li>
 *   <li>i2: Statement index in nested brane (if applicable)</li>
 *   <li>... continues for deeper nesting</li>
 * </ul>
 * <p>
 * <b>Example:</b>
 * <pre>
 * {                        // Root brane
 *   a = 1;                 // Index: [0, 0]
 *   b = {                  // Index: [0, 1] - outer assignment
 *     c = 5;               // Index: [0, 1, 0]
 *     d = {                // Index: [0, 1, 1] - inner brane
 *       e = 10;            // Index: [0, 1, 1, 0]
 *     };
 *   };
 * }
 * </pre>
 * <p>
 * <b>Usage:</b>
 * <ul>
 *   <li>{@link FIR#getMyIndex()} returns the FoolishIndex for any FIR in the tree</li>
 *   <li>Unique identification of any statement in a brane tree</li>
 *   <li>Debugging and error reporting (see {@link org.foolish.fvm.AlarmSystem})</li>
 *   <li>Correlation between error messages and source locations</li>
 * </ul>
 * <p>
 * <b>Separator:</b> Default is '#': {@code [0#1#2]}, roughly producing a seek
 * sequence that would arrive at the referenced element from root.
 * Custom separators supported via {@link FoolishIndexBuilder#separator(String)}.
 * <p>
 * <b>Equality:</b> Two FoolishIndex objects are equal if their index lists are equal,
 * regardless of separator.
 *
 * @see FoolishIndexBuilder
 * @see FIR#getMyIndex()
 */
public class FoolishIndex {
    private final List<Integer> indices;
    private final String separator;

    public FoolishIndex(List<Integer> indices, String separator) {
        this.indices = List.copyOf(Objects.requireNonNull(indices));
        this.separator = Objects.requireNonNull(separator);
    }

    public List<Integer> getIndices() {
        return indices;
    }

    @Override
    public String toString() {
        return indices.stream()
                .map(Object::toString)
                .collect(Collectors.joining(separator, "[", "]"));
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        FoolishIndex that = (FoolishIndex) o;
        return Objects.equals(indices, that.indices);
    }

    @Override
    public int hashCode() {
        return Objects.hash(indices);
    }
}
