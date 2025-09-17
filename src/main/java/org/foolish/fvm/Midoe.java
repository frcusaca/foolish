package org.foolish.fvm;

import java.util.ArrayList;
import java.util.List;

/**
 * MIDdle Of Evaluation representation.
 * Holds a heap of {@link Targoe} where the first element is the original {@link Insoe}.
 */
public class Midoe implements Targoe {
    private final Insoe base;
    private final List<Targoe> heap = new ArrayList<>();

    public Midoe(Insoe base) {
        this.base = base;
        heap.add(base);
    }

    protected List<Targoe> heap() { return heap; }

    public Targoe top() { return heap.get(heap.size() - 1); }

    public boolean isFinal() { return top() instanceof Finer; }

    public boolean isUnknown() { return top() instanceof Unknown; }

    public Finer finalResult() {
        Targoe top = top();
        if (top instanceof Finer finer) {
            return finer;
        }
        throw new IllegalStateException("Midoe has not reached a final result");
    }

    public Finer evaluate(Environment env) {
        if (isFinal()) {
            return finalResult();
        }
        Targoe next = base.execute(env);
        Finer result = Evaluator.eval(next, env);
        heap.add(result);
        return result;
    }
}
