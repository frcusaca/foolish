package org.foolish.fvm.scubc

import org.foolish.ast.AST
import scala.jdk.CollectionConverters.*

/**
 * ConcatenationFiroe combines multiple branes/expressions into a single evaluation context.
 *
 * Semantics:
 * - Each element is first evaluated to PRIMED in its original context
 * - Then all elements are cloned and re-parented into this concatenation's braneMemory
 * - Later elements can resolve identifiers from earlier elements
 *
 * Stage A: UNINITIALIZED -> INITIALIZED -> CHECKED
 *   - Create FIRs from source elements
 *   - Use ExecutionFir to step all FIRs to CONSTANIC state (breadth-first)
 *   - FIRs retain their original parents during this stage
 *
 * Stage B: CHECKED -> PRIMED
 *   - Clone and re-parent all source FIRs into braneMemory
 *   - Brane becomes searchable at end of this stage
 *
 * Stage C: PRIMED -> EVALUATING -> CONSTANT
 *   - Normal evaluation with shared memory context
 */
class ConcatenationFiroe(override val ast: AST.Concatenation)
  extends FiroeWithBraneMind(ast) with Constanicable:

  private var sourceElements: List[AST.Expr] = ast.elements().asScala.toList
  private var stageAExecutor: ExecutionFir = null
  private var sourceFirs: List[FIR] = null
  private var joinComplete = false

  /**
   * Copy constructor for cloneConstanic.
   * Creates a copy with independent braneMemory and updated parent chain.
   *
   * @param original the ConcatenationFiroe to copy
   * @param newParent the new parent for this clone
   */
  private def this(original: ConcatenationFiroe, newParent: FIR) =
    this(original.ast)
    setParentFir(newParent)
    // Copy braneMemory contents with cloned items
    original.braneMemory.stream.foreach { fir =>
      val cloned = fir.asInstanceOf[Constanicable].cloneConstanic(this, Some(Nyes.INITIALIZED))
      braneMemory.put(cloned)
      indexLookup.put(cloned, braneMemory.size - 1)
      // For nested FiroeWithBraneMind instances, set up parent memory link
      if cloned.isInstanceOf[FiroeWithBraneMind] then
        val fwbm = cloned.asInstanceOf[FiroeWithBraneMind]
        // Reset ordinated flag so we can re-ordinate in new context
        fwbm.ordinated = false
        // Use the NEW index in the concatenated brane, not the original index.
        // This ensures that when the cloned brane searches parent memories,
        // it uses its position in the concatenated brane (not the original brane).
        fwbm.ordinateToParentBraneMind(this, indexLookup.size - 1)
    }
    // Set myPos to -1 (no limit) so that nested branes can search the entire
    // concatenated brane when looking up identifiers. The original brane's
    // position is not relevant here because the cloned branes are now part of
    // a new concatenated context.
    // Copy source elements for later operations
    sourceElements = original.sourceElements
    stageAExecutor = null
    sourceFirs = null
    joinComplete = true
    // Set state to INITIALIZED so children can re-evaluate
    setNyes(Nyes.INITIALIZED)

  /**
   * Checks if this concatenation is ready for LHS identifier searches.
   * A concatenation becomes searchable when it reaches PRIMED state,
   * meaning all component branes have been cloned and stitched together.
   */
  def isLhsSearchable: Boolean =
    getNyes.ordinal >= Nyes.PRIMED.ordinal

  override protected def initialize(): Unit =
    if isInitialized then return
    setInitialized()

    sourceFirs = List.empty
    var index = 0
    sourceElements.foreach { element =>
      val fir = FIR.createFiroeFromExpr(element)
      fir.setParentFir(this)

      // Only ordinate non-brane elements (identifiers, searches, etc.)
      // Branes are kept isolated so their contents don't resolve to outer scope
      // This matches Java's "concatenates before resolution" semantics.
      if fir.isInstanceOf[FiroeWithBraneMind] then
        val fwbm = fir.asInstanceOf[FiroeWithBraneMind]
        if !fwbm.isInstanceOf[BraneFiroe] && !fwbm.isInstanceOf[ConcatenationFiroe] then
          // Non-brane FiroeWithBraneMind (like IdentifierFiroe) - ordinate for resolution
          indexLookup.put(fir, index)
          fwbm.ordinateToParentBraneMind(this, index)
      sourceFirs = sourceFirs :+ fir
      index += 1
    }


    stageAExecutor = ExecutionFir.stepping(sourceFirs*)
      .setParent(false)
      .stepUntil(Nyes.CONSTANIC)
      .build()

  override def step(): Int =
    getNyes match
      case Nyes.UNINITIALIZED =>
        initialize()
        setNyes(Nyes.INITIALIZED)
        1

      case Nyes.INITIALIZED =>
        // Stage A: Use ExecutionFir to step source FIRs to CONSTANIC (breadth-first)
        if !stageAExecutor.isConstanic then
          stageAExecutor.step()
          return 1

        // ExecutionFir finished - check result
        if stageAExecutor.isStuck then
          // Some FIRs couldn't reach CONSTANIC - we become CONSTANIC
          setNyes(Nyes.CONSTANIC)
        else
          // All FIRs reached CONSTANIC - proceed to Stage B
          setNyes(Nyes.CHECKED)
        1

      case Nyes.CHECKED =>
        // Stage B: Clone and re-parent all source FIRs into braneMemory
        performJoin()
        prime()
        setNyes(Nyes.PRIMED)
        1

      case Nyes.PRIMED =>
        setNyes(Nyes.EVALUATING)
        1

      case Nyes.EVALUATING =>
        super.step()

      case Nyes.CONSTANT | Nyes.CONSTANIC =>
        // Already evaluated, nothing to do
        0

  private def performJoin(): Unit =
    System.out.println(s"DEBUG performJoin: THIS ConcatenationFiroe hashCode=${System.identityHashCode(this)}")
    // Track the maximum original position of component branes for search scope limitation
    var maxOriginalPos = -1

    sourceFirs.foreach { fir =>
      val resolved = FIR.unwrapConstanicable(fir)
      System.out.println(s"DEBUG performJoin: fir=${fir.getClass.getSimpleName}, resolved=${resolved.getClass.getSimpleName}, resolvedState=${resolved.getNyes}")

      resolved match
        case fwbm: FiroeWithBraneMind =>
          val stmtCount = fwbm.braneMemory.size
          System.out.println(s"DEBUG performJoin: fwbm.stream.size = $stmtCount")
          // Get the original position of this brane from the sourceFirs index
          val originalIndex = sourceFirs.indexOf(fir)
          System.out.println(s"DEBUG performJoin: originalIndex for ${fwbm.getClass.getSimpleName} = $originalIndex")
          if originalIndex > maxOriginalPos then
            maxOriginalPos = originalIndex

          // Flatten: iterate over the brane's statements and clone each one
          // into this concatenation's braneMemory
          fwbm.stream.foreach { statement =>
            // Only clone constanic statements (CONSTANIC or CONSTANT)
            // Statements not yet at constanic should be skipped and left in their original brane.
            // This matches Java's behavior where non-constanic statements remain unresolved.
            if !statement.isConstanic then
              // Skip non-constanic statements - they remain in their original brane
              return
            // Clone each statement with this concatenation as new parent
            // CONSTANT items stay CONSTANT (they're fully resolved, immutable).
            // CONSTANIC items reset to INITIALIZED to re-evaluate in new context.
            val targetState = if statement.isConstant then
              None  // Keep CONSTANT as-is
            else
              Some(Nyes.INITIALIZED)  // Reset CONSTANIC to re-evaluate

            val cloned = statement.asInstanceOf[Constanicable].cloneConstanic(this, targetState)

            // Only reset ordinated flag for FIRs that are ACTUALLY NEW clones.
            // If cloned == statement, the object is being shared (CONSTANT FIRs)
            // and should NOT be re-ordinated - it belongs to its original parent.
            val isNewClone = cloned ne statement
            System.out.println(s"DEBUG performJoin: targetState=$targetState, cloned=${cloned.getClass.getSimpleName}, clonedState=${cloned.getNyes}, isNewClone=$isNewClone")
            System.out.println(s"DEBUG performJoin: this=${System.identityHashCode(this)} this.braneMemory=${System.identityHashCode(this.braneMemory)}")

            storeFirs(cloned)
            if isNewClone && cloned.isInstanceOf[FiroeWithBraneMind] then
              val clonedFwbm = cloned.asInstanceOf[FiroeWithBraneMind]
              clonedFwbm.ordinated = false
          }

        case _ =>
          throw new IllegalStateException(
            s"Concatenation element resolved to non-brane: ${resolved.getClass.getSimpleName}. " +
            "Concatenation can only contain branes.")
    }

    val currentMyPos = braneMemory.getMyPos
    System.out.println(s"DEBUG performJoin: maxOriginalPos = $maxOriginalPos braneMemory.myPos=$currentMyPos")

    System.out.println(s"DEBUG performJoin: AFTER STORE - braneMemory.size=${braneMemory.size}")
    braneMemory.stream.foreach { item =>
      System.out.println(s"DEBUG performJoin:   braneMemory item=$item class=${item.getClass.getSimpleName}")
    }

    sourceFirs = null
    stageAExecutor = null
    joinComplete = true

  override def isNye: Boolean =
    getNyes != Nyes.CONSTANT && getNyes != Nyes.CONSTANIC

  override def isAbstract: Boolean =
    // Concatenations are not abstract - they're containers that hold statements
    // from other branes. The statements themselves may be abstract, but the
    // concatenation as a container is not.
    false

  override def getResult: FIR = this

  override def getValue: Long =
    throw UnsupportedOperationException("getValue not supported for ConcatenationFiroe")

  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    if !isConstanic then
      throw IllegalStateException(
        s"cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
        s"but this FIR is in state: ${getNyes}")
    if isConstant then
      return this
    val copy = new ConcatenationFiroe(this, newParent)
    targetNyes.foreach(copy.setNyes)
    copy

  override def toString: String =
    if !joinComplete then
      val pendingCount = if sourceFirs != null then sourceFirs.size else 0
      s"ConcatenationFiroe[pending=$pendingCount, state=${getNyes}]"
    else
      System.out.println(s"DEBUG ConcatenationFiroe.toString: this=${System.identityHashCode(this)} state=${getNyes} braneMemory.size=${braneMemory.size}")
      System.out.println(s"DEBUG ConcatenationFiroe.toString: stream contents:")
      braneMemory.stream.foreach { fir =>
        System.out.println(s"DEBUG   - $fir (${fir.getClass.getSimpleName}) state=${fir.getNyes}")
      }
      Sequencer4Human().sequence(this)

object ConcatenationFiroe:
  def apply(concatenation: AST.Concatenation): ConcatenationFiroe =
    new ConcatenationFiroe(concatenation)
