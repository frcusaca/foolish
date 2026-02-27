package org.foolish.fvm.scubc

import org.foolish.ast.AST
import scala.collection.mutable

/**
 * FIR with a braneMind queue for managing evaluation tasks.
 * The braneMind enables breadth-first execution of nested expressions.
 */
abstract class FiroeWithBraneMind(ast: AST, comment: Option[String] = None)
    extends FIR(ast, comment) with Constanicable:

  protected[scubc] var braneMind = mutable.Queue[FIR]()
  protected[scubc] var braneMemory = new BraneMemory(null)
  protected[scubc] var ordinated: Boolean = false
  protected val indexLookup = new java.util.IdentityHashMap[FIR, Int]()

  def ordinateToParentBraneMind(parent: FiroeWithBraneMind, myPos: Int): Unit =
    assert(!this.ordinated)
    this.braneMemory.setParent(parent.braneMemory)
    // For BraneFiroe: position is tracked via parent FIR relationships (getMyBraneIndex)
    // For other FIRs (like IdentifierFiroe): we need to set myPos since they don't have owningBrane
    if !this.isInstanceOf[BraneFiroe] then
      this.braneMemory.setMyPosInternal(myPos)
    this.ordinated = true

  /** Enqueues FIRs into the braneMind */
  protected def enqueueFirs(firs: FIR*): Unit =
    firs.foreach { fir =>
      braneMind.enqueue(fir)
      braneMemory.put(fir)
      // Set parent FIR relationship
      fir.setParentFir(this)
      val index = braneMind.size - 1
      indexLookup.put(fir, index)
      fir match
        case fwbm: FiroeWithBraneMind =>
          fwbm.ordinateToParentBraneMind(this, index)
        case _ =>
    }

  def getIndexOf(f: FIR): Int =
    indexLookup.get(f)

  /**
   * Returns an iterator over all FIRs in braneMemory.
   * Used by ConcatenationFiroe to iterate over statements.
   */
  def stream: Iterator[FIR] =
    braneMemory.stream

  /**
   * Adds a FIR to braneMind only (not braneMemory).
   * Used by IdentifierFiroe for delayed enqueuing.
   */
  protected def braneEnqueue(fir: FIR): Unit =
    braneMind.enqueue(fir)

  /**
   * Stores FIRs in braneMemory only (not braneMind).
   * Used by IdentifierFiroe and ConcatenationFiroe for storage without immediate evaluation.
   */
  protected def storeFirs(firs: FIR*): Unit =
    firs.foreach { fir =>
      braneMemory.put(fir)
      val index = braneMemory.size - 1
      indexLookup.put(fir, index)
      // Set parent FIR relationship for proper getMyBrane() and getMyBraneIndex() tracking
      fir.setParentFir(this)
      // For nested FiroeWithBraneMind instances, set up parent memory link
      if fir.isInstanceOf[FiroeWithBraneMind] then
        val fwbm = fir.asInstanceOf[FiroeWithBraneMind]
        if !fwbm.ordinated then
          fwbm.ordinateToParentBraneMind(this, index)
    }

  /**
   * Enqueues non-constanic FIRs from braneMemory to braneMind.
   * Used to transition from CHECKED to PRIMED state.
   * Only enqueues items that are not at CONSTANIC or CONSTANT state.
   */
  protected def prime(): Unit =
    val iterator = braneMemory.iterator
    while iterator.hasNext do
      val fir = iterator.next()
      if !fir.isConstanic then
        braneMind.enqueue(fir)

  protected def enqueueExprs(exprs: AST.Expr*): Unit =
    exprs.foreach(expr => enqueueFirs(FIR.createFiroeFromExpr(expr)))

  protected def enqueueSubfirOfExprs(exprs: AST.Expr*): Unit =
    enqueueFirs(FiroeWithBraneMind.ofExpr(exprs*))

  /**
   * A FiroeWithBraneMind is NYE (Not Yet Evaluated) if its Nyes state is not CONSTANT.
   * Derived classes should override this method and call super.isNye as needed.
   */
  def isNye: Boolean = getNyes != Nyes.CONSTANT && getNyes != Nyes.CONSTANIC

  /** Check if any FIRs in braneMind or braneMemory are abstract */
  def isAbstract: Boolean =
    braneMind.exists(_.isAbstract) || braneMemory.stream.exists(_.isAbstract)

  /**
   * Copy constructor for cloneConstanic.
   * Creates an exact copy of braneMemory with cloned items (updated parent chains).
   * BraneMemory is verified to be empty for CONSTANIC FIRs.
   *
   * @param original the FiroeWithBraneMind to copy
   * @param newParent the new parent for this clone
   */
  protected def this(original: FiroeWithBraneMind, newParent: FIR) =
    this(original.ast, original.comment)
    setParentFir(newParent)

    // Verify braneMind is empty (critical invariant for CONSTANIC FIRs)
    if !original.braneMind.isEmpty then
      throw IllegalStateException(
        s"cloneConstanic requires empty braneMind, but found " +
        s"${original.braneMind.size} items. " +
        "This indicates the FIR is not truly CONSTANIC.")

    // Create empty braneMind (already verified original is empty)
    this.braneMind = mutable.Queue[FIR]()

    // Create new braneMemory with null parent initially
    // The parent will be set by ordinateToParentBraneMind
    this.braneMemory = new BraneMemory(null)
    var index = 0
    original.braneMemory.stream.foreach { fir =>
      // Clone each item, resetting to INITIALIZED so they will re-evaluate
      // This is critical: nested FIRs must also be reset, not just the top-level brane
      val clonedFir = fir.asInstanceOf[Constanicable].cloneConstanic(this, Some(Nyes.INITIALIZED))
      this.braneMemory.put(clonedFir)

      // Add to indexLookup so getIndexOf works for cloned FIRs
      this.indexLookup.put(clonedFir, index)

      // Link cloned FIR's braneMemory to this brane's memory (ordination)
      // This is critical for identifier resolution in the new context
      clonedFir match
        case fwbm: FiroeWithBraneMind =>
          // Reset ordinated flag so we can re-ordinate in new context
          fwbm.ordinated = false
          fwbm.ordinateToParentBraneMind(this, index)
        case _ =>
      index += 1
    }

    this.ordinated = original.ordinated
    setInitialized()

  override def getResult: FIR = this

  /**
   * Clones this Constanicable FIR for use in a new context.
   * This base implementation creates a fresh copy from AST and sets up the
   * parent memory link for nested FiroeWithBraneMind instances.
   *
   * @param newParent The new parent FIR for the clone
   * @param targetNyes Optional target state to set on the clone
   * @return A new clone of this FIR
   */
  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    throw UnsupportedOperationException(
      s"cloneConstanic not supported for ${getClass.getSimpleName}. " +
      "Subclasses must override this method if they need to be cloneable.")

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
   * - INITIALIZED → CHECKED: Step non-branes only until all are CHECKED
   * - CHECKED → EVALUATING: Immediate transition when detected
   * - EVALUATING → CONSTANT: Step everything (including branes) until all complete
   *
   * Derived classes should override this method and call super.step() as needed.
   *
   * @return 1 for meaningful work, 0 for empty transitions (already evaluated)
   */
  def step(): Int =
    getNyes match
      case Nyes.UNINITIALIZED =>
        // First step: initialize this FIR
        initialize()
        setNyes(Nyes.INITIALIZED)
        1

      case Nyes.INITIALIZED =>
        // Step non-brane expressions until all are CHECKED
        if stepNonBranesUntilState(Nyes.CHECKED) then
          // All non-branes have reached CHECKED
          setNyes(Nyes.CHECKED)
        1

      case Nyes.CHECKED =>
        // Prime braneMind with non-constanic items from braneMemory
        prime()
        setNyes(Nyes.PRIMED)
        1

      case Nyes.PRIMED =>
        // Immediate transition to EVALUATING
        setNyes(Nyes.EVALUATING)
        1

      case Nyes.EVALUATING =>
        // Step everything including sub-branes
        if braneMind.isEmpty then
          // All expressions evaluated, check if any are CONSTANIC
          val anyConstanic = braneMemory.stream.exists(_.atConstanic)
          val newState = if anyConstanic then Nyes.CONSTANIC else Nyes.CONSTANT
          setNyes(newState)
          return 1

        val current = braneMind.dequeue()
        try
          val w = current.step()
          if current.isNye then
            braneMind.enqueue(current)
          w.asInstanceOf[Int]
        catch
          case e: Exception =>
            braneMind.prepend(current) // Re-enqueue on error
            throw RuntimeException("Error during braneMind step execution", e)

      case Nyes.CONSTANT | Nyes.CONSTANIC =>
        // Already evaluated, nothing to do
        0

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
