package org.foolish.fvm.ubc;

/**
 * Marker interface for FIRs that can reach the CONSTANIC state.
 * <p>
 * A Constanicable FIR is one that:
 * <ol>
 *   <li>Can meaningfully reach CONSTANIC state (not immediately CONSTANT like literals)</li>
 *   <li>Has an underlying result that can be unwrapped/resolved</li>
 *   <li>May need cloning and re-evaluation in different contexts (e.g., concatenation)</li>
 * </ol>
 * <p>
 * <b>Wrapper Constanicables</b> resolve to another FIR via {@link #getResult()}:
 * <ul>
 *   <li>{@link IdentifierFiroe} - resolves identifier name to FIR in memory</li>
 *   <li>{@link AssignmentFiroe} - evaluates RHS expression to FIR</li>
 *   <li>{@link CMFir} - context manipulation wrapper, phases A/B</li>
 *   <li>{@link AbstractSearchFiroe} - search operations (DerefSearchFiroe, RegexpSearchFiroe, etc.)</li>
 *   <li>{@link UnanchoredSeekFiroe} - backward seek by offset</li>
 *   <li>{@link IfFiroe} - conditional branch selection</li>
 * </ul>
 * <p>
 * <b>Computation Constanicables</b> compute a result from operands:
 * <ul>
 *   <li>{@link BinaryFiroe} - binary operations (+, -, *, /, etc.)</li>
 *   <li>{@link UnaryFiroe} - unary operations (-, !)</li>
 * </ul>
 * <p>
 * <b>Container Constanicables</b> hold statements that may not fully resolve:
 * <ul>
 *   <li>{@link BraneFiroe} - statement container { ... }</li>
 *   <li>{@link ConcatenationFiroe} - joined branes</li>
 * </ul>
 * <p>
 * <b>NOT Constanicable</b> (always immediately CONSTANT):
 * <ul>
 *   <li>{@link ValueFiroe} - literal integer values</li>
 *   <li>{@link NKFiroe} - Not-Known error values (???)</li>
 * </ul>
 *
 * @see FIR#unwrapConstanicable(FIR) for unwrapping Constanicables to their resolved value
 */
public interface Constanicable {

    /**
     * Returns the resolved/computed result of this Constanicable.
     * <p>
     * For wrapper types (IdentifierFiroe, AssignmentFiroe, SearchFiroe, etc.),
     * this returns the underlying FIR that this wrapper resolves to.
     * <p>
     * For container types (BraneFiroe, ConcatenationFiroe), this returns {@code this}
     * since the container IS the result.
     * <p>
     * Returns {@code null} if the result is not yet available (still evaluating)
     * or if the Constanicable is stuck (truly CONSTANIC with no resolution).
     *
     * @return the resolved FIR, or null if not resolved
     */
    FIR getResult();
}
