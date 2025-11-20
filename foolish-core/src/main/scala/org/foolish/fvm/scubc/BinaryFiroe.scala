package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * BinaryFiroe represents a binary expression in the UBC system.
 * Arithmetic errors (division by zero, etc.) result in NK (not-known) values.
 */
class BinaryFiroe(binaryExpr: AST.BinaryExpr) extends FiroeWithBraneMind(binaryExpr):

  private val operator = binaryExpr.op()
  private var result: Option[FIR] = None

  protected def initialize(): Unit =
    enqueueExprs(binaryExpr.left(), binaryExpr.right())
    setInitialized()

  override def step(): Unit =
    if result.isDefined then
      return

    if !isInitialized then
      initialize()
      return

    // Let parent class handle braneMind stepping
    super.step()

    // Check if we can compute the final result
    if super.isNye then
      return

    val leftFir = braneMemory.remove(0)
    val rightFir = braneMemory.remove(0)

    // If either operand is abstract (NK), the result is NK
    if leftFir.isAbstract || rightFir.isAbstract then
      result = Some(NKFiroe(ast, Some("Operand is not-known")))
      return

    try
      val left = leftFir.getValue
      val right = rightFir.getValue

      // Handle division and modulo by zero -> NK
      if (operator == "/" || operator == "%") && right == 0 then
        val errorMsg = if operator == "/" then "Division by zero" else "Modulo by zero"
        result = Some(NKFiroe(ast, Some(errorMsg)))
        return

      val resultValue = operator match
        case "+" => left + right
        case "-" => left - right
        case "*" => left * right
        case "/" => left / right
        case "%" => left % right
        case "==" => if left == right then 1L else 0L
        case "!=" | "<>" => if left != right then 1L else 0L
        case "<" => if left < right then 1L else 0L
        case "<=" => if left <= right then 1L else 0L
        case ">" => if left > right then 1L else 0L
        case ">=" => if left >= right then 1L else 0L
        case "&&" => if left != 0 && right != 0 then 1L else 0L
        case "||" => if left != 0 || right != 0 then 1L else 0L
        case _ => throw UnsupportedOperationException(s"Unknown operator: $operator")

      result = Some(ValueFiroe(null, resultValue))
    catch
      case e: Exception =>
        result = Some(NKFiroe(ast, Some(Option(e.getMessage).getOrElse(e.getClass.getSimpleName))))

  override def isNye: Boolean = result.isEmpty

  override def isAbstract: Boolean =
    result.map(_.isAbstract).getOrElse(super.isAbstract)

  override def getValue: Long =
    result.map(_.getValue).getOrElse(
      throw IllegalStateException("BinaryFiroe not fully evaluated"))

object BinaryFiroe:
  def apply(binaryExpr: AST.BinaryExpr): BinaryFiroe =
    new BinaryFiroe(binaryExpr)
