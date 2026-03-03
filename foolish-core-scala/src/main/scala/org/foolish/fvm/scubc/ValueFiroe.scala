package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * ValueFiroe represents a finalized value in the computation.
 * For now, supports integral long values.
 */
case class ValueFiroe(override val ast: AST, value: Long) extends FiroeWithoutBraneMind(ast) with Constanicable:

  def this(value: Long) = this(null, value)

  /** Returns the integral value stored in this ValueFiroe */
  override def getValue: Long = value

  /** ValueFiroe is never abstract as it represents a concrete value */
  def isAbstract: Boolean = false

  /** ValueFiroe is always constanic (CONSTANT state) */
  override def isConstanic: Boolean = atConstanic

  /** Returns this ValueFiroe as the result (it's the final value) */
  override def getResult: FIR = this

  override def toString: String = value.toString

  override def clone: FIR = this // immutable constant

  /**
   * Clones this ValueFiroe with updated parent.
   * Even though ValueFiroe is CONSTANT (immutable), we need to update the parent
   * when it's part of a cloned brane. The clone returns a new ValueFiroe with
   * the same value but updated parent chain.
   */
  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    // Create a new ValueFiroe with the same value but new parent
    val copy = ValueFiroe(ast, value)
    copy.setParentFir(newParent)
    // Set target state if specified
    targetNyes.foreach(copy.setNyes)
    copy
