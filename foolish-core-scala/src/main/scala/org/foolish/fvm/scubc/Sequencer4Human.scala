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
    case identifier: IdentifierFiroe => sequenceIdentifier(identifier, depth)
    case oneShotSearch: OneShotSearchFiroe => sequenceOneShotSearch(oneShotSearch, depth)
    case _ => indent(depth) + "???"

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
    assignment.getFiroeState match
      case FiroeState.Constantic() =>
        indent(depth) + s"${assignment.getId} = ?C?"
      case FiroeState.Value(result) if !assignment.isNye =>
        // Check if the result is fully evaluated
        if !result.isNye then
            unwrap(result) match
              case FiroeState.Constantic() =>
                 indent(depth) + s"${assignment.getId} = ?C?"
              case FiroeState.Value(unwrapped) =>
                if unwrapped.isAbstract then
                   indent(depth) + s"${assignment.getId} = ???"
                else
                   unwrapped match
                      case brane: BraneFiroe =>
                         // Special handling for nested branes to align indentation
                         val sequencedBrane = sequence(brane, depth)
                         // Strip the first line's indentation
                         val indentStr = indent(depth)
                         val strippedBrane = if (sequencedBrane.startsWith(indentStr)) then
                           sequencedBrane.substring(indentStr.length)
                         else
                           sequencedBrane

                         // Calculate padding for subsequent lines
                         val padding = " " * (assignment.getId.length + 3)
                         // Apply padding to subsequent lines
                         val alignedBrane = strippedBrane.replace("\n", "\n" + padding)

                         indent(depth) + s"${assignment.getId} = ${alignedBrane}"

                      case simple =>
                         // Simple values
                         indent(depth) + s"${assignment.getId} = ${simple.getValue}"
              case _ => indent(depth) + s"${assignment.getId} = ???"
        else
          indent(depth) + s"${assignment.getId} = ???"
      case _ =>
        // If not yet evaluated, show the structure
        indent(depth) + s"${assignment.getId} = ???"

  protected def sequenceIdentifier(identifier: IdentifierFiroe, depth: Int): String =
    identifier.state match
      case FiroeState.Constantic() =>
        indent(depth) + "?C?"
      case FiroeState.Value(fir) if !identifier.isNye =>
          if identifier.isAbstract then
             indent(depth) + "???"
          else
             unwrap(identifier) match
                case FiroeState.Constantic() => indent(depth) + "?C?"
                case FiroeState.Value(unwrapped) =>
                   unwrapped match
                      case brane: BraneFiroe => sequence(brane, depth)
                      case simple => indent(depth) + simple.getValue.toString
                case _ => indent(depth) + "???"
      case _ => indent(depth) + "???"

  protected def sequenceOneShotSearch(oneShotSearch: OneShotSearchFiroe, depth: Int): String =
    if !oneShotSearch.isNye then
      if oneShotSearch.isAbstract then
        indent(depth) + "???"
      else
        // Use sequence() recursively on the result if we can access it
        sequence(oneShotSearch.getResult, depth)
    else
      indent(depth) + "???"

  @scala.annotation.tailrec
  private def unwrap(fir: FIR): FiroeState = fir match
    case assignment: AssignmentFiroe =>
       assignment.getFiroeState match
          case FiroeState.Value(v) => unwrap(v)
          case other => other
    case identifier: IdentifierFiroe =>
       identifier.state match
          case FiroeState.Value(v) => unwrap(v)
          case other => other
    case oneShotSearch: OneShotSearchFiroe if oneShotSearch.getResult != null =>
       unwrap(oneShotSearch.getResult)
    case other => FiroeState.Value(other)

  protected def sequenceNK(nk: FIR, depth: Int): String =
    indent(depth) + "???"

  private def indent(depth: Int): String = tabChar * depth

  def getTabChar: String = tabChar
