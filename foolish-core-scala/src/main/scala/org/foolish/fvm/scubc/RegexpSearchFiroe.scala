package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * RegexpSearchFiroe performs a regular expression search on a brane.
 * It evaluates an anchor expression (expected to be a brane), then searches
 * within that brane's memory for an identifier matching the regexp pattern.
 *
 * Syntax: brane ? pattern or brane ?? pattern or brane ?* pattern
 *
 * The ? operator performs localized search (only within the brane, no parent search).
 * The ?? operator performs globalized search (cursor-based search upward through parents).
 * The ?* operator performs multi-search (returns a brane with all matching results).
 */
class RegexpSearchFiroe(regexpSearch: AST.RegexpSearchExpr)
  extends FiroeWithBraneMind(regexpSearch):

  import RegexpSearchFiroe.SearchResultType

  private val operator: String = regexpSearch.operator()
  private val pattern: String = regexpSearch.pattern()
  private val resultType: SearchResultType = SearchResultType.VALUE
  private[scubc] var searchResult: FIR = null // Package-private for chained searches

  override protected def initialize(): Unit =
    setInitialized()
    // Enqueue the anchor expression to be evaluated first
    val anchorExpr = regexpSearch.anchor()
    anchorExpr match
      case _: AST.Brane | _: AST.Branes | _: AST.RegexpSearchExpr | _: AST.Identifier =>
        enqueueExprs(anchorExpr)
      case _ =>
        throw IllegalArgumentException("regular expression search must anchor onto a brane context.")

  override def step(): Unit =
    getNyes match
      case Nyes.INITIALIZED =>
        // Wait for anchor expression to be evaluated
        if stepNonBranesUntilState(Nyes.REFERENCES_IDENTIFIED) then
          setNyes(Nyes.REFERENCES_IDENTIFIED)

      case Nyes.REFERENCES_IDENTIFIED =>
        if stepNonBranesUntilState(Nyes.ALLOCATED) then
          setNyes(Nyes.ALLOCATED)

      case Nyes.ALLOCATED =>
        if stepNonBranesUntilState(Nyes.RESOLVED) then
          setNyes(Nyes.RESOLVED)

      case Nyes.RESOLVED =>
        // Wait for anchor brane to be fully CONSTANT before searching
        if stepNonBranesUntilState(Nyes.CONSTANT) then
          // Check if the anchor is ready (unwrapping identifiers and assignments)
          if isAnchorReady() then
            // Perform the search
            performSearch()
            setNyes(Nyes.CONSTANT)

      case _ =>
        super.step()

  private def stepNonBranesUntilState(targetState: Nyes): Boolean =
    if braneMind.isEmpty then
      return true

    val current = braneMind.dequeue()
    current.step()

    if current.isNye then
      braneMind.enqueue(current)

    current.getNyes.ordinal >= targetState.ordinal

  private def isAnchorReady(): Boolean =
    if braneMemory.isEmpty then
      return false

    // Unwrap identifier to get the actual value
    val afterIdentifierUnwrap = braneMemory.getLast match
      case identifierFiroe: IdentifierFiroe =>
        if identifierFiroe.value == null then
          return false // Identifier not yet resolved
        identifierFiroe.value
      case other => other

    // Unwrap assignment to get the assigned value
    val afterAssignmentUnwrap = afterIdentifierUnwrap match
      case assignmentFiroe: AssignmentFiroe =>
        // Check if the assignment has been fully evaluated
        if assignmentFiroe.isNye then
          // Assignment not yet complete, step it
          assignmentFiroe.step()
          return false
        if assignmentFiroe.getResult.isEmpty then
          return false // Assignment not yet evaluated
        assignmentFiroe.getResult.get
      case other => other

    // Check if it's a chained RegexpSearchFiroe
    afterAssignmentUnwrap match
      case regexpSearchFiroe: RegexpSearchFiroe =>
        // Wait for the chained search to complete
        if regexpSearchFiroe.isNye then
          regexpSearchFiroe.step()
          false
        else
          true
      case _ => true // Anchor is ready

  private def performSearch(): Unit =
    // Check which result type is requested
    resultType match
      case SearchResultType.VALUE =>
        // Implemented below - return the actual value
      case SearchResultType.NAME =>
        throw UnsupportedOperationException("Search result type NAME not yet implemented")
      case SearchResultType.CONTEXT =>
        throw UnsupportedOperationException("Search result type CONTEXT not yet implemented")
      case SearchResultType.ASSIGNMENT =>
        throw UnsupportedOperationException("Search result type ASSIGNMENT not yet implemented")

    // Get the anchor brane from braneMemory (the evaluated anchor expression)
    if braneMemory.isEmpty then
      searchResult = NKFiroe()
      return

    // Unwrap layers until we reach a searchable value (brane or ???)
    //
    // NOTE: Current FIR architecture has IdentifierFiroe.value point to AssignmentFiroe,
    // which then contains the actual value. This means we must unwrap both:
    //   brn (IdentifierFiroe) -> brn=... (AssignmentFiroe) -> {...} (BraneFiroe)
    //
    // Question for consideration: Should identifiers point directly to values for search purposes?
    // Assignments are only meaningful as brane members, not as intermediate search steps.
    var anchor = braneMemory.getLast
    var continue = true
    var searchPerformed = false // Track whether we've already searched

    while continue do
      anchor match
        case identifierFiroe: IdentifierFiroe =>
          anchor = identifierFiroe.value
          if anchor == null then
            searchResult = NKFiroe()
            return

        case assignmentFiroe: AssignmentFiroe =>
          // Unwrap assignment - this is needed because identifiers point to assignments
          // Step the assignment until it's fully evaluated
          while assignmentFiroe.isNye do
            assignmentFiroe.step()

          if assignmentFiroe.getResult.isEmpty then
            searchResult = NKFiroe()
            return
          anchor = assignmentFiroe.getResult.get

        case regexpSearchFiroe: RegexpSearchFiroe =>
          anchor = regexpSearchFiroe.searchResult
          if anchor == null then
            searchResult = NKFiroe()
            return

        case braneFiroe: BraneFiroe =>
          // If we've already performed a search, this brane is the result - return it
          if searchPerformed then
            searchResult = braneFiroe
            return

          // Found a brane - perform the search
          val query = BraneMemory.RegexpQuery(pattern)
          val targetMemory = braneFiroe.braneMemory
          val searchFrom = targetMemory.size - 1

          val result: Option[(Int, FIR)] = if operator == "?" then
            targetMemory.getLocal(query, searchFrom)
          else
            targetMemory.get(query, searchFrom)

          searchPerformed = true // Mark that we've performed the search

          var foundValue = result match
            case Some((_, fir)) => fir
            case None => NKFiroe()

          // For VALUE result type: Unwrap assignments to get the actual value
          // When we find "name=value" in a brane, we want to return the value, not the assignment
          foundValue = foundValue match
            case assignmentFiroe: AssignmentFiroe =>
              val res = assignmentFiroe.getResult
              if res.isEmpty then NKFiroe() else res.get
            case other => other

          // If the result is still a wrapper type (Identifier/Assignment/RegexpSearch),
          // put it back through the unwrapping loop to fully resolve it to a concrete value
          foundValue match
            case _: IdentifierFiroe | _: AssignmentFiroe | _: RegexpSearchFiroe =>
              anchor = foundValue
              // Continue unwrapping
            case _ =>
              // Fully unwrapped - return the concrete value
              searchResult = foundValue
              return

        case _: NKFiroe =>
          // Searching ??? returns ???
          searchResult = NKFiroe()
          return

        case _ =>
          // Can only search branes or ???
          searchResult = NKFiroe()
          return

  override def isAbstract: Boolean =
    if searchResult == null then
      true
    else
      searchResult.isAbstract

  override def getValue: Long =
    if searchResult == null then
      throw IllegalStateException("RegexpSearch not yet evaluated")
    // searchResult is the FIR that was found, which should be fully evaluated by now
    searchResult.getValue

  override def toString: String =
    regexpSearch.toString

object RegexpSearchFiroe:
  /**
   * Specifies what kind of result a search should return.
   */
  enum SearchResultType:
    /** Return the identifier name that matched (e.g., "alice") */
    case NAME
    /** Return the value bound to the identifier (e.g., the brane or number) */
    case VALUE
    /** Return contextual information about the match (line number, characterizations, etc.) */
    case CONTEXT
    /** Return the full assignment (identifier + value pair) */
    case ASSIGNMENT
