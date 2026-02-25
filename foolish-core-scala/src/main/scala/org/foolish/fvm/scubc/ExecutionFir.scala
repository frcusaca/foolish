package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * ExecutionFir coordinates stepping multiple FIRs to a target Nyes state.
 *
 * This is a reusable utility for breadth-first stepping of FIR collections
 * with specific milestone targeting.
 */
class ExecutionFir private (
  val managedFirs: List[FIR],
  val targetState: Nyes,
  val shouldSetParent: Boolean,
  onComplete: List[FIR] => Unit = _ => (),
  onStuck: List[FIR] => Unit = _ => ()
) extends FiroeWithBraneMind(null, Some("ExecutionFir")):

  // Set parent if requested
  if shouldSetParent then
    managedFirs.foreach(_.setParentFir(this))

  override protected def initialize(): Unit =
    setInitialized()
    // Add all FIRs that haven't reached target to braneMind for stepping
    managedFirs.foreach { fir =>
      if !hasReachedTarget(fir) then
        braneEnqueue(fir)
    }

  override def step(): Int =
    getNyes match
      case Nyes.UNINITIALIZED =>
        initialize()
        setNyes(Nyes.INITIALIZED)
        1
      case Nyes.INITIALIZED | Nyes.CHECKED | Nyes.PRIMED =>
        // Skip directly to EVALUATING
        setNyes(Nyes.EVALUATING)
        1
      case Nyes.EVALUATING =>
        stepManagedFirs()
        1
      case Nyes.CONSTANT | Nyes.CONSTANIC =>
        // Already evaluated, nothing to do
        0

  /**
   * Steps one FIR from the queue toward the target state.
   * Uses breadth-first stepping: dequeue, step, re-enqueue if not done.
   */
  private def stepManagedFirs(): Unit =
    if braneMind.isEmpty then
      // Check final state
      finalizeExecution()
      return

    val current = braneMind.dequeue()

    // Check if already at target
    if hasReachedTarget(current) then
      // Don't re-enqueue, check if we're done
      if braneMind.isEmpty then
        finalizeExecution()
      return

    // Check if stuck at CONSTANIC before target
    if isStuckBeforeTarget(current) then
      // This FIR cannot progress further
      // Remove from queue (don't re-enqueue)
      if braneMind.isEmpty then
        finalizeExecution()
      return

    // Step the FIR
    current.step()

    // Re-enqueue if not at target and not stuck
    if !hasReachedTarget(current) && !isStuckBeforeTarget(current) then
      braneMind.enqueue(current)

    // Check if we're done after this step
    if braneMind.isEmpty then
      finalizeExecution()

  /**
   * Checks if a FIR has reached (or passed) the target state.
   */
  private def hasReachedTarget(fir: FIR): Boolean =
    fir.getNyes.ordinal >= targetState.ordinal

  /**
   * Checks if a FIR is stuck at CONSTANIC before reaching target.
   * A FIR is stuck if it's at CONSTANIC but hasn't reached the target state.
   */
  private def isStuckBeforeTarget(fir: FIR): Boolean =
    fir.atConstanic && !hasReachedTarget(fir)

  /**
   * Finalizes execution - determines if completed successfully or stuck.
   */
  private def finalizeExecution(): Unit =
    // Check if any FIR is stuck at CONSTANIC before target
    val anyStuck = managedFirs.exists(fir => isStuckBeforeTarget(fir))

    if anyStuck then
      stuck = true
      setNyes(Nyes.CONSTANIC)
      onStuck(managedFirs)
    else
      completed = true
      setNyes(Nyes.CONSTANT)
      onComplete(managedFirs)

  private var completed = false
  private var stuck = false

  /** Returns true if all managed FIRs have reached the target state */
  def isCompleted: Boolean = completed

  /** Returns true if some managed FIRs are stuck at CONSTANIC before target */
  def isStuck: Boolean = stuck

  /**
   * Returns true if this ExecutionFir has reached CONSTANIC or CONSTANT state.
   * This indicates the execution has completed (successfully or stuck).
   */
  override def isConstanic: Boolean =
    getNyes == Nyes.CONSTANIC || getNyes == Nyes.CONSTANT

  /** Returns the target state this ExecutionFir is stepping toward */
  def getTargetState: Nyes = targetState

object ExecutionFir:

  /** Creates a new ExecutionFir Builder with the given FIRs */
  def stepping(firs: FIR*): Builder = new Builder(firs.toList)

  /** Builder for creating ExecutionFir instances with fluent API */
  class Builder(private val firs: List[FIR]):
    private var targetState: Nyes = Nyes.CONSTANT
    private var setParent: Boolean = true
    private var onComplete: List[FIR] => Unit = _ => ()
    private var onStuck: List[FIR] => Unit = _ => ()

    /** Sets the target state to step all FIRs toward */
    def stepUntil(state: Nyes): Builder =
      this.targetState = state
      this

    /** Controls whether FIRs should be re-parented to the ExecutionFir */
    def setParent(set: Boolean): Builder =
      this.setParent = set
      this

    /** Sets callback to invoke when all FIRs reach the target state */
    def onComplete(callback: List[FIR] => Unit): Builder =
      this.onComplete = callback
      this

    /** Sets callback to invoke when some FIRs are stuck at CONSTANIC before target */
    def onStuck(callback: List[FIR] => Unit): Builder =
      this.onStuck = callback
      this

    /** Builds and returns the configured ExecutionFir */
    def build(): ExecutionFir =
      new ExecutionFir(firs, targetState, setParent, onComplete, onStuck)
