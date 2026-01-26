package org.foolish.fvm.scubc

import scala.collection.mutable

/**
 * BraneMemory implementation for the UBC system.
 * Stores FIRs in a list indexed by line number, matching the Java implementation.
 */
class BraneMemory(private var parent: BraneMemory = null):
  private var myPos: Option[Int] = None
  private val memory = mutable.ArrayBuffer[FIR]()
  private var owningBrane: BraneFiroe = null  // The BraneFiroe that owns this memory

  def this(parent: BraneMemory, myPos: Int) =
    this(parent)
    this.myPos = Some(myPos)

  /**
   * Internal method for setting position in parent memory.
   * Only used internally - external code should use FIR's getMyBraneIndex() instead.
   */
  def setMyPosInternal(pos: Int): Unit =
    if myPos.isEmpty then
      myPos = Some(pos)
    else
      throw RuntimeException("Cannot recoordinate a BraneMemory.")

  def setParent(parent: BraneMemory): Unit =
    this.parent = parent

  def getParent: BraneMemory = parent

  def getMyPos: Int = myPos.getOrElse(-1)

  /**
   * Sets the BraneFiroe that owns this BraneMemory.
   * Should only be called once, typically by BraneFiroe during construction.
   */
  def setOwningBrane(brane: BraneFiroe): Unit =
    if owningBrane == null then
      owningBrane = brane
    else
      throw RuntimeException("Cannot reassign owning brane of BraneMemory.")

  /**
   * Gets the BraneFiroe that owns this BraneMemory.
   * Returns null if this is not a brane's memory (e.g., expression evaluation memory).
   */
  def getOwningBrane: BraneFiroe = owningBrane

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
      // Compute position dynamically using owningBrane's getMyBraneIndex() if available
      // Otherwise fall back to myPos (for unit tests without BraneFiroe)
      // Default to searching from end of parent if neither is available
      // This is crucial for CMFir which links memory without fixed position
      val parentPos = if owningBrane != null then
        owningBrane.getMyBraneIndex
      else
        myPos.getOrElse(parent.size - 1)
      return parent.get(query, parentPos)

    None // Not found

  /**
   * Search for a query locally within this brane only, without searching parent branes.
   * Searches backward from fromLine to 0 (finds last match).
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

  /**
   * Search for a query locally within this brane only, without searching parent branes.
   * Searches forward from fromLine to end (finds first match).
   * Used for localized forward regex search (~ operator).
   */
  def getLocalForward(query: BraneMemory.Query, fromLine: Int): Option[(Int, FIR)] =
    val startLine = math.max(fromLine, 0)

    // Search forwards from fromLine to end
    for line <- startLine until memory.size do
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
  /**
   * Query represents a search pattern for finding lines in BraneMemory.
   * This is a sealed trait with different query types:
   * - StrictlyMatchingQuery: exact identifier match
   * - RegexpQuery: regular expression pattern match
   */
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

  /**
   * RegexpQuery matches identifiers using a regular expression pattern.
   * If the pattern doesn't start with ^ or end with $, they are added
   * to make it a "whole identifier" match (similar to StrictlyMatchingQuery).
   * If the pattern contains anchors, it uses partial matching within the identifier.
   */
  class RegexpQuery(regexPattern: String) extends Query:
    val originalPattern: String = regexPattern
    val startsWithCaret: Boolean = regexPattern.startsWith("^")
    val endsWithDollar: Boolean = regexPattern.endsWith("$")
    val isAnchored: Boolean = startsWithCaret || endsWithDollar

    // If no anchoring, add both ^ and $ to make it a whole match
    val finalPattern: String =
      if !startsWithCaret && !endsWithDollar then
        "^" + regexPattern + "$"
      else
        regexPattern

    val pattern: scala.util.matching.Regex = finalPattern.r

    override def matches(brane_line: FIR): Boolean =
      brane_line match
        case ass: AssignmentFiroe =>
          val lhs = ass.getLhs
          val fullName = lhs.toString  // Includes characterization
          val nameOnly = lhs.getId

          // Try matching against both the full name and name only
          pattern.findFirstIn(fullName).isDefined || pattern.findFirstIn(nameOnly).isDefined
        case _ => false
