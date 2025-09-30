package org.foolish.fvm;

import java.util.ArrayList;
import java.util.List;

/**
 * MIDdle Of Evaluation. Each {@code Midoe} wraps an underlying {@link Targoe}
 * and tracks progress toward a final {@link Finear} result on its own heap.
 */
public class Midoe implements Targoe {
    protected final List<Targoe> progress_heap;

    Midoe(Targoe base) {
        this.progress_heap = new ArrayList<>();
        if (base != null) {
            this.progress_heap.add(base);
        }
    }

    /** Creates an empty midoe with no base instruction. */
    public Midoe() {
        this(null);
    }

    public Targoe base() {
        return this.progress_heap.getFirst();
    }

    /** @return {@code true} if the top of the heap is not a {@link Finear}. */
    public boolean isUnknown() {
        if (progress_heap.isEmpty()) return true;
        Targoe top = progress_heap.getLast();
        return !(top instanceof Finear);
    }

    public List<Targoe> progress_heap() {
        return progress_heap;
    }
}
