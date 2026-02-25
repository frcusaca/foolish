package org.foolish.fvm.scubc

/**
 * Trait for FIRs that can reach the CONSTANIC state and have an underlying result
 * that can be unwrapped/resolved.
 *
 * Constanicables may need cloning and re-evaluation in different contexts
 * (e.g., concatenation).
 *
 * @see FIR.unwrapConstanicable for unwrapping Constanicables to their resolved value
 */
trait Constanicable:
  /**
   * Returns true if this Constanicable is at CONSTANIC or CONSTANT state.
   */
  def isConstanic: Boolean

  /**
   * Returns the resolved/computed result of this Constanicable.
   *
   * For wrapper types (IdentifierFiroe, AssignmentFiroe, SearchFiroe, etc.),
   * this returns the underlying FIR that this wrapper resolves to.
   *
   * For container types (BraneFiroe, ConcatenationFiroe), this returns `this`
   * since the container IS the result.
   *
   * Returns `null` if the result is not yet available (still evaluating)
   * or if the Constanicable is stuck (truly CONSTANIC with no resolution).
   */
  def getResult: FIR

  /**
   * Clones this Constanicable FIR for use in a new context.
   *
   * Used during brane concatenation and other scenarios where a constanic FIR
   * needs to be re-evaluated with different parent relationships or context.
   *
   * @param newParent The new parent FIR for the clone
   * @param targetNyes Optional target state to set on the clone
   * @return A new clone of this FIR
   */
  def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR
