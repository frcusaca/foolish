package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * Human-friendly sequencer that formats FIR objects with configurable indentation.
 */
case class Sequencer4Human(tabChar: String = "＿") extends Sequencer[String]:

  def sequence(fir: FIR, depth: Int): String = fir match
    case brane: BraneFiroe => sequenceBrane(brane, depth)
    case _: NKFiroe => sequenceNK(fir, depth)
    case value: ValueFiroe => sequenceValue(value, depth)
    case binary: BinaryFiroe => sequenceBinary(binary, depth)
    case unary: UnaryFiroe => sequenceUnary(unary, depth)
    case ifFiroe: IfFiroe => sequenceIf(ifFiroe, depth)
    case searchUp: SearchUpFiroe => sequenceSearchUp(searchUp, depth)
    case assignment: AssignmentFiroe => sequenceAssignment(assignment, depth)
    case _ => indent(depth) + "???"

  protected def sequenceBrane(brane: BraneFiroe, depth: Int): String =
    val sb = StringBuilder()

    // Add characterization (name) if present
    brane.ast match
      case braneAst: AST.Brane
        if braneAst.characterization() != null && braneAst.canonicalCharacterization().nonEmpty =>
        sb.append(indent(depth)).append(braneAst.canonicalCharacterization()).append("'")
      case _ =>
        sb.append(indent(depth))

    sb.append("{\n")

    brane.getExpressionFiroes.foreach { expr =>
      sb.append(sequence(expr, depth + 1))
      sb.append(";\n")
    }

    sb.append(indent(depth)).append("}")
    sb.toString

  protected def sequenceValue(value: ValueFiroe, depth: Int): String =
    indent(depth) + value.getValue.toString

  protected def sequenceBinary(binary: BinaryFiroe, depth: Int): String =
    // If the binary expression has been fully evaluated
    if !binary.isNye then
      // Check if the result is NK (not-known)
      if binary.isAbstract then
        indent(depth) + "???"
      else
        indent(depth) + binary.getValue.toString
    else
      // Show the expression structure (shouldn't normally happen in evaluated branes)
      indent(depth) + binary.toString

  protected def sequenceUnary(unary: UnaryFiroe, depth: Int): String =
    // If the unary expression has been fully evaluated, just show the result
    if !unary.isNye then
      indent(depth) + unary.getValue.toString
    else
      // Show the expression structure (shouldn't normally happen in evaluated branes)
      indent(depth) + unary.toString

  protected def sequenceIf(ifFiroe: IfFiroe, depth: Int): String =
    // If the if expression has been fully evaluated, show the result
    if !ifFiroe.isNye then
      ifFiroe.getResult.map(result => sequence(result, depth)).getOrElse(indent(depth) + "if ???")
    else
      // Show the if structure (shouldn't normally happen in evaluated branes)
      indent(depth) + "if ???"

  protected def sequenceSearchUp(searchUp: SearchUpFiroe, depth: Int): String =
    indent(depth) + "↑"

  protected def sequenceAssignment(assignment: AssignmentFiroe, depth: Int): String =
    if !assignment.isNye && assignment.getResult.isDefined then
      val result = assignment.getResult.get
      // Check if the result is fully evaluated
      if !result.isNye then
        // Check if the result is NK (not-known)
        if result.isAbstract then
          indent(depth) + s"${assignment.getId} = ???"
        else
          indent(depth) + s"${assignment.getId} = ${result.getValue}"
      else
        indent(depth) + s"${assignment.getId} = ???"
    else
      // If not yet evaluated, show the structure
      indent(depth) + s"${assignment.getId} = ???"

  protected def sequenceNK(nk: FIR, depth: Int): String =
    indent(depth) + "???"

  private def indent(depth: Int): String = tabChar * depth

  def getTabChar: String = tabChar
