package org.foolish.fvm.scubc

import scala.collection.mutable

/**
 * BraneMemory implementation for the UBC system.
 * Stores FIRs in a list indexed by line number, matching the Java implementation.
 */
class BraneMemory(private var parent: BraneMemory = null):
  private var myPos: Option[Int] = None
  private val memory = mutable.ArrayBuffer[FIR]()

  def this(parent: BraneMemory, myPos: Int) =
    this(parent)
    setMyPos(myPos)

  def setMyPos(pos: Int): Unit =
    if myPos.isEmpty then
      myPos = Some(pos)
    else
      throw RuntimeException("Cannot recoordinate a BraneMemory.")

  def setParent(parent: BraneMemory): Unit =
    this.parent = parent

  def get(idx: Int): FIR =
    if idx >= 0 && idx < memory.size then
      memory(idx)
    else
      throw IndexOutOfBoundsException(s"Index: $idx, Size: ${memory.size}")

  /**
   * Get the FIR matching the query, searching backwards from fromLine.
   * Returns Option[(Int, FIR)] where Int is the line number.
   */
  def get(query: BraneMemory.Query, fromLine: Int): Option[(Int, FIR)] =
    val startLine = math.min(fromLine, memory.size - 1)

    // Search backwards from fromLine to 0
    for line <- startLine to 0 by -1 do
      val lineMemory = memory(line)
      if query.matches(lineMemory) then
        return Some((line, lineMemory))

    // If not found in this brane, search in parent
    if parent != null then
      return parent.get(query, myPos.get)

    None // Not found

  /**
   * Search for a query locally within this brane only, without searching parent branes.
   * Used for localized regex search (? operator).
   */
  def getLocal(query: BraneMemory.Query, fromLine: Int): Option[(Int, FIR)] =
    val startLine = math.min(fromLine, memory.size - 1)

    // Search backwards from fromLine to 0
    for line <- startLine to 0 by -1 do
      val lineMemory = memory(line)
      if query.matches(lineMemory) then
        return Some((line, lineMemory))

    None // Not found, don't search parents

  def put(line: FIR): Unit =
    memory.addOne(line)

  def isEmpty: Boolean = memory.isEmpty

  def stream: Iterator[FIR] = memory.iterator

  def size: Int = memory.size

  def getLast: FIR =
    if memory.isEmpty then
      throw new NoSuchElementException("BraneMemory is empty")
    memory.last

  def removeFirst(): FIR =
    if memory.isEmpty then
      throw new NoSuchElementException("BraneMemory is empty")
    memory.remove(0)

object BraneMemory:
  sealed trait Query:
    def matches(brane_line: FIR): Boolean

  class StrictlyMatchingQuery(name: String, characterization: String)
    extends CharacterizedIdentifier(name, characterization) with Query:

    override def matches(brane_line: FIR): Boolean =
      brane_line match
        case ass: AssignmentFiroe =>
          val lhs = ass.getLhs
          lhs == this
        case _ =>
          // mostly here is unnamed lines in brane
          false
