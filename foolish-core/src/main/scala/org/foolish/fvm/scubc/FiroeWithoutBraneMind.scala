package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * FIR without a braneMind queue.
 * Used for simple values that don't require step-by-step evaluation.
 */
abstract class FiroeWithoutBraneMind(val ast: AST, val comment: Option[String] = None) extends FIR:

  /** FiroeWithoutBraneMind instances don't require stepping */
  def step(): Unit = () // No-op

  /** FiroeWithoutBraneMind instances are never NYE (Not Yet Evaluated) */
  def isNye: Boolean = false
