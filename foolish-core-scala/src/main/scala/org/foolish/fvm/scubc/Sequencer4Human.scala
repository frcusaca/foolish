package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * Human-friendly sequencer that formats FIR objects with configurable indentation.
 */
object Sequencer4Human {
  val NK_STR = "???"
  val CC_STR = "⎵⎵"
}

case class Sequencer4Human(tabChar: String = "＿") extends Sequencer[String]:
  import Sequencer4Human._

  def sequence(fir: FIR, depth: Int): String = fir match
    case brane: BraneFiroe => sequenceBrane(brane, depth)
    case _: NKFiroe => sequenceNK(fir, depth)
    case value: ValueFiroe => sequenceValue(value, depth)
    case binary: BinaryFiroe => sequenceBinary(binary, depth)
    case unary: UnaryFiroe => sequenceUnary(unary, depth)
    case ifFiroe: IfFiroe => sequenceIf(ifFiroe, depth)
    case searchUp: SearchUpFiroe => sequenceSearchUp(searchUp, depth)
    case assignment: AssignmentFiroe => sequenceAssignment(assignment, depth)
    case identifier: IdentifierFiroe => sequenceIdentifier(identifier, depth)
    case oneShotSearch: OneShotSearchFiroe => sequenceOneShotSearch(oneShotSearch, depth)
    case _ => indent(depth) + NK_STR

  protected def sequenceBrane(brane: BraneFiroe, depth: Int): String =
    val sb = StringBuilder()

    // Add characterization (name) if present
    brane.ast match
      case braneAst: AST.Brane
        if braneAst.canonicalCharacterization().nonEmpty =>
        sb.append(indent(depth)).append(braneAst.canonicalCharacterization())
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
        indent(depth) + NK_STR
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
      ifFiroe.getResult.map(result => sequence(result, depth)).getOrElse(indent(depth) + "if " + NK_STR)
    else
      // Show the if structure (shouldn't normally happen in evaluated branes)
      indent(depth) + "if " + NK_STR

  protected def sequenceSearchUp(searchUp: SearchUpFiroe, depth: Int): String =
    indent(depth) + "↑"

  protected def sequenceAssignment(assignment: AssignmentFiroe, depth: Int): String =
    val fullId = assignment.getLhs.toString
    if !assignment.isNye && assignment.getResult.isDefined then
      val result = assignment.getResult.get
      // Check if the result is fully evaluated
      if !result.isNye then
        if result.atConstanic then
           indent(depth) + s"$fullId = $CC_STR"
        else
          unwrap(result) match
            case constanic if constanic.atConstanic =>
               indent(depth) + s"$fullId = $CC_STR"
            case brane: BraneFiroe =>
              // Special handling for nested branes to align indentation
              // Must check for BraneFiroe BEFORE isAbstract check, since branes
              // containing only CONSTANIC values will have isAbstract=true
              val sequencedBrane = sequence(brane, depth)
              // Strip the first line's indentation
              val indentStr = indent(depth)
              val strippedBrane = if (sequencedBrane.startsWith(indentStr)) then
                sequencedBrane.substring(indentStr.length)
              else
                sequencedBrane

              // Calculate padding for subsequent lines
              // The padding should come BETWEEN the parent depth marker and the nested depth marker
              // Example: "\n＿＿content" becomes "\n＿    ＿content" for "b = " (4 chars)
              val padding = " " * (fullId.length + 3)
              val nestedIndent = indent(depth + 1)  // e.g., "＿＿"
              val parentIndent = indent(depth)       // e.g., "＿"
              // Apply padding between parent and nested indentation
              val alignedBrane = strippedBrane.replace(s"\n$nestedIndent", s"\n$parentIndent$padding$tabChar")

              indent(depth) + s"$fullId = ${alignedBrane}"

            case abstractFir if abstractFir.isAbstract =>
               indent(depth) + s"$fullId = $NK_STR"
            case unwrapped =>
              // Simple values
              indent(depth) + s"$fullId = ${unwrapped.getValue}"
      else
        indent(depth) + s"$fullId = $NK_STR"
    else if assignment.atConstanic then
        indent(depth) + s"$fullId = $CC_STR"
    else
      // If not yet evaluated, show the structure
      indent(depth) + s"$fullId = $NK_STR"

  protected def sequenceIdentifier(identifier: IdentifierFiroe, depth: Int): String =
    // If the identifier has been resolved and is not NYE
    if !identifier.isNye then
      if identifier.atConstanic then
         indent(depth) + CC_STR
      else if identifier.isAbstract then
        indent(depth) + NK_STR
      else
        unwrap(identifier) match
          case constanic if constanic.atConstanic => indent(depth) + CC_STR
          case brane: BraneFiroe =>
             // Should not typically happen for top-level sequencing but good to handle
             sequence(brane, depth)
          case unwrapped =>
             indent(depth) + unwrapped.getValue.toString
    else if identifier.atConstanic then
        indent(depth) + CC_STR
    else
      // If not yet evaluated
      indent(depth) + NK_STR

  protected def sequenceOneShotSearch(oneShotSearch: OneShotSearchFiroe, depth: Int): String =
    if !oneShotSearch.isNye then
      // Check if search found nothing - not found is CONSTANIC
      if !oneShotSearch.isFound then
        indent(depth) + CC_STR
      else if oneShotSearch.atConstant then
        // Found and CONSTANT - use sequence() recursively on the result
        sequence(oneShotSearch.getResult, depth)
      else
        // Search found something but it's CONSTANIC (unresolved)
        indent(depth) + CC_STR
    else
      indent(depth) + NK_STR

  @scala.annotation.tailrec
  private def unwrap(fir: FIR): FIR = fir match
    case assignment: AssignmentFiroe =>
       if assignment.atConstanic then assignment
       else if assignment.getResult.isDefined then unwrap(assignment.getResult.get)
       else assignment
    case identifier: IdentifierFiroe =>
       if identifier.atConstanic then identifier
       else if identifier.value != null then unwrap(identifier.value)
       else identifier
    case abstractSearch: AbstractSearchFiroe if abstractSearch.getResult != null =>
       unwrap(abstractSearch.getResult)
    case unanchoredSeek: UnanchoredSeekFiroe if unanchoredSeek.getResult != null =>
       unwrap(unanchoredSeek.getResult)
    case other => other

  protected def sequenceNK(nk: FIR, depth: Int): String =
    indent(depth) + NK_STR

  private def indent(depth: Int): String = tabChar * depth

  def getTabChar: String = tabChar
