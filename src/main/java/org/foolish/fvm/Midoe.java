package org.foolish.fvm;

import java.util.ArrayList;
import java.util.List;

/**
 * MIDdle Of Evaluation. Each {@code Midoe} wraps an underlying {@link Targoe}
 * and tracks progress toward a final {@link Finear} result on its own heap.
 */
public class Midoe implements Targoe {
    private final Targoe base;
    private final List<Targoe> progress_heap;

    Midoe(Targoe base) {
        this.base = base;
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
        return base;
    }

    /** Evaluates this midoe within the given environment. */
    public Finear evaluate(Environment env) {
        return FinearVm.evaluate(this, env);
    }

    /** @return {@code true} if the top of the heap is not a {@link Finear}. */
    public boolean isUnknown() {
        if (progress_heap.isEmpty()) return true;
        Targoe top = progress_heap.get(progress_heap.size() - 1);
        return !(top instanceof Finear);
    }

    public List<Targoe> progress_heap() {
        return progress_heap;
    }
}
