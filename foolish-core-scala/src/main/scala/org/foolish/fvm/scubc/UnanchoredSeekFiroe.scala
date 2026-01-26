package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * UnanchoredSeekFiroe implements unanchored backward seek in the current brane.
 * Syntax: #-1 (previous statement), #-2 (two statements back), etc.
 *
 * Unanchored seek searches backwards from the current position in the brane.
 * Only negative offsets are allowed (excluding 0).
 * The search is bound by the current brane and becomes CONSTANIC if the offset goes beyond the brane start.
 *
 * Examples:
 *   {a=1; b=2; c=#-1 + #-2}  // c = 2 + 1 = 3
 *   {x=10; y=#-5}            // y is CONSTANIC (out of bounds, rendered as ⎵⎵)
 *
 * IMPORTANT: Unanchored seeks with out-of-bounds offsets become CONSTANIC, not NK (???).
 * This is intentional - the seek awaits potential brane concatenation that could provide
 * the missing statements. When branes are concatenated, a previously out-of-bounds seek
 * may become resolved.
 *
 * Future Feature - Brane Concatenation:
 *   When branes are concatenated (e.g., {a=1}{b=#-1}), the unanchored seek #-1 in the second
 *   brane should find 'a' from the first brane. Currently, brane concatenation is not fully
 *   implemented, so out-of-bounds seeks remain CONSTANIC.
 *
 * TODO: When implementing brane concatenation, ensure:
 *   1. Unanchored seeks re-evaluate after concatenation
 *   2. CONSTANIC seeks transition to CHECKED when they find their target
 *   3. The search uses the concatenated brane's full memory, not just the original brane
 */
class UnanchoredSeekFiroe(seekExpr: AST.UnanchoredSeekExpr) extends FiroeWithBraneMind(seekExpr):

  private val offset: Int = seekExpr.offset()
  private var value: FIR = null

  // Validate: only negative offsets allowed
  if offset >= 0 then
    throw IllegalArgumentException(s"Unanchored seek only allows negative offsets: #$offset")

  override protected def initialize(): Unit =
    setInitialized()

  /**
   * An unanchored seek is abstract if it hasn't been resolved yet or if its resolved value is abstract.
   */
  override def isAbstract: Boolean =
    if atConstanic then
      true
    else if value == null then
      true // Not yet resolved or out of bounds
    else
      value.isAbstract

  override def isNye: Boolean =
    getNyes.ordinal < Nyes.CONSTANIC.ordinal

  /**
   * Resolve the unanchored seek during the INITIALIZED phase.
   *
   * Unanchored seeks search in the containing brane's memory, not in the current
   * expression's memory. Since this FIR is typically ordinated to an AssignmentFiroe
   * (which has empty memory), we need to access the parent's memory (the actual brane).
   */
  override def step(): Unit =
    getNyes match
      case Nyes.INITIALIZED =>
        // Find the actual brane's memory by traversing up the parent chain
        // We need to go all the way to the top-level brane (BraneFiroe) which contains
        // the actual statements. Intermediate FIRs (BinaryFiroe, AssignmentFiroe) have
        // empty memories.
        var targetMemory = braneMemory
        var candidateMemory = targetMemory.getParent
        var stepsUp = 0

        // Traverse all the way up the parent chain
        while candidateMemory != null do
          targetMemory = candidateMemory
          candidateMemory = candidateMemory.getParent
          stepsUp += 1

        // Target memory should now be the BraneFiroe's memory
        val size = targetMemory.size

        if size == 0 then
          // Empty brane - out of bounds
          value = null
          setNyes(Nyes.CONSTANIC)
        else
          // Now find the current position by checking myPos at each level going back up
          // The last non-(-1) myPos we find tells us which statement we're in
          var temp = braneMemory
          var currentPos = -1
          while temp != null do
            val pos = temp.getMyPos
            if pos >= 0 then
              currentPos = pos
            temp = temp.getParent

          if currentPos < 0 then
            currentPos = size - 1  // Default to last position if not set

          // Calculate target index: currentPos + offset (offset is negative)
          // Example: currentPos=2, offset=-1 -> targetIdx=1 (previous statement)
          val targetIdx = currentPos + offset

          // Check bounds
          if targetIdx >= 0 && targetIdx < size then
            value = targetMemory.get(targetIdx)
            setNyes(Nyes.CHECKED)
          else
            // Out of bounds - return constanic (not found)
            value = null
            setNyes(Nyes.CONSTANIC)

      case _ =>
        super.step()

  override def getValue: Long =
    if value == null then
      if atConstanic then
        throw IllegalStateException(s"Unanchored seek is Constanic (out of bounds): #$offset")
      throw IllegalStateException(s"Unanchored seek not resolved: #$offset")
    value.getValue

  override def toString: String = ast.toString
