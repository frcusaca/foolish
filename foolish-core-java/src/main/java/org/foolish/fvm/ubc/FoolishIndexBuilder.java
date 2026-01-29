package org.foolish.fvm.ubc;

import java.util.ArrayList;
import java.util.LinkedList;
import java.util.List;

public class FoolishIndexBuilder {
    private final LinkedList<Integer> indices = new LinkedList<>();
    private String separator = ",";

    public FoolishIndexBuilder append(int index) {
        indices.addLast(index);
        return this;
    }

    public FoolishIndexBuilder prepend(int index) {
        indices.addFirst(index);
        return this;
    }

    public FoolishIndexBuilder separator(String separator) {
        this.separator = separator;
        return this;
    }

    public FoolishIndex build() {
        // Create a copy of the list to ensure the FoolishIndex is truly immutable/independent
        return new FoolishIndex(new ArrayList<>(indices), separator);
    }
}
