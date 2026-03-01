package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * BinaryFiroe represents a binary expression in the UBC system.
 * Arithmetic errors (division by zero, etc.) result in NK (not-known) values.
 */
class BinaryFiroe(binaryExpr: AST.BinaryExpr) extends FiroeWithBraneMind(binaryExpr):

  private val operator = binaryExpr.op()
  private var result: Option[FIR] = None

  override protected def initialize(): Unit =
    super.initialize()
    storeExprs(binaryExpr.left(), binaryExpr.right())

  override def step(): Int =
    if result.isDefined then
      setNyes(Nyes.CONSTANT)
      return 0

    getNyes match
      case Nyes.UNINITIALIZED | Nyes.INITIALIZED | Nyes.CHECKED | Nyes.PRIMED =>
        // Let parent handle state progression through these phases
        super.step()

      case Nyes.EVALUATING =>
        // Step operands through evaluation
        if braneMind.isEmpty then
          // All operands evaluated, compute result
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
    val leftFir = braneMemory.get(0)
    val rightFir = braneMemory.get(1)

    // If either operand is at CONSTANIC (unresolved, not CONSTANT), the result is constanic.
    // We do NOT convert to NK. result stays None.
    // Use atConstanic() to check for exactly CONSTANIC state, not CONSTANT.
    if leftFir.atConstanic || rightFir.atConstanic then
      setNyes(Nyes.CONSTANIC)
      return

    try
      val left = leftFir.getValue
      val right = rightFir.getValue

      // Handle division and modulo by zero -> NK
      if (operator == "/" || operator == "%") && right == 0 then
        val errorMsg = if operator == "/" then "Division by zero" else "Modulo by zero"
        result = Some(NKFiroe(ast, Some(errorMsg)))
        setNyes(Nyes.CONSTANT)
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
        throw IllegalStateException("BinaryFiroe is Constanic (unresolved)")
      else
        throw IllegalStateException("BinaryFiroe not fully evaluated")
    else
      result.get.getValue

  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    if !isConstanic then
      throw IllegalStateException(
        s"cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
        s"but this FIR is in state: ${getNyes}")
    if isConstant then
      return this  // Share CONSTANT binary expressions completely
    // CONSTANIC: create fresh copy from AST
    val copy = new BinaryFiroe(ast.asInstanceOf[AST.BinaryExpr])
    copy.setParentFir(newParent)
    // Reset result for re-evaluation
    copy.result = None
    copy.setNyes(getNyes)
    // Set target state if specified
    targetNyes.foreach(copy.setNyes)
    copy

object BinaryFiroe:
  def apply(binaryExpr: AST.BinaryExpr): BinaryFiroe =
    new BinaryFiroe(binaryExpr)
