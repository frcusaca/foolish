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
   * Constructor for cloneConstanic that creates a fresh AssignmentFiroe from AST.
   * This ensures that identifiers are re-evaluated in the new context.
   * Matches Java's behavior: stores expressions but doesn't call ordination for children.
   */
  private def this(assignment: AST.Assignment, newParent: FIR, lhs: CharacterizedIdentifier) =
    this(assignment)
    setParentFir(newParent)
    _lhs = lhs
    result = null
    ordinated = false
    // Initialize braneMemory from AST (same as original constructor)
    // This must be called AFTER setting up the object but BEFORE marking as initialized
    storeExprs(assignment.expr())

  override protected def initialize(): Unit =
    if isInitialized then return
    setInitialized()
    // Store the expression in braneMemory (prime() will enqueue to braneMind)
    // This matches Java's AssignmentFiroe.initialize() behavior
    val expr = assignment.expr()
    System.out.println(s"DEBUG AssignmentFiroe.initialize: assignment=$assignment assignmentClass=${assignment.getClass.getSimpleName} expr=$expr exprClass=${expr.getClass.getSimpleName}")
    storeExprs(expr)
    System.out.println(s"DEBUG AssignmentFiroe.initialize: braneMemory=${braneMemory.stream.map(_.getClass.getSimpleName).mkString(", ")}")

  override def step(): Int =
    if result != null then
      return 0

    // Check if we're constanic (e.g., unresolved due to missing identifier)
    if atConstanic then
      return 0

    // Handle EVALUATING state - step the expression in braneMind
    if getNyes == Nyes.EVALUATING then
      // If braneMind is empty, expression is complete
      if braneMind.isEmpty then
        // Expression evaluated, store result and determine final state
        if !braneMemory.isEmpty then
          val res = braneMemory.get(0)
          result = res
          if res.atConstanic then
            setNyes(Nyes.CONSTANIC)
          else
            setNyes(Nyes.CONSTANT)
        else
          setNyes(Nyes.CONSTANT)
        return 1

      // Step the expression in braneMind
      val current = braneMind.dequeue()
      try
        val w = current.step()
        if current.isNye then
          braneMind.enqueue(current)
        // After stepping, check if braneMind is now empty (expression completed)
        if braneMind.isEmpty then
          // Expression is complete, store result
          if !braneMemory.isEmpty then
            val res = braneMemory.get(0)
            result = res
            if res.atConstanic then
              setNyes(Nyes.CONSTANIC)
            else
              setNyes(Nyes.CONSTANT)
        return w
      catch
        case e: Exception =>
          braneMind.prepend(current)
          throw RuntimeException("Error during expression evaluation", e)

    // Let parent class handle initialization and other state transitions
    val work = super.step()

    work

  override def isAbstract: Boolean =
    if atConstanic then
      true
    else if result == null then
      true
    else
      result.isAbstract

  override def isConstanic: Boolean =
    if result != null then
      // Use the result's isConstanic, but fall back to checking our own state
      // if the result's isConstanic returns false (which can happen if the result
      // has its own override that doesn't match the expected behavior)
      result.isConstanic || getNyes == Nyes.CONSTANIC || getNyes == Nyes.CONSTANT
    else if getNyes == Nyes.CONSTANIC then
      true
    else
      super.isConstanic

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

  override def valuableSelf(): java.util.Optional[FIR] =
    if result != null then
      result.valuableSelf()
    else if atConstanic then
      java.util.Optional.empty[FIR]()
    else
      null

  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    if !isConstanic then
      throw IllegalStateException(
        s"cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
        s"but this FIR is in state: ${getNyes}")
    if isConstant then
      return this  // Share CONSTANT assignments completely
    // CONSTANIC: create copy manually to avoid cloning braneMemory items
    // which may contain non-constanic expressions that were replaced by result.
    // Matches Java's behavior: use private constructor, don't call ordination.
    val copy = new AssignmentFiroe(assignment.asInstanceOf[AST.Assignment], newParent, _lhs)
    val originalHashCode = System.identityHashCode(this)
    val copyHashCode = System.identityHashCode(copy)
    val newParentHashCode = System.identityHashCode(newParent)
    System.out.println(s"DEBUG AssignmentFiroe.cloneConstanic: original=$this originalHashCode=$originalHashCode copy=$copy copyHashCode=$copyHashCode newParent=$newParent newParentClass=${newParent.getClass.getSimpleName} newParentHashCode=$newParentHashCode")
    // Set initialized so initialize() won't be called again when stepped
    // (expressions are already stored via storeExprsQuietly in the constructor)
    copy.setInitialized()
    // Apply targetNyes if specified (matches Java behavior)
    // If not specified, copy inherits the original's state
    if targetNyes.isDefined then
      copy.setNyes(targetNyes.get)
    else
      copy.setNyes(this.getNyes)
    System.out.println(s"DEBUG AssignmentFiroe.cloneConstanic: After targetNyes nyes=${copy.getNyes} targetNyes=$targetNyes copy.ordinated=${copy.ordinated}")
    copy

object AssignmentFiroe:
  def apply(assignment: AST.Assignment): AssignmentFiroe =
    new AssignmentFiroe(assignment)
