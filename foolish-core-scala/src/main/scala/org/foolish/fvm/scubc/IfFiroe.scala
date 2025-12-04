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

  override def step(): Unit =
    if result.isDefined then
      setNyes(Nyes.CONSTANT)
      return

    // Let parent class handle state transitions first
    super.step()

    if getNyes != Nyes.CONSTANT then
      return

    braneMemory.get(nextPossibleIdx) match
      case cfiroe: ConditionalFiroe =>
        if cfiroe.hasTrueCondition then
          step()
          val thenFir = cfiroe.getThenFir
          if !thenFir.isNye then
            result = Some(thenFir)
        else if cfiroe.hasFalseCondition then
          nextPossibleIdx += 1
        else
          // condition is nye, need to step further
          cfiroe.step()

      case firoe: FIR =>
        // Else branch (explicitly provided or implicit ???) has been fully evaluated
        nextPossibleIdx = braneMemory.size - 1
        result = Some(firoe)

  override def isNye: Boolean =
    result.isEmpty || getNyes != Nyes.CONSTANT

  /** Get the result of the if expression */
  def getResult: Option[FIR] = result

  /**
   * ConditionalFiroe - private to ensure nothing else can insert into the else branch
   */
  private class ConditionalFiroe(ifExpr: AST.IfExpr)
    extends FiroeWithBraneMind(ifExpr):

    private var conditionValue: Option[Boolean] = None

    enqueueSubfirOfExprs(ifExpr.condition(), ifExpr.thenExpr())

    override protected def initialize(): Unit =
      super.initialize()

    override def step(): Unit =
      super.step()
      // After stepping, check if condition has been evaluated
      if conditionValue.isEmpty && !braneMemory.isEmpty && braneMemory.get(0).getNyes == Nyes.CONSTANT then
        val conditionFir = braneMemory.get(0)
        if !conditionFir.isAbstract then
          val condValue = conditionFir.getValue
          conditionValue = Some(condValue != 0)
        else
          // Condition is NK, treat as false
          conditionValue = Some(false)

    override def isNye: Boolean =
      hasUnknownCondition || (hasTrueCondition && super.isNye)

    def hasUnknownCondition: Boolean = conditionValue.isEmpty
    def hasTrueCondition: Boolean = conditionValue.contains(true)
    def hasFalseCondition: Boolean = conditionValue.contains(false)
    def getThenFir: FIR = braneMemory.getLast

  end ConditionalFiroe

end IfFiroe

object IfFiroe:
  def apply(ifExpr: AST.IfExpr): IfFiroe =
    new IfFiroe(ifExpr)
