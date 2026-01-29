package org.foolish.fvm.ubc;

import java.util.ArrayList;
import java.util.List;

/**
 * Builder for constructing immutable FoolishIndex instances.
 */
public class FoolishIndexBuilder {
    private final List<Integer> indices = new ArrayList<>();

    public FoolishIndexBuilder add(int index) {
        indices.add(index);
        return this;
    }

    public FoolishIndexBuilder prepend(int index) {
        indices.add(0, index);
        return this;
    }

    public FoolishIndex build() {
        return new FoolishIndex(new ArrayList<>(indices));
    }
}
