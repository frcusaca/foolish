package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * Foolish Internal Representation (FIR).
 * The FIR is the internal representation of computation that holds an AST
 * and tracks evaluation progress.
 */
abstract class FIR(val ast: AST, val comment: Option[String] = None):
  private var initialized: Boolean = false
  private var nyes: Nyes = Nyes.UNINITIALIZED

  /**
   * Returns the current NYE state of this FIR.
   */
  def getNyes: Nyes = nyes

  /**
   * Sets the NYE state of this FIR.
   * All changes to a Firoe's Nyes must be made through this method.
   */
  protected def setNyes(newNyes: Nyes): Unit =
    nyes = newNyes

  /**
   * Checks if this FIR has been initialized.
   */
  protected def isInitialized: Boolean = initialized

  /**
   * Marks this FIR as initialized.
   */
  protected def setInitialized(): Unit =
    initialized = true

  /**
   * Initializes this FIR. Called once before first step.
   * Override in subclasses to add initialization logic.
   */
  protected def initialize(): Unit =
    if !initialized then
      setInitialized()

  /** Perform one step of evaluation on this FIR */
  def step(): Unit

  /**
   * Query method returning false if an additional step on this FIR does not change it.
   * Returns true when an additional step would change the FIR.
   * Not Yet Evaluated (NYE) indicates the FIR requires further evaluation steps.
   */
  def isNye: Boolean

  /**
   * Query method returning false only when all identifiers are bound.
   * Returns true if there are unbound identifiers (abstract state).
   */
  def isAbstract: Boolean

  /**
   * Gets the value from this FIR if it represents a simple value.
   * Throws UnsupportedOperationException if not supported.
   */
  def getValue: Long =
    throw UnsupportedOperationException(s"getValue not supported for ${getClass.getSimpleName}")

object FIR:
  /** Creates a FIR from an AST expression */
  def createFiroeFromExpr(expr: AST.Expr): FIR = expr match
    case literal: AST.IntegerLiteral => ValueFiroe(literal, literal.value())
    case binary: AST.BinaryExpr => BinaryFiroe(binary)
    case unary: AST.UnaryExpr => UnaryFiroe(unary)
    case ifExpr: AST.IfExpr => IfFiroe(ifExpr)
    case brane: AST.Brane => BraneFiroe(brane)
    case assignment: AST.Assignment => AssignmentFiroe(assignment)
    case identifier: AST.Identifier => IdentifierFiroe(identifier)
    case _ => ValueFiroe(null, 0L) // Placeholder for unsupported types
