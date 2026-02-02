package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;

/**
 * ExecutionFir coordinates stepping multiple FIRs to a target Nyes state.
 * <p>
 * This is a reusable utility for breadth-first stepping of FIR collections
 * with specific milestone targeting. Use cases include:
 * <ul>
 *   <li>ConcatenationFiroe Stage A: Step identifiers/searches to PRIMED</li>
 *   <li>ConcatenationFiroe Stage C: Step joined branes to completion</li>
 *   <li>Any coordinated multi-FIR stepping pattern</li>
 * </ul>
 * <p>
 * <b>Usage:</b>
 * <pre>
 * ExecutionFir executor = ExecutionFir.stepping(fir1, fir2, fir3)
 *     .setParent(false)           // don't re-parent these FIRs
 *     .stepUntil(Nyes.PRIMED)     // target state for all FIRs
 *     .onComplete(firs -> {...})  // callback when all reach target
 *     .onStuck(firs -> {...})     // callback if any stuck at CONSTANIC < target
 *     .build();
 * </pre>
 * <p>
 * <b>State Transitions:</b>
 * <ul>
 *   <li>EVALUATING: Still stepping FIRs toward target</li>
 *   <li>CONSTANT: All FIRs reached target state → success</li>
 *   <li>CONSTANIC: Some FIRs stuck at CONSTANIC before reaching target → caller decides</li>
 * </ul>
 *
 * @see ConcatenationFiroe
 */
public class ExecutionFir extends FiroeWithBraneMind {

    private final Nyes targetState;
    private final boolean shouldSetParent;
    private final List<FIR> managedFirs;
    private final Consumer<List<FIR>> onComplete;
    private final Consumer<List<FIR>> onStuck;
    private boolean completed = false;
    private boolean stuck = false;

    /**
     * Private constructor - use Builder pattern via of().
     */
    private ExecutionFir(Builder builder) {
        super((AST) null, "ExecutionFir");
        this.targetState = builder.targetState;
        this.shouldSetParent = builder.setParent;
        this.managedFirs = new ArrayList<>(builder.firs);
        this.onComplete = builder.onComplete;
        this.onStuck = builder.onStuck;

        // Set parent if requested
        if (shouldSetParent) {
            for (FIR fir : managedFirs) {
                fir.setParentFir(this);
            }
        }
    }

    /**
     * Creates a new ExecutionFir Builder with the given FIRs.
     *
     * @param firs the FIRs to coordinate
     * @return a new Builder instance
     */
    public static Builder stepping(FIR... firs) {
        return new Builder(List.of(firs));
    }

    /**
     * Creates a new ExecutionFir Builder with the given FIR list.
     *
     * @param firs the FIRs to coordinate
     * @return a new Builder instance
     */
    public static Builder stepping(List<FIR> firs) {
        return new Builder(firs);
    }

    @Override
    protected void initialize() {
        setInitialized();
        // Add all FIRs that haven't reached target to braneMind for stepping
        for (FIR fir : managedFirs) {
            if (!hasReachedTarget(fir)) {
                brainEnqueue(fir);
            }
        }
    }

    @Override
    public int step() {
        switch (getNyes()) {
            case UNINITIALIZED -> {
                initialize();
                setNyes(Nyes.INITIALIZED);
                return 1;
            }
            case INITIALIZED, CHECKED, PRIMED -> {
                // Skip directly to EVALUATING
                setNyes(Nyes.EVALUATING);
                return 1;
            }
            case EVALUATING -> {
                return stepManagedFirs();
            }
            case CONSTANIC, CONSTANT -> {
                return 0;
            }
        }
        return 0;
    }

    /**
     * Steps one FIR from the queue toward the target state.
     * Uses breadth-first stepping: dequeue, step, re-enqueue if not done.
     */
    private int stepManagedFirs() {
        if (isBrainEmpty()) {
            // Check final state
            finalizeExecution();
            return 1;
        }

        FIR current = brainDequeue();

        // Check if already at target
        if (hasReachedTarget(current)) {
            // Don't re-enqueue, check if we're done
            if (isBrainEmpty()) {
                finalizeExecution();
            }
            return 1;
        }

        // Check if stuck at CONSTANIC before target
        if (isStuckBeforeTarget(current)) {
            // This FIR cannot progress further
            // Remove from queue (don't re-enqueue)
            if (isBrainEmpty()) {
                finalizeExecution();
            }
            return 1;
        }

        // Step the FIR
        current.step();

        // Re-enqueue if not at target and not stuck
        if (!hasReachedTarget(current) && !isStuckBeforeTarget(current)) {
            brainEnqueue(current);
        }

        // Check if we're done after this step
        if (isBrainEmpty()) {
            finalizeExecution();
        }

        return 1;
    }

