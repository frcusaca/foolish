package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * UnaryFiroe represents a unary expression in the UBC system.
 * Arithmetic errors during evaluation result in NK (not-known) values.
 */
class UnaryFiroe(unaryExpr: AST.UnaryExpr) extends FiroeWithBraneMind(unaryExpr):

  private val operator = unaryExpr.op()
  private var operandFiroe: Option[FIR] = None
  private var operandCreated = false
  private var result: Option[FIR] = None

  protected def initialize(): Unit =
    setInitialized()
    enqueueExprs(unaryExpr.expr())

  override def step(): Unit =
    if result.isDefined then
      return

    if !operandCreated then
      operandFiroe = Some(FIR.createFiroeFromExpr(unaryExpr.expr()))
      operandCreated = true
      enqueueFirs(operandFiroe.get)
      return

    // Let parent class handle braneMind stepping
    super.step()

    // Check if we can compute the final result
    if super.isNye then
      return

    // If operand is abstract (NK), the result is NK
    if operandFiroe.get.isAbstract then
      result = Some(NKFiroe(ast, Some("Operand is not-known")))
      return

    try
      val operandValue = operandFiroe.get.getValue
      val resultValue = operator match
        case "-" => -operandValue
        case "!" => if operandValue == 0 then 1L else 0L
        case _ => throw UnsupportedOperationException(s"Unknown operator: $operator")

      result = Some(ValueFiroe(ast, resultValue))
    catch
      case e: Exception =>
        result = Some(NKFiroe(ast, Some(Option(e.getMessage).getOrElse(e.getClass.getSimpleName))))

  override def isNye: Boolean = result.isEmpty

  override def isAbstract: Boolean =
    result.map(_.isAbstract).getOrElse(operandFiroe.exists(_.isAbstract))

  override def getValue: Long =
    result.map(_.getValue).getOrElse(
      throw IllegalStateException("UnaryFiroe not fully evaluated"))

  override def toString: String =
    result.map(_.toString).getOrElse(s"$operator${operandFiroe.map(_.toString).getOrElse("?")}")

object UnaryFiroe:
  def apply(unaryExpr: AST.UnaryExpr): UnaryFiroe =
    new UnaryFiroe(unaryExpr)
