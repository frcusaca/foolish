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
    enqueueExprs(unaryExpr.expr())

  override def step(): Unit =
    if result.isDefined then
      setNyes(Nyes.CONSTANT)
      return

    // Let parent class handle braneMind stepping and state transitions
    super.step()

    // Check if we can compute the final result
    if getNyes != Nyes.CONSTANT then
      return

    val operandFir = braneMemory.get(0)

    // If operand is abstract (NK), the result is NK
    if operandFir.isAbstract then
      result = Some(NKFiroe(ast, Some("Operand is not-known")))
      return

    try
      val operandValue = operandFir.getValue
      val resultValue = operator match
        case "-" => -operandValue
        case "!" => if operandValue == 0 then 1L else 0L
        case _ => throw UnsupportedOperationException(s"Unknown operator: $operator")

      result = Some(ValueFiroe(ast, resultValue))
    catch
      case e: Exception =>
        result = Some(NKFiroe(ast, Some(Option(e.getMessage).getOrElse(e.getClass.getSimpleName))))

  override def isNye: Boolean =
    result.isEmpty && getNyes != Nyes.CONSTANT

  override def isAbstract: Boolean =
    result.map(_.isAbstract).getOrElse(super.isAbstract)

  override def getValue: Long =
    result.map(_.getValue).getOrElse(
      throw IllegalStateException("UnaryFiroe not fully evaluated"))

  override def toString: String =
    result.map(_.toString).getOrElse(s"$operator${if braneMemory.isEmpty then "?" else braneMemory.get(0).toString}")

object UnaryFiroe:
  def apply(unaryExpr: AST.UnaryExpr): UnaryFiroe =
    new UnaryFiroe(unaryExpr)
