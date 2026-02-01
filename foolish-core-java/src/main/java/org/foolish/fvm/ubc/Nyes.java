package org.foolish.fvm.ubc;

/**
 * NYE STATES (say "NICE") - represents the evaluation stages of a Firoe.
 * <p>
 * This enum tracks the progression of a FIR from initial construction through
 * complete evaluation. Each state represents a distinct phase in the evaluation
 * lifecycle, with transitions managed through the {@link FIR#setNyes(Nyes)} method.
 * <p>
 * The evaluation flow typically progresses as follows:
 * UNINITIALIZED → INITIALIZED → CHECKED → PRIMED → EVALUATING → CONSTANIC → CONSTANT
 * <p>
 * CONSTANIC (say "CON-STAN-NICK") represents a state where evaluation has paused due to missing information (e.g. unbound identifiers),
 * but could resume in a different context. CONSTANt IN Context - "Stay Foolish" state.
 * CONSTANT represents a fully evaluated, immutable state (Result or Error).
 * <p>
 * <b>STATE CHECKING CONVENTIONS ("at" vs "is"):</b>
 * <p>
 * Two conventions exist for checking states in {@link FIR}:
 * <ul>
 *   <li><b>"at" methods</b> - EXACT state match only
 *     <ul>
 *       <li>{@code atConstanic()}: true ONLY when {@code nyes == CONSTANIC}</li>
 *       <li>{@code atConstant()}: true ONLY when {@code nyes == CONSTANT}</li>
 *     </ul>
 *   </li>
 *   <li><b>"is" methods</b> - AT LEAST that state (includes higher states)
 *     <ul>
 *       <li>{@code isConstanic()}: true when {@code nyes >= CONSTANIC} (i.e., CONSTANIC OR CONSTANT)</li>
 *       <li>{@code isConstant()}: true when {@code nyes >= CONSTANT}</li>
 *     </ul>
 *   </li>
 * </ul>
 * <p>
 * <b>DECISION GUIDE:</b>
 * <ul>
 *   <li>Need to distinguish CONSTANIC from CONSTANT? → Use {@code atConstanic()} and {@code atConstant()}</li>
 *   <li>Checking if FIR is done evaluating (either state acceptable)? → Use {@code isConstanic()}</li>
 *   <li>Checking if FIR is fully resolved? → Use {@code isConstant()}</li>
 * </ul>
 * <p>
 * Example: CMFir Phase A decision uses {@code atConstanic()} to detect when to start Phase B,
 * while cloneConstanic precondition uses {@code isConstanic()} since both states are valid.
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
     * Type/reference checking completed.
     * Reserved for future type checking and reference validation.
     * Currently used as a transitional state between INITIALIZED and PRIMED.
     * AB (Abstract Brane), IB (Implementation Brane) established firmly.
     * All variables for an expression are collected and validated.
     * Transition taken care of by step().
     */
    CHECKED,

    /**
     * BraneMind has been primed with non-constant items from braneMemory.
     * For FiroeWithBraneMind: non-constant items from braneMemory are enqueued into braneMind.
     * For other FIRs: this state is typically transitioned through immediately.
     * This separation ensures CONSTANIC FIRs have empty braneMind (critical for cloneConstanic).
     * Transition taken care of by step().
     */
    PRIMED,

    /**
     * Take a step() as we do currently, including branes.
     * Active evaluation is in progress.
     * Transition taken care of by step().
     */
    EVALUATING,

    /**
     * Like CONSTANT, it is a terminal state as far as `step()` is concerned.
     * Evaluation halted due to missing information (unbound identifiers).
     * It is CONSTANt IN Context (say "CON-STAN-NICK"). It is constant if context does not change.
     * It is not required that this FIR do change for some context. But
     * for computational efficiency, it would be best if Constanic state
     * only happens for a FIR that is expected to change if context changes.
     */
    CONSTANIC,

    /**
     * No more changes will happen with call to step() unless the environment changes.
     * This is the only !isNye() state - a FIR in CONSTANT state is fully evaluated.
     */
    CONSTANT
}
