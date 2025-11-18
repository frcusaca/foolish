package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * ValueFiroe represents a finalized value in the computation.
 * For now, supports integral long values.
 */
case class ValueFiroe(override val ast: AST, value: Long) extends FiroeWithoutBraneMind(ast):

  def this(value: Long) = this(null, value)

  /** Returns the integral value stored in this ValueFiroe */
  override def getValue: Long = value

  /** ValueFiroe is never abstract as it represents a concrete value */
  def isAbstract: Boolean = false

  override def toString: String = value.toString

  override def clone: FIR = this // immutable constant