    /**
     * Checks if a FIR has reached (or passed) the target state.
     */
    private boolean hasReachedTarget(FIR fir) {
        return fir.getNyes().ordinal() >= targetState.ordinal();
    }

    /**
     * Checks if a FIR is stuck at CONSTANIC before reaching target.
     * A FIR is stuck if it's at CONSTANIC but hasn't reached the target state.
     */
    private boolean isStuckBeforeTarget(FIR fir) {
        return fir.atConstanic() && !hasReachedTarget(fir);
    }

    /**
     * Finalizes execution - determines if completed successfully or stuck.
     */
    private void finalizeExecution() {
        // Check if any FIR is stuck at CONSTANIC before target
        boolean anyStuck = false;
        for (FIR fir : managedFirs) {
            if (isStuckBeforeTarget(fir)) {
                anyStuck = true;
                break;
            }
        }

        if (anyStuck) {
            stuck = true;
            setNyes(Nyes.CONSTANIC);
            if (onStuck != null) {
                onStuck.accept(managedFirs);
            }
        } else {
            completed = true;
            setNyes(Nyes.CONSTANT);
            if (onComplete != null) {
                onComplete.accept(managedFirs);
            }
        }
    }

    /**
     * Returns true if all managed FIRs have reached the target state.
     */
    public boolean isCompleted() {
        return completed;
    }

    /**
     * Returns true if some managed FIRs are stuck at CONSTANIC before target.
     */
    public boolean isStuck() {
        return stuck;
    }

    /**
     * Returns the target state this ExecutionFir is stepping toward.
     */
    public Nyes getTargetState() {
        return targetState;
    }

    /**
     * Returns the list of managed FIRs (read-only view).
     */
    public List<FIR> getManagedFirs() {
        return List.copyOf(managedFirs);
    }

    @Override
    protected FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes) {
        // ExecutionFir is a transient coordination object, not meant to be cloned
        throw new UnsupportedOperationException(
            "ExecutionFir is a transient coordination object and cannot be cloned");
    }

    /**
     * Builder for creating ExecutionFir instances with fluent API.
     */
    public static class Builder {
        private final List<FIR> firs;
        private Nyes targetState = Nyes.CONSTANT; // Default to full evaluation
        private boolean setParent = true;         // Default to set parent
        private Consumer<List<FIR>> onComplete = null;
        private Consumer<List<FIR>> onStuck = null;

        private Builder(List<FIR> firs) {
            this.firs = new ArrayList<>(firs);
        }

        /**
         * Sets the target state to step all FIRs toward.
         *
         * @param state the target Nyes state
         * @return this builder
         */
        public Builder stepUntil(Nyes state) {
            this.targetState = state;
            return this;
        }

        /**
         * Controls whether FIRs should be re-parented to the ExecutionFir.
         *
         * @param setParent true to set ExecutionFir as parent, false to leave parents unchanged
         * @return this builder
         */
        public Builder setParent(boolean setParent) {
            this.setParent = setParent;
            return this;
        }

        /**
         * Sets callback to invoke when all FIRs reach the target state.
         *
         * @param callback consumer that receives the list of FIRs
         * @return this builder
         */
        public Builder onComplete(Consumer<List<FIR>> callback) {
            this.onComplete = callback;
            return this;
        }

        /**
         * Sets callback to invoke when some FIRs are stuck at CONSTANIC before target.
         *
         * @param callback consumer that receives the list of FIRs
         * @return this builder
         */
        public Builder onStuck(Consumer<List<FIR>> callback) {
            this.onStuck = callback;
            return this;
        }

        /**
         * Builds and returns the configured ExecutionFir.
         *
         * @return the new ExecutionFir instance
         */
        public ExecutionFir build() {
            return new ExecutionFir(this);
        }
    }
}
