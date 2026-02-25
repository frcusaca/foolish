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

  private val sourceElements: List[AST.Expr] = ast.elements().asScala.toList
  private var stageAExecutor: ExecutionFir = null
  private var sourceFirs: List[FIR] = null
  private var joinComplete = false

  /**
   * Copy constructor for cloneConstanic.
   */
  private def this(original: ConcatenationFiroe, newParent: FIR) =
    this(original.ast)
    setParentFir(newParent)
    sourceFirs = null
    joinComplete = true

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

      fir match
        case fwbm: FiroeWithBraneMind =>
          if !fwbm.isInstanceOf[BraneFiroe] && !fwbm.isInstanceOf[ConcatenationFiroe] then
            indexLookup.put(fir, index)
            fwbm.ordinateToParentBraneMind(this, index)
        case _ =>
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
    sourceFirs.foreach { fir =>
      val resolved = FIR.unwrapConstanicable(fir)

      resolved match
        case fwbm: FiroeWithBraneMind =>
          // Use a fold to skip non-constanic statements
          fwbm.stream.foreach { statement =>
            if statement.isConstanic then
              val targetState = if statement.isConstant then
                None
              else
                Some(Nyes.INITIALIZED)

              val cloned = statement.asInstanceOf[Constanicable].cloneConstanic(this, targetState)

              val isNewClone = cloned ne statement
              if isNewClone && cloned.isInstanceOf[FiroeWithBraneMind] then
                cloned.asInstanceOf[FiroeWithBraneMind].ordinated = false

              storeFirs(cloned)
          }

        case _ =>
          throw new IllegalStateException(
            s"Concatenation element resolved to non-brane: ${resolved.getClass.getSimpleName}. " +
            "Concatenation can only contain branes.")
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
      Sequencer4Human().sequence(this)

object ConcatenationFiroe:
  def apply(concatenation: AST.Concatenation): ConcatenationFiroe =
    new ConcatenationFiroe(concatenation)
