package org.foolish.fvm.scubc

import org.foolish.ast.AST
import scala.collection.mutable

/**
 * FIR with a braneMind queue for managing evaluation tasks.
 * The braneMind enables breadth-first execution of nested expressions.
 */
abstract class FiroeWithBraneMind(ast: AST, comment: Option[String] = None) extends FIR(ast, comment):

  protected val braneMind = mutable.Queue[FIR]()
  protected val braneMemory = mutable.ArrayBuffer[FIR]()

  /** Enqueues FIRs into the braneMind */
  protected def enqueueFirs(firs: FIR*): Unit =
    firs.foreach { fir =>
      braneMind.enqueue(fir)
      braneMemory.addOne(fir)
    }

  protected def enqueueExprs(exprs: AST.Expr*): Unit =
    exprs.foreach(expr => enqueueFirs(FIR.createFiroeFromExpr(expr)))

  protected def enqueueSubfirOfExprs(exprs: AST.Expr*): Unit =
    enqueueFirs(FiroeWithBraneMind.ofExpr(exprs*))

  /**
   * A FiroeWithBraneMind is NYE (Not Yet Evaluated) if its Nyes state is not CONSTANT.
   * Derived classes should override this method and call super.isNye as needed.
   */
  def isNye: Boolean = getNyes != Nyes.CONSTANT

  /** Check if any FIRs in braneMind or braneMemory are abstract */
  def isAbstract: Boolean =
    braneMind.exists(_.isAbstract) || braneMemory.exists(_.isAbstract)

  /**
   * Checks if a FIR is a brane (BraneFiroe).
   */
  protected def isBrane(fir: FIR): Boolean =
    fir.isInstanceOf[BraneFiroe]

  /**
   * Steps the next FIR in the braneMind queue with state-aware brane handling.
   *
   * State transitions:
   * - UNINITIALIZED → INITIALIZED: Initialize this FIR
   * - INITIALIZED → REFERENCES_IDENTIFIED: Step non-branes only until all are REFERENCES_IDENTIFIED
   * - REFERENCES_IDENTIFIED → ALLOCATED: Step non-branes only until all are ALLOCATED
   * - ALLOCATED → RESOLVED: Step non-branes only until all are RESOLVED
   * - RESOLVED → EVALUATING: Immediate transition when detected
   * - EVALUATING → CONSTANT: Step everything (including branes) until all complete
   *
   * Derived classes should override this method and call super.step() as needed.
   */
  def step(): Unit =
    getNyes match
      case Nyes.UNINITIALIZED =>
        // First step: initialize this FIR
        initialize()
        setNyes(Nyes.INITIALIZED)

      case Nyes.INITIALIZED =>
        // Step non-brane expressions until all are REFERENCES_IDENTIFIED
        if stepNonBranesUntilState(Nyes.REFERENCES_IDENTIFIED) then
          // All non-branes have reached REFERENCES_IDENTIFIED
          setNyes(Nyes.REFERENCES_IDENTIFIED)

      case Nyes.REFERENCES_IDENTIFIED =>
        // Step non-brane expressions until all are ALLOCATED
        if stepNonBranesUntilState(Nyes.ALLOCATED) then
          // All non-branes have reached ALLOCATED
          setNyes(Nyes.ALLOCATED)

      case Nyes.ALLOCATED =>
        // Step non-brane expressions until all are RESOLVED
        if stepNonBranesUntilState(Nyes.RESOLVED) then
          // All non-branes have reached RESOLVED
          setNyes(Nyes.RESOLVED)

      case Nyes.RESOLVED =>
        // Immediate transition to EVALUATING when step() is called
        setNyes(Nyes.EVALUATING)

      case Nyes.EVALUATING =>
        // Step everything including sub-branes
        if braneMind.isEmpty then
          // All expressions evaluated, transition to CONSTANT
          setNyes(Nyes.CONSTANT)
          return

        val current = braneMind.dequeue()
        try
          current.step()

          if current.isNye then
            braneMind.enqueue(current)
        catch
          case e: Exception =>
            braneMind.prepend(current) // Re-enqueue on error
            throw RuntimeException("Error during braneMind step execution", e)
            //TODO: Handle this exception Foolishly

      case Nyes.CONSTANT =>
        // Already evaluated, nothing to do

  /**
   * Steps non-brane FIRs until they reach the target state.
   * Branes are re-enqueued without stepping.
   *
   * @param targetState The state that non-branes should reach
   * @return true if all non-branes have reached the target state, false otherwise
   */
  private def stepNonBranesUntilState(targetState: Nyes): Boolean =
    if braneMind.isEmpty || allNonBranesReachedState(targetState) then
      return true

    val current = braneMind.dequeue()

    //Reach here only when we have found the first non-brane sub-targetState member
    try
      // Step non-brane expression
      current.step()

      // re-enqueue if still NYE
      if current.isNye then
        braneMind.enqueue(current)

      // Check if it has reached the target state
      if current.getNyes.ordinal < targetState.ordinal then
        return false // don't need to detect further, at least one non-brane is not at target state

    catch
      case e: Exception =>
        braneMind.prepend(current) // Re-enqueue on error
        throw RuntimeException("Error during braneMind step execution", e)
        //TODO: Handle this exception Foolishly

    // Check if all non-branes in the queue have reached target state
    allNonBranesReachedState(targetState)

  /**
   * Checks if all non-brane FIRs in the braneMind have reached at least the target state.
   * The first sub-target-state non-brane member is shifted to the front of the queue.
   */
  private def allNonBranesReachedState(targetState: Nyes): Boolean =
    var current = braneMind.headOption.getOrElse(return true)
    var seen = 1

    // Let's skip branes and the sub expressions that have already reached desired state.
    while isBrane(current) || (current.getNyes.ordinal >= targetState.ordinal) do
      // Re-enqueue brane without stepping - keep its place in line
      if seen >= braneMind.size then
        return true

      braneMind.enqueue(braneMind.dequeue())
      current = braneMind.head
      seen += 1

    false

object FiroeWithBraneMind:

  def ofExpr(tasks: AST.Expr*): FiroeWithBraneMind =
    of(tasks.map(FIR.createFiroeFromExpr)*)

  def of(tasks: FIR*): FiroeWithBraneMind =
    val result = new FiroeWithBraneMind(null, None):
      override protected def initialize(): Unit = ()
    result.enqueueFirs(tasks*)
    result
