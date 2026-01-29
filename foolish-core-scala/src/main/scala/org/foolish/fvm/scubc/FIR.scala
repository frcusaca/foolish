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
  private var parentFir: FIR = null  // The FIR that contains this FIR (set during enqueue)

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
   * Sets the parent FIR that contains this FIR.
   * This is called when this FIR is enqueued into a parent's braneMind.
   */
  protected[scubc] def setParentFir(parent: FIR): Unit =
    parentFir = parent

  /**
   * Gets the parent FIR that contains this FIR.
   * Returns null if this FIR has no parent (e.g., root brane).
   */
  protected def getParentFir: FIR = parentFir

  final def atConstant: Boolean = nyes == Nyes.CONSTANT
  final def atConstanic: Boolean = nyes == Nyes.CONSTANIC

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

  /**
   * Gets the BraneFiroe that contains this FIR in its statement list.
   * Chains through parent FIRs until finding one whose parent is a BraneFiroe.
   * <p>
   * The containing brane is the closest BraneFiroe in the FIR hierarchy,
   * representing the brane where this FIR appears as a statement.
   * <p>
   * Parallel expressions (such as operands in a+b) are at the "same height"
   * and all statements in a brane are parallel/same height. The height does
   * NOT deepen with deepening FIR structures - height only changes when
   * crossing brane boundaries.
   *
   * @return the containing BraneFiroe, or null if this FIR is not contained
   *         in a brane (e.g., at root level)
   */
  def getMyBrane: BraneFiroe =
    parentFir match
      case bf: BraneFiroe => bf
      case null => null
      case _ => parentFir.getMyBrane

  /**
   * Gets the index of this FIR in its containing brane's memory.
   * Chains through parent FIRs to find the statement-level FIR, then returns its position.
   * <p>
   * The brane index defines the "order of expressions" within a height level.
   * All statements at the same brane level are parallel/same height, and the
   * index orders them for operations like unanchored backward search.
   * <p>
   * Note: The index is for the statement containing this FIR, not necessarily
   * this exact FIR object. For example, if this is a sub-expression of an
   * assignment, it returns the assignment's index in the brane.
   *
   * @return the index in the containing brane's memory (0-based), or -1 if
   *         this FIR is not in a brane (root level)
   */
  def getMyBraneIndex: Int =
    parentFir match
      case null => -1
      case bf: BraneFiroe => bf.getIndexOf(this)
      case _ => parentFir.getMyBraneIndex

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
    case regexpSearch: AST.RegexpSearchExpr =>
      if (DerefSearchFiroe.isExactMatch(regexpSearch.pattern())) {
        new DerefSearchFiroe(regexpSearch)
      } else {
        RegexpSearchFiroe(regexpSearch)
      }
    case oneShotSearch: AST.OneShotSearchExpr => OneShotSearchFiroe(oneShotSearch)
    case dereferenceExpr: AST.DereferenceExpr =>
      val synthetic = new AST.RegexpSearchExpr(dereferenceExpr.anchor(), org.foolish.ast.SearchOperator.REGEXP_LOCAL, dereferenceExpr.coordinate().toString)
      new DerefSearchFiroe(synthetic, dereferenceExpr)
    case seekExpr: AST.SeekExpr => SeekFiroe(seekExpr)
    case unanchoredSeekExpr: AST.UnanchoredSeekExpr => UnanchoredSeekFiroe(unanchoredSeekExpr)
    case _ => ValueFiroe(null, 0L) // Placeholder for unsupported types
