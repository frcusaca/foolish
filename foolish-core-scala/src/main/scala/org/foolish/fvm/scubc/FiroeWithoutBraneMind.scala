package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * FIR without a braneMind queue.
 * Used for simple values that don't require step-by-step evaluation.
 */
abstract class FiroeWithoutBraneMind(ast: AST, comment: Option[String] = None) extends FIR(ast, comment):
  // FiroeWithoutBraneMind instances are immediately CONSTANT
  setNyes(Nyes.CONSTANT)

  /** FiroeWithoutBraneMind instances don't require stepping */
  def step(): Unit = () // No-op

  /**
   * FiroeWithoutBraneMind instances are never NYE (Not Yet Evaluated).
   * They represent finalized values and are always in CONSTANT state.
   */
  override def isNye: Boolean = getNyes != Nyes.CONSTANT
