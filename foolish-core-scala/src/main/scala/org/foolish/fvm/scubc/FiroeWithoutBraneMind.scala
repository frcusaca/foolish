package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * FIR without a braneMind queue.
 * Used for simple values that don't require step-by-step evaluation.
 */
abstract class FiroeWithoutBraneMind(ast: AST, comment: Option[String] = None) extends FIR(ast, comment):
  // FiroeWithoutBraneMind instances are immediately CONSTANT
  setNyes(Nyes.CONSTANT)

  /** FiroeWithoutBraneMind instances don't require stepping, but may need state recovery */
  def step(): Int =
    // If we are somehow reset to non-CONSTANT (e.g. via cloneConstanic resetting to INITIALIZED),
    // we must transition back to CONSTANT because we are intrinsically constant.
    if getNyes != Nyes.CONSTANT then
      setNyes(Nyes.CONSTANT)
      return 1
    0 // No-op, returns 0 for no work done

  /**
   * FiroeWithoutBraneMind instances are never NYE (Not Yet Evaluated).
   * They represent finalized values and are always in CONSTANT state.
   */
  override def isNye: Boolean = getNyes != Nyes.CONSTANT
