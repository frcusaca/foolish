package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * IfFiroe represents an if-else expression in the UBC system.
 * Contains a series of Firoes representing conditions and values.
 */
class IfFiroe(ifExpr: AST.IfExpr) extends FiroeWithBraneMind(ifExpr):

  private var result: Option[FIR] = None
  protected var nextPossibleIdx = 0

  override protected def initialize(): Unit =
    super.initialize()

    // Create Firoes for condition, then, and else branches
    enqueueSubfirOfExprs(ifExpr)
    enqueueSubfirOfExprs(ifExpr.condition(), ifExpr.thenExpr())

    import scala.jdk.CollectionConverters.*
    ifExpr.elseIfs().asScala.foreach { elseIf =>
      enqueueFirs(ConditionalFiroe(elseIf))
    }

    // Enqueue else branch - if not present (UnknownExpr), use NK (???)
    if ifExpr.elseExpr() == AST.UnknownExpr.INSTANCE || ifExpr.elseExpr() == null then
      enqueueFirs(NKFiroe(null, Some("No matching condition in if-elif chain")))
    else
      enqueueFirs(FIR.createFiroeFromExpr(ifExpr.elseExpr()))

    nextPossibleIdx = 0

  override def step(): Int =
    if result.isDefined then
      setNyes(Nyes.CONSTANT)
      return 0

    getNyes match
      case Nyes.UNINITIALIZED | Nyes.INITIALIZED | Nyes.CHECKED | Nyes.PRIMED =>
        // Let parent handle state progression through these phases
        super.step()
        1

      case Nyes.EVALUATING =>
        // Step through conditionals to find the matching branch
        while nextPossibleIdx < braneMemory.size do
          braneMemory.get(nextPossibleIdx) match
            case cfiroe: ConditionalFiroe =>
              if cfiroe.hasTrueCondition then
                // Condition is true, evaluate then branch
                val thenFir = cfiroe.getThenFir
                val work =
                  if thenFir.isNye then
                    thenFir.step()
                  else
                    0
                if !thenFir.isNye then
                  result = Some(thenFir)
                  return work
              else if cfiroe.hasFalseCondition then
                // Condition is false, move to next conditional
                nextPossibleIdx += 1
                1
              else
                // Condition is still evaluating, step it
                cfiroe.step()

            case firoe: FIR =>
              // This is the else branch or NK
              val work =
                if firoe.isNye then
                  firoe.step()
                else
                  0
              if !firoe.isNye then
                nextPossibleIdx = braneMemory.size - 1
                result = Some(firoe)
                return work
              return 1  // Still evaluating, need another step

            case _ =>
              return 1  // Unknown, wait for next step

          if result.isEmpty then
            return 1  // Need another step to continue

        1

      case Nyes.CONSTANT | Nyes.CONSTANIC =>
        // Already evaluated, nothing to do
        0

  override def isNye: Boolean =
    result.isEmpty && getNyes != Nyes.CONSTANT && getNyes != Nyes.CONSTANIC

  /** Get the result of the if expression */
  override def getResult: FIR = result.orNull

  /**
   * ConditionalFiroe - private to ensure nothing else can insert into the else branch
   */
  private class ConditionalFiroe(ifExpr: AST.IfExpr)
    extends FiroeWithBraneMind(ifExpr):

    private var conditionValue: Option[Boolean] = None

    enqueueSubfirOfExprs(ifExpr.condition(), ifExpr.thenExpr())

    override protected def initialize(): Unit =
      super.initialize()

    override def step(): Int =
      getNyes match
        case Nyes.UNINITIALIZED | Nyes.INITIALIZED | Nyes.CHECKED | Nyes.PRIMED =>
          super.step()
          1
        case Nyes.EVALUATING =>
          // Step operands
          if braneMind.isEmpty then
            // All operands evaluated, check condition value
            if conditionValue.isEmpty && !braneMemory.isEmpty then
              val conditionFir = braneMemory.get(0)
              if !conditionFir.isAbstract && !conditionFir.isNye then
                val condValue = conditionFir.getValue
                conditionValue = Some(condValue != 0)
              else if conditionFir.isNye then
                conditionFir.step()
              else
                // Condition is NK (not known), treat as false
                conditionValue = Some(false)
            1
          else
            val current = braneMind.dequeue()
            current.step()
            if current.isNye then
              braneMind.enqueue(current)
            1
        case Nyes.CONSTANT | Nyes.CONSTANIC =>
          // Already evaluated, nothing to do
          0

    override def isNye: Boolean =
      // Use parent's state-based isNye logic
      super.isNye

    def hasUnknownCondition: Boolean = conditionValue.isEmpty
    def hasTrueCondition: Boolean = conditionValue.contains(true)
    def hasFalseCondition: Boolean = conditionValue.contains(false)
    def getThenFir: FIR = braneMemory.getLast

  end ConditionalFiroe

  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    if !isConstanic then
      throw IllegalStateException(
        s"cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
        s"but this FIR is in state: ${getNyes}")
    if isConstant then
      return this  // Share CONSTANT if expressions completely
    // CONSTANIC: create fresh copy from AST
    val copy = new IfFiroe(ast.asInstanceOf[AST.IfExpr])
    copy.setParentFir(newParent)
    // Reset result for re-evaluation
    copy.result = None
    copy.nextPossibleIdx = 0
    copy.setNyes(getNyes)
    // Set target state if specified
    targetNyes.foreach(copy.setNyes)
    copy

end IfFiroe

object IfFiroe:
  def apply(ifExpr: AST.IfExpr): IfFiroe =
    new IfFiroe(ifExpr)
