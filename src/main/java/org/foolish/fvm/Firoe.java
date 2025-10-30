package org.foolish.fvm;

import java.util.ArrayList;
import java.util.List;

/**
 * Foolish Internal Representation Of Evaluation. Each {@code Firoe} wraps an underlying {@link Targoe}
 * and tracks progress toward a final {@link Finear} result on its own heap.
 */
public class Firoe implements Targoe {
    protected final List<Targoe> progress_heap;

    Firoe(Targoe base) {
        this.progress_heap = new ArrayList<>();
        if (base != null) {
            this.progress_heap.add(base);
        }
    }

    /** Creates an empty firoe with no base instruction. */
    public Firoe() {
        this(null);
    }

    public Targoe base() {
        return this.progress_heap.isEmpty() ? null : this.progress_heap.getFirst();
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
