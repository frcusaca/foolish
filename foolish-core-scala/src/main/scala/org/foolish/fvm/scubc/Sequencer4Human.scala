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

  private var nyesStateInOutput: Boolean = true // Default is on

  def setNyesStateInOutput(enabled: Boolean): Unit =
    nyesStateInOutput = enabled

  def isNyesStateInOutputEnabled: Boolean =
    nyesStateInOutput

  private def addNyesStateIfEnabled(value: String, nyes: Nyes): String =
    if nyesStateInOutput then
      s"$value ($nyes)"
    else
      value

  def sequence(fir: FIR, depth: Int): String = fir match
    case brane: BraneFiroe => sequenceBrane(brane, depth)
    case concat: ConcatenationFiroe => sequenceConcatenation(concat, depth)
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
      // Check if the result is constanic (unresolved)
      if binary.atConstanic then
        indent(depth) + addNyesStateIfEnabled(CC_STR, binary.getNyes)
      else if binary.isAbstract then
        indent(depth) + NK_STR
      else
        try
          indent(depth) + binary.getValue.toString
        catch
          case _: IllegalStateException => indent(depth) + NK_STR
    else
      // Show the expression structure (shouldn't normally happen in evaluated branes)
      indent(depth) + binary.toString

  protected def sequenceUnary(unary: UnaryFiroe, depth: Int): String =
    // If the unary expression has been fully evaluated, just show the result
    if !unary.isNye then
      // Check if the result is constanic (unresolved)
      if unary.atConstanic then
        indent(depth) + addNyesStateIfEnabled("⎵", unary.getNyes)
      else
        try
          indent(depth) + unary.getValue.toString
        catch
          case _: IllegalStateException => indent(depth) + NK_STR
    else
      // Show the expression structure (shouldn't normally happen in evaluated branes)
      indent(depth) + unary.toString

  protected def sequenceIf(ifFiroe: IfFiroe, depth: Int): String =
    // If the if expression has been fully evaluated, show the result
    if !ifFiroe.isNye then
      val result = ifFiroe.getResult
      if result != null then
        sequence(result, depth)
      else
        indent(depth) + "if " + NK_STR
    else
      // Show the if structure (shouldn't normally happen in evaluated branes)
      indent(depth) + "if " + NK_STR

  protected def sequenceSearchUp(searchUp: SearchUpFiroe, depth: Int): String =
    indent(depth) + "↑"

  protected def sequenceAssignment(assignment: AssignmentFiroe, depth: Int): String =
    val fullId = assignment.getLhs.toString
    val result = assignment.getResult
    if !assignment.isNye && result != null then
      // Check if the result is fully evaluated
      if !result.isNye then
        // Unwrap to get the actual result
        val unwrapped = unwrap(result)
        if unwrapped != null then
          // Check if constanic, but exclude BraneFiroe and ConcatenationFiroe (Java behavior)
          // This allows branes/concatenations to be sequenced even when they're constanic
          if unwrapped.atConstanic
             && !unwrapped.isInstanceOf[BraneFiroe]
             && !unwrapped.isInstanceOf[ConcatenationFiroe] then
            indent(depth) + addNyesStateIfEnabled(s"$fullId = $CC_STR", unwrapped.getNyes)
          else if unwrapped.isInstanceOf[BraneFiroe] then
            // Special handling for nested branes to align indentation
            val sequencedBrane = sequence(unwrapped.asInstanceOf[BraneFiroe], depth)
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

          else if unwrapped.isInstanceOf[ConcatenationFiroe] then
            // Handle concatenation result - sequence it properly
            val sequencedConcat = sequence(unwrapped.asInstanceOf[ConcatenationFiroe], depth)
            // Strip the first line's indentation
            val indentStr = indent(depth)
            val strippedConcat = if (sequencedConcat.startsWith(indentStr)) then
              sequencedConcat.substring(indentStr.length)
            else
              sequencedConcat

            // Calculate padding for subsequent lines
            val padding = " " * (fullId.length + 3)
            val nestedIndent = indent(depth + 1)  // e.g., "＿＿"
            val parentIndent = indent(depth)       // e.g., "＿"
            // Apply padding between parent and nested indentation
            val alignedConcat = strippedConcat.replace(s"\n$nestedIndent", s"\n$parentIndent$padding$tabChar")

            indent(depth) + s"$fullId = ${alignedConcat}"

          else if unwrapped.isAbstract then
            indent(depth) + s"$fullId = $NK_STR"
          else
            // Simple values
            indent(depth) + s"$fullId = ${unwrapped.getValue}"
        else
          indent(depth) + s"$fullId = $NK_STR"
      else
        indent(depth) + s"$fullId = $NK_STR"
    else if assignment.atConstanic then
        indent(depth) + addNyesStateIfEnabled(s"$fullId = $CC_STR", assignment.getNyes)
    else
      // If not yet evaluated, show the structure
      indent(depth) + s"$fullId = $NK_STR"

  protected def sequenceIdentifier(identifier: IdentifierFiroe, depth: Int): String =
    // If the identifier has been resolved and is not NYE
    if !identifier.isNye then
      if identifier.atConstanic then
         indent(depth) + addNyesStateIfEnabled(CC_STR, identifier.getNyes)
      else if identifier.isAbstract then
        indent(depth) + NK_STR
      else
        unwrap(identifier) match
          case constanic if constanic.atConstanic => indent(depth) + addNyesStateIfEnabled(CC_STR, constanic.getNyes)
          case brane: BraneFiroe =>
             // Should not typically happen for top-level sequencing but good to handle
             sequence(brane, depth)
          case unwrapped =>
             indent(depth) + unwrapped.getValue.toString
    else if identifier.atConstanic then
        indent(depth) + addNyesStateIfEnabled(CC_STR, identifier.getNyes)
    else
      // If not yet evaluated
      indent(depth) + NK_STR

  protected def sequenceOneShotSearch(oneShotSearch: OneShotSearchFiroe, depth: Int): String =
    if !oneShotSearch.isNye then
      // Check if search found nothing - not found is CONSTANIC
      if !oneShotSearch.isFound then
        indent(depth) + addNyesStateIfEnabled(CC_STR, oneShotSearch.getNyes)
      else if oneShotSearch.atConstant then
        // Found and CONSTANT - use sequence() recursively on the result
        sequence(oneShotSearch.getResult, depth)
      else
        // Search found something but it's CONSTANIC (unresolved)
        indent(depth) + addNyesStateIfEnabled(CC_STR, oneShotSearch.getNyes)
    else
      indent(depth) + NK_STR

  @scala.annotation.tailrec
  private def unwrap(fir: FIR): FIR = fir match
    case assignment: AssignmentFiroe =>
       // Always try to unwrap to get the actual result
       // Don't stop at constanic - we want to see if result is a brane/concatenation
       val result = assignment.getResult
       if result != null then unwrap(result)
       else assignment
    case identifier: IdentifierFiroe =>
       if identifier.atConstanic then identifier
       else if identifier.value != null then unwrap(identifier.value)
       else identifier
    case abstractSearch: AbstractSearchFiroe if abstractSearch.getResult != null =>
       unwrap(abstractSearch.getResult)
    case unanchoredSeek: UnanchoredSeekFiroe if unanchoredSeek.getResult != null =>
       unwrap(unanchoredSeek.getResult)
    case brane: BraneFiroe => brane  // Stop at branes
    case concat: ConcatenationFiroe => concat  // Stop at concatenations
    case other => other

  protected def sequenceConcatenation(concat: ConcatenationFiroe, depth: Int): String =
    // Rendering depends on Nyes state:
    // - If at least PRIMED: render as flat brane - flatten sub-brane contents into single brane
    // - If not PRIMED: render as {..} {..} with single-space separators showing proximity
    if concat.getNyes.ordinal >= Nyes.PRIMED.ordinal then
      // At least PRIMED - render as flat brane with flattened contents
      val sb = StringBuilder()
      sb.append(indent(depth)).append("{\n")

      // Render each item in the concatenation's flattened memory
      // After performJoin, the concatenation contains individual statements (not nested branes)
      concat.stream.foreach { fir =>
        sb.append(sequence(fir, depth + 1))
        sb.append(";\n")
      }

      sb.append(indent(depth)).append("}")
      sb.toString()
    else
      // Not yet PRIMED - render elements with single-space separators
      // This shows the proximity of concatenated elements
      val sb = StringBuilder()
      sb.append(indent(depth))

      var first = true
      concat.stream.foreach { fir =>
        if !first then
          sb.append(" ")  // Single space separator for proximity
        first = false

        // Render each element inline (compact format)
        val element = sequenceInline(fir)
        sb.append(element)
      }

      sb.toString()

  /**
   * Sequences a FIR for inline display (used in concatenation proximity rendering).
   * Branes are rendered as {...} without internal expansion.
   */
  private def sequenceInline(fir: FIR): String = fir match
    case brane: BraneFiroe => "{...}"
    case concat: ConcatenationFiroe => "{...}"
    case value: ValueFiroe => value.getValue.toString
    case identifier: IdentifierFiroe =>
      if identifier.isConstant then
        try
          identifier.getValue.toString
        catch
          case _: UnsupportedOperationException =>
            // For IdentifierFiroe, value field holds resolved FIR
            if identifier.value != null then
              if identifier.value.isInstanceOf[BraneFiroe] || identifier.value.isInstanceOf[ConcatenationFiroe] then
                "{...}"
              else
                CC_STR
            else
              CC_STR
      else if identifier.atConstanic then
        addNyesStateIfEnabled(CC_STR, identifier.getNyes)
      else
        NK_STR
    case _ => NK_STR

  protected def sequenceNK(nk: FIR, depth: Int): String =
    indent(depth) + NK_STR

  private def indent(depth: Int): String = tabChar * depth

  def getTabChar: String = tabChar
