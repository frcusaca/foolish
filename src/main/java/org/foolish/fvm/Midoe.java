package org.foolish.fvm;

import java.util.ArrayList;
import java.util.List;

/**
 * MIDdle Of Evaluation.  A container that holds the sequence of evaluation
 * stages for a computation.  Initially a {@code Midoe} contains a single
 * {@link Targoe} but more stages may be added as evaluation proceeds.
 */
public class Midoe extends Instruction {
    private final List<Targoe> stages = new ArrayList<>();

    public Midoe(Targoe initial) {
        super(TargoeType.MIDOE);
        stages.add(initial);
    }

    public void push(Targoe t) {
        stages.add(t);
    }

    public Targoe peek() {
        return stages.get(stages.size() - 1);
    }

    @Override
    public EvalResult execute(Environment env) {
        return peek().execute(env);
    }
}

