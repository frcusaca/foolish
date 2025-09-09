package org.foolish.fvm;

import java.util.ArrayList;
import java.util.List;

/**
 * MIDdle Of Evaluation.  Wraps an {@link Insoe} and tracks intermediate
 * results on a heap.  The bottom of the heap always contains the original
 * {@code Insoe}.  When the top of the heap is not a {@link Finear}, the midoe is
 * considered unknown for the purposes of evaluation.
 */
public class Midoe implements Targoe {
    private final List<Targoe> progress_heap;

    /** Creates an empty midoe with no base instruction. */
    public Midoe() {
        this.progress_heap = new ArrayList<>();
    }

    public Midoe(Insoe base) {
        this();
        progress_heap.add(base);
    }

    /**
     * Evaluates this midoe within the given environment.  The result is pushed
     * onto the heap and returned.
     */
    public Finear evaluate(Environment env) {
        return MidoeVm.evaluate(this, env);
    }

    /**
     * @return {@code true} if the top of the heap is not a {@link Finear}.
     */
    public boolean isUnknown() {
        if (progress_heap.isEmpty()) return true;
        Targoe top = progress_heap.get(progress_heap.size() - 1);
        return !(top instanceof Finear);
    }

    public List<Targoe> progress_heap() {
        return progress_heap;
    }
}
