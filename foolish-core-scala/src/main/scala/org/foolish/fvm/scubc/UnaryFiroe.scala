package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * UnaryFiroe represents a unary expression in the UBC system.
 * Arithmetic errors during evaluation result in NK (not-known) values.
 */
class UnaryFiroe(unaryExpr: AST.UnaryExpr) extends FiroeWithBraneMind(unaryExpr):

  private val operator = unaryExpr.op()
  private var result: Option[FIR] = None

  override protected def initialize(): Unit =
    super.initialize()
    storeExprs(unaryExpr.expr())

  override def step(): Int =
    if result.isDefined then
      setNyes(Nyes.CONSTANT)
      return 0

    getNyes match
      case Nyes.UNINITIALIZED | Nyes.INITIALIZED | Nyes.CHECKED | Nyes.PRIMED =>
        // Let parent handle state progression through these phases
        super.step()

      case Nyes.EVALUATING =>
        // Step operand through evaluation
        if braneMind.isEmpty then
          // Operand evaluated, compute result
          computeResult()
          1
        else
          // Step the next operand
          val current = braneMind.dequeue()
          try
            val work = current.step()
            if current.isNye then
              braneMind.enqueue(current)
            work
          catch
            case e: Exception =>
              braneMind.prepend(current) // Re-enqueue on error
              throw RuntimeException("Error during operand evaluation", e)

      case Nyes.CONSTANT | Nyes.CONSTANIC =>
        // Already evaluated, nothing to do
        0

  private def computeResult(): Unit =
    val operandFir = braneMemory.get(0)

    // If operand is at CONSTANIC (unresolved, not CONSTANT), the result is constanic.
    // Use atConstanic() to check for exactly CONSTANIC state, not CONSTANT.
    if operandFir.atConstanic then
      setNyes(Nyes.CONSTANIC)
      return

    try
      val operandValue = operandFir.getValue
      val resultValue = operator match
        case "-" => -operandValue
        case "!" => if operandValue == 0 then 1L else 0L
        case _ => throw UnsupportedOperationException(s"Unknown operator: $operator")

      result = Some(ValueFiroe(ast, resultValue))
      setNyes(Nyes.CONSTANT)
    catch
      case e: Exception =>
        result = Some(NKFiroe(ast, Some(Option(e.getMessage).getOrElse(e.getClass.getSimpleName))))
        setNyes(Nyes.CONSTANT)

  override def isNye: Boolean =
    result.isEmpty && getNyes != Nyes.CONSTANT && getNyes != Nyes.CONSTANIC

  override def isAbstract: Boolean =
    result.map(_.isAbstract).getOrElse(super.isAbstract)

  override def getValue: Long =
    if result.isEmpty then
      // If result is None, we are Constanic (unresolved).
      // Calling getValue() on an unresolved expression is an error.
      if isConstanic then
        throw IllegalStateException("UnaryFiroe is Constanic (unresolved)")
      else
        throw IllegalStateException("UnaryFiroe not fully evaluated")
    else
      result.get.getValue

  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    if !isConstanic then
      throw IllegalStateException(
        s"cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
        s"but this FIR is in state: ${getNyes}")
    if isConstant then
      return this  // Share CONSTANT unary expressions completely
    // CONSTANIC: create fresh copy from AST
    val copy = new UnaryFiroe(ast.asInstanceOf[AST.UnaryExpr])
    copy.setParentFir(newParent)
    // Reset result for re-evaluation
    copy.result = None
    copy.setNyes(getNyes)
    // Set target state if specified
    targetNyes.foreach(copy.setNyes)
    copy

  override def toString: String =
    result.map(_.toString).getOrElse(s"$operator${if braneMemory.isEmpty then "?" else braneMemory.get(0).toString}")

object UnaryFiroe:
  def apply(unaryExpr: AST.UnaryExpr): UnaryFiroe =
    new UnaryFiroe(unaryExpr)
