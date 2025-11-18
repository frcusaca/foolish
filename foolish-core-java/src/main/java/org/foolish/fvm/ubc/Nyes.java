package org.foolish.fvm.ubc;

/**
 * NYE STATES (say "NICE") - represents the evaluation stages of a Firoe.
 * <p>
 * This enum tracks the progression of a FIR from initial construction through
 * complete evaluation. Each state represents a distinct phase in the evaluation
 * lifecycle, with transitions managed through the {@link FIR#setNyes(Nyes)} method.
 * <p>
 * The evaluation flow typically progresses as follows:
 * UNINITIALIZED → INITIALIZED → REFERENCES_IDENTIFIED → ALLOCATED → RESOLVED → EVALUATING → CONSTANT
 * <p>
 * Only CONSTANT represents a fully evaluated state where {@code isNye()} returns false.
 */
public enum Nyes {
    /**
     * Just an AST - this is where the constructor leaves the object.
     * No initialization has occurred yet.
     */
    UNINITIALIZED,

    /**
     * Various misc items initialized including cache for final value.
     * Transition taken care of by step().
     */
    INITIALIZED,

    /**
     * All referenced identifiers collected (not including sub-branes).
     * For non-brane expressions: step() until it reaches RESOLVED state.
     * For branes: step() until it reaches REFERENCES_IDENTIFIED state.
     * Transition taken care of by step().
     */
    REFERENCES_IDENTIFIED,

    /**
     * AB (Abstract Brane), IB (Implementation Brane) established firmly.
     * Transition taken care of by step().
     */
    ALLOCATED,

    /**
     * All variables for an expression are resolved.
     * For non-brane expressions: step() until it reaches RESOLVED state.
     * For branes: identifiers that can be resolved are resolved.
     * Branes should step until non-brane expressions are resolved.
     * For now, stub this branch for other expressions and return ???.
     * Transition taken care of by step().
     */
    RESOLVED,

    /**
     * Take a step() as we do currently, including branes.
     * Active evaluation is in progress.
     * Transition taken care of by step().
     */
    EVALUATING,

    /**
     * No more changes will happen with call to step() unless the environment changes.
     * This is the only !isNye() state - a FIR in CONSTANT state is fully evaluated.
     */
    CONSTANT
}
