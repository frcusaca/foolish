package org.foolish.fvm.scubc

import org.foolish.fvm.BraneMemoryInterface
import scala.collection.mutable
import java.util.regex.Pattern

/**
 * BraneMemory implementation for the UBC system.
 */
class BraneMemory(val parent: BraneMemory = null) extends BraneMemoryInterface[BraneMemory.Query, FIR]:

  private val memory = mutable.Map[Int, mutable.Map[CharacterizedIdentifier, FIR]]()

  def get(id: BraneMemory.Query, fromLine: Int): FIR =
    (fromLine to 0 by -1).view.flatMap { line =>
      memory.get(line).flatMap { lineMemory =>
        val charId = id.asInstanceOf[CharacterizedIdentifier]
        if lineMemory.contains(charId) then Some(lineMemory(charId))
        else None
      }
    }.headOption.orNull

  def put(id: BraneMemory.Query, value: FIR, byLine: Int): Unit =
    id match
      case charId: CharacterizedIdentifier =>
        memory.getOrElseUpdate(byLine, mutable.Map()).put(charId, value)
      case _ =>
        throw IllegalArgumentException("Only IdentifierQuery is supported for put operations")

object BraneMemory:
  trait Query

  /** Regular class to avoid case-to-case inheritance prohibition */
  class IdentifierQuery(name: String, characterization: String = "")
    extends CharacterizedIdentifier(name, characterization) with Query

  case class RegExpQuery(regex: String) extends Query:
    val pattern: Pattern = Pattern.compile(regex)
