package org.foolish.fvm.scubc

import org.foolish.ast.AST
import scala.jdk.CollectionConverters.*

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
  protected[scubc] def getParentFir: FIR = parentFir

  final def atConstant: Boolean = nyes == Nyes.CONSTANT
  final def atConstanic: Boolean = nyes == Nyes.CONSTANIC

  /**
   * Returns true when nyes >= CONSTANIC (i.e., CONSTANIC OR CONSTANT)
   * This is the Scala equivalent of Java's isConstanic() method.
   */
  def isConstanic: Boolean = nyes == Nyes.CONSTANIC || nyes == Nyes.CONSTANT

  /**
   * Returns true when nyes >= CONSTANT (i.e., CONSTANT only)
   * This is the Scala equivalent of Java's isConstant() method.
   */
  final def isConstant: Boolean = nyes == Nyes.CONSTANT

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
  def step(): Int

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
   * Returns the "valuable self" of this FIR, resolving wrappers to their significant values.
   * <p>
   * This matches Java's valuableSelf() method behavior:
   * - Returns Some(this) by default (literal constants, branes, etc).
   * - Returns result.valuableSelf() for AssignmentFiroe.
   * - Returns value.valuableSelf() for IdentifierFiroe.
   * - Returns None if the FIR is constanic/unresolved.
   * - Returns null if the FIR shouldn't have been called yet (pre-PRIMED).
   */
  def valuableSelf(): java.util.Optional[FIR] =
    java.util.Optional.of(this)

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
   * First finds the containing BraneFiroe using getMyBrane(), then returns its index.
   * If not directly in the brane, delegates to parent.
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
    val containingBrane = getMyBrane
    if containingBrane == null then
      -1
    else
      val directIndex = containingBrane.getIndexOf(this)
      if directIndex != -1 then
        directIndex
      else
        // Not directly in the brane - delegate to parent
        if parentFir != null then
          parentFir.getMyBraneIndex
        else
          -1

  /**
   * Gets the FiroeWithBraneMind container (BraneFiroe or ConcatenationFiroe)
   * that contains this FIR in its braneMemory.
   * Chains through parent FIRs to find the container.
   * <p>
   * This method is used by unanchored seeks to find the scope of their backward search.
   * The search scope includes both branes and concatenations (joined branes).
   *
   * Note: This method only returns BraneFiroe or ConcatenationFiroe, not other
   * FiroeWithBraneMind subclasses like AssignmentFiroe or BinaryFiroe.
   *
   * @return the containing BraneFiroe or ConcatenationFiroe, or null if not contained
   */
  def getMyBraneContainer: BraneFiroe | ConcatenationFiroe =
    parentFir match
      case bf: BraneFiroe => bf
      case cf: ConcatenationFiroe => cf
      case null => null
      case _ => parentFir.getMyBraneContainer

  /**
   * Gets the index of this FIR in its containing brane container's memory.
   * Chains through parent FIRs to find the statement-level FIR, then returns its position.
   * <p>
   * Works with both BraneFiroe and ConcatenationFiroe containers.
   *
   * Note: This method only considers BraneFiroe or ConcatenationFiroe as containers,
   * not other FiroeWithBraneMind subclasses like AssignmentFiroe or BinaryFiroe.
   *
   * @return the index in the containing memory (0-based), or -1 if not in a container
   */
  def getMyBraneContainerIndex: Int =
    parentFir match
      case null =>
        System.out.println(s"DEBUG getMyBraneContainerIndex: null parentFir returning -1 for $this")
        -1
      case bf: BraneFiroe =>
        val idx = bf.getIndexOf(this)
        System.out.println(s"DEBUG getMyBraneContainerIndex: parent is BraneFiroe (hashCode=${System.identityHashCode(bf)}), getIndexOf(this)=$idx for $this (hashCode=${System.identityHashCode(this)})")
        if idx < 0 then
          System.out.println(s"DEBUG getMyBraneContainerIndex: WARNING - idx < 0, searching parent chain")
        idx
      case cf: ConcatenationFiroe =>
        val idx = cf.getIndexOf(this)
        System.out.println(s"DEBUG getMyBraneContainerIndex: parent is ConcatenationFiroe (hashCode=${System.identityHashCode(cf)}), getIndexOf(this)=$idx for $this (hashCode=${System.identityHashCode(this)})")
        if idx < 0 then
          System.out.println(s"DEBUG getMyBraneContainerIndex: WARNING - idx < 0, searching parent chain")
        idx
      case _ =>
        val parentIdx = parentFir.getMyBraneContainerIndex
        System.out.println(s"DEBUG getMyBraneContainerIndex: parent is ${parentFir.getClass.getSimpleName} (hashCode=${System.identityHashCode(parentFir)}), chaining to parent, parentIdx=$parentIdx for $this (hashCode=${System.identityHashCode(this)})")
        parentIdx

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
      if DerefSearchFiroe.isExactMatch(regexpSearch.pattern()) then
        new DerefSearchFiroe(regexpSearch)
      else
        new RegexpSearchFiroe(regexpSearch)
    case oneShotSearch: AST.OneShotSearchExpr => new OneShotSearchFiroe(oneShotSearch)
    case dereferenceExpr: AST.DereferenceExpr =>
      val synthetic = new AST.RegexpSearchExpr(dereferenceExpr.anchor(), org.foolish.ast.SearchOperator.REGEXP_LOCAL, dereferenceExpr.coordinate().toString)
      new DerefSearchFiroe(synthetic, dereferenceExpr)
    case seekExpr: AST.SeekExpr => SeekFiroe(seekExpr)
    case unanchoredSeekExpr: AST.UnanchoredSeekExpr => UnanchoredSeekFiroe(unanchoredSeekExpr)
    case concatenation: AST.Concatenation => ConcatenationFiroe(concatenation)
    case _ => ValueFiroe(null, 0L) // Placeholder for unsupported types

  /**
   * Unwraps a Constanicable FIR to its resolved value.
   *
   * Follows the wrapper chain while the FIR is constanic:
   * - IdentifierFiroe -> its value
   * - AssignmentFiroe -> its result
   * - SearchFiroe -> its searchResult
   *
   * Returns the first FIR in the chain that is either:
   * - Not a Constanicable
   * - A Constanicable with null or self-referential result
   */
  def unwrapConstanicable(fir: FIR): FIR =
    var current = fir
    while current != null do
      if current.isInstanceOf[Constanicable] then
        val result = current.asInstanceOf[Constanicable].getResult
        if result == null || result == current then return current
        current = result
      else
        return current
    current
