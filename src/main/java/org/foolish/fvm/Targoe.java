package org.foolish.fvm;

import java.util.concurrent.atomic.AtomicInteger;

/**
 * TARGet Of Evaluation.  All executable or evaluatable nodes within the
 * virtual machine ultimately derive from this class.  Each instance carries a
 * {@link TargoeType} and a unique numeric identifier.
 */
public abstract class Targoe {
    private static final AtomicInteger COUNTER = new AtomicInteger();

    private final int numericId;
    private final TargoeType type;

    protected Targoe(TargoeType type) {
        this.type = type;
        this.numericId = COUNTER.incrementAndGet();
    }

    /**
     * @return the unique numeric id for this targoe instance
     */
    public int numericId() {
        return numericId;
    }

    /**
     * @return the enum based name/type for this targoe
     */
    public TargoeType name() {
        return type;
    }

    /**
     * Evaluate this targoe in the given environment returning both the result
     * of the evaluation and the potentially updated environment.
     */
    public abstract EvalResult execute(Environment env);
}

