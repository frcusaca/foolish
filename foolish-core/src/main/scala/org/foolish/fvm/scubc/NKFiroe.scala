package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * NKFiroe represents a Not-Known (NK) value, displayed as ???.
 * This occurs when an operation cannot produce a valid result, such as:
 * - Division by zero
 * - Modulo by zero
 * - Other arithmetic errors
 */
case class NKFiroe(override val ast: AST = null, nkComment: Option[String] = None)
  extends FiroeWithoutBraneMind(ast, nkComment):

  def this(comment: String) = this(null, Some(comment))

  /** NKFiroe is abstract because the value is not known */
  def isAbstract: Boolean = true

  /** Cannot get a value from NK - it's not known */
  override def getValue: Long =
    throw IllegalStateException("Cannot get value from NK (not-known)")

  override def toString: String = "???"

  override def clone: FIR = this // immutable constant
