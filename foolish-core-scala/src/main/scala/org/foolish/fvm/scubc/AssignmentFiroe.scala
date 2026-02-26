package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * FIR for assignment expressions.
 * An assignment evaluates its right-hand side expression and stores the result
 * with a coordinate name in the brane's environment.
 */
class AssignmentFiroe(assignment: AST.Assignment)
  extends FiroeWithBraneMind(assignment) with Constanicable:

  private var _lhs = CharacterizedIdentifier(assignment.identifier())
  private var result: FIR = null

  /**
   * Constructor for cloneConstanic that avoids cloning braneMemory items.
   * Instead, it creates a fresh AssignmentFiroe from the AST.
   */
  private def this(assignment: AST.Assignment, newParent: FIR, lhs: CharacterizedIdentifier) =
    this(assignment)
    setParentFir(newParent)
    _lhs = lhs
    result = null
    // indexLookup is already initialized in base class, don't reassign
    // Just need to clear it if needed, but it should be fine as-is since we're creating a fresh copy
    enqueueExprs(assignment.expr())

  override protected def initialize(): Unit =
    if isInitialized then return
    setInitialized()

    println(s"DEBUG AssignmentFiroe.initialize: assignment.expr=${assignment.expr()}")
    enqueueExprs(assignment.expr())
    println(s"DEBUG AssignmentFiroe.initialize: braneMemory.size=${braneMemory.size}")

  override def step(): Int =
    if result != null then
      return 0

    if atConstanic then
      return 0

    if !isInitialized then
      initialize()
      return 1

    // Let parent class handle braneMind stepping and state transitions
    val work = super.step()

    // Check if we can get the final result
    if !super.isNye && !braneMemory.isEmpty then
      val res = braneMemory.get(0)
      println(s"DEBUG AssignmentFiroe.step: result=$res, res.class=${res.getClass.getSimpleName}, res.atConstanic=${res.atConstanic}, self.getNyes=${getNyes}")
      result = res
      if res.atConstanic then
        setNyes(Nyes.CONSTANIC)
      else
        setNyes(Nyes.CONSTANT)
      println(s"DEBUG AssignmentFiroe.step: setNyes to ${getNyes}")
    work

  override def isAbstract: Boolean =
    if atConstanic then
      true
    else if result == null then
      true
    else
      result.isAbstract

  override def isNye: Boolean =
    result == null && !isConstanic

  /** Gets the coordinate name for this assignment (without characterization) */
  def getId: String = _lhs.getId

  /** Gets the LHS characterized identifier */
  def getLhs: CharacterizedIdentifier = _lhs

  /** Gets the evaluated result FIR */
  override def getResult: FIR = result

  override def getValue: Long =
    if atConstanic then
      throw IllegalStateException("AssignmentFiroe is constanic")
    if result == null then
      if getNyes == Nyes.CONSTANIC then
        throw IllegalStateException("AssignmentFiroe evaluated to constanic (unresolved)")
      throw IllegalStateException("AssignmentFiroe not fully evaluated")
    result.getValue

  override def toString: String =
    Sequencer4Human().sequence(this)

  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    if !isConstanic then
      throw IllegalStateException(
        s"cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
        s"but this FIR is in state: ${getNyes}")
    if isConstant then
      return this  // Share CONSTANT assignments completely
    // CONSTANIC: create copy manually to avoid cloning braneMemory items
    val copy = new AssignmentFiroe(ast.asInstanceOf[AST.Assignment], newParent, _lhs)
    // Don't copy result - let the clone re-evaluate from scratch to ensure
    // it resolves identifiers in the new context (e.g., CMFir's parent brane)
    copy.result = null
    copy.setInitialized()
    copy.setNyes(getNyes)
    // Set target state if specified
    targetNyes.foreach(copy.setNyes)
    copy

object AssignmentFiroe:
  def apply(assignment: AST.Assignment): AssignmentFiroe =
    new AssignmentFiroe(assignment)
