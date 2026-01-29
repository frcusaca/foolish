package org.foolish.fvm.ubc;

import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

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
