package org.foolish.ubc;

import org.foolish.ast.AST;
import org.foolish.fvm.Env;
import org.foolish.fvm.Insoe;

/**
 * Unicellular Brane Computer (UBC).
 *
 * The UBC is a simple computational machine that has just enough capacity to hold
 * the AST of a brane and the ability to interpret and understand a single expression
 * at a time. It proceeds from the beginning of the brane to the end, evaluating and
 * creating new values.
 *
 * The UBC has two sources of information:
 * - Ancestral Brane (AB): The search context containing the parent brane's environment
 * - Immediate Brane (IB): The current context accumulated inside the UBC so far
 */
public class UnicelluarBraneComputer {
    private final BraneFiroe rootBrane;
    private final Env ancestralContext;
    private final Env immediateContext;

    /**
     * Creates a UBC with a Brane Insoe and AB context.
     * The corresponding BraneFiroe is created with only the AST attached.
     *
     * @param braneInsoe The Insoe containing the brane AST
     * @param ancestralContext The Ancestral Brane context (AB)
     */
    public UnicelluarBraneComputer(Insoe braneInsoe, Env ancestralContext) {
        if (braneInsoe == null) {
            throw new IllegalArgumentException("Brane Insoe cannot be null");
        }

        AST ast = braneInsoe.ast();
        if (!(ast instanceof AST.Brane)) {
            throw new IllegalArgumentException("Insoe must contain a Brane AST");
        }

        this.rootBrane = new BraneFiroe(ast);
        this.ancestralContext = ancestralContext != null ? ancestralContext : new Env();
        this.immediateContext = new Env(this.ancestralContext, 0);
    }

    /**
     * Creates a UBC with a Brane Insoe and no ancestral context.
     *
     * @param braneInsoe The Insoe containing the brane AST
     */
    public UnicelluarBraneComputer(Insoe braneInsoe) {
        this(braneInsoe, null);
    }

    /**
     * Takes a single evaluation step.
     * Steps forward from the braneMind until it's empty, at which time it returns false.
     *
     * @return true if more steps are needed, false if evaluation is complete
     */
    public boolean step() {
        if (!rootBrane.underevaluated()) {
            return false;
        }

        rootBrane.step();
        return rootBrane.underevaluated();
    }

    /**
     * Runs the UBC until evaluation is complete.
     *
     * @return the number of steps taken
     */
    public int runToCompletion() {
        int steps = 0;
        while (step()) {
            steps++;

            // Safety check to prevent infinite loops
            if (steps > 100000) {
                throw new RuntimeException("Evaluation exceeded maximum step count (possible infinite loop)");
            }
        }
        return steps;
    }

    /**
     * Returns true if the UBC has completed evaluation.
     */
    public boolean isComplete() {
        return !rootBrane.underevaluated();
    }

    /**
     * Returns the root BraneFiroe being evaluated.
     */
    public BraneFiroe getRootBrane() {
        return rootBrane;
    }

    /**
     * Returns the ancestral context (AB).
     */
    public Env getAncestralContext() {
        return ancestralContext;
    }

    /**
     * Returns the immediate context (IB).
     */
    public Env getImmedateContext() {
        return immediateContext;
    }

    /**
     * Gets the final environment after full evaluation.
     * This is the frozen Env representing the fully evaluated brane.
     *
     * @return the frozen environment, or null if evaluation is not complete
     */
    public Env getFinalEnvironment() {
        if (!isComplete()) {
            return null;
        }
        return rootBrane.getEnvironment();
    }
}
