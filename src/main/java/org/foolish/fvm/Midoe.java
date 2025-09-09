package org.foolish.fvm;

import java.util.ArrayList;
import java.util.List;

/**
 * MIDdle Of Evaluation.  Wraps an {@link Insoe} and tracks intermediate
 * results on a heap.  The bottom of the heap always contains the original
 * {@code Insoe}.  When the top of the heap is not a {@link Finer}, the midoe is
 * considered unknown for the purposes of evaluation.
 */
public class Midoe implements Targoe {
    private final List<Targoe> heap = new ArrayList<>();

    public Midoe(Insoe base) {
        heap.add(base);
    }

    /**
     * Evaluates this midoe within the given environment.  The result is pushed
     * onto the heap and returned.
     */
    public Finer evaluate(Environment env) {
        Targoe top = heap.get(heap.size() - 1);
        if (top instanceof Finer finer) {
            return finer;
        }
        if (top instanceof Insoe insoe) {
            Finer result = insoe.execute(env);
            heap.add(result);
            return result;
        }
        if (top instanceof Midoe m) {
            Finer result = m.evaluate(env);
            heap.add(result);
            return result;
        }
        return Finer.UNKNOWN;
    }

    /** Convenience helper for evaluating arbitrary {@link Targoe} values. */
    public static Finer evaluate(Targoe targoe, Environment env) {
        if (targoe instanceof Finer f) {
            return f;
        }
        if (targoe instanceof Midoe m) {
            return m.evaluate(env);
        }
        if (targoe instanceof Insoe i) {
            return new Midoe(i).evaluate(env);
        }
        return Finer.UNKNOWN;
    }

    /**
     * @return {@code true} if the top of the heap is not a {@link Finer}.
     */
    public boolean isUnknown() {
        Targoe top = heap.get(heap.size() - 1);
        return !(top instanceof Finer);
    }

    public List<Targoe> heap() {
        return heap;
    }
}
