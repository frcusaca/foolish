package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * RegexpSearchFiroe performs a regular expression search on a brane.
 * It evaluates a base expression (expected to be a brane), then searches
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

  private val operator: String = regexpSearch.operator()
  private val pattern: String = regexpSearch.pattern()
  private var searchResult: FIR = null

  override protected def initialize(): Unit =
    setInitialized()
    // Enqueue the base expression to be evaluated first
    enqueueExprs(regexpSearch.base())

  override def step(): Unit =
    getNyes match
      case Nyes.INITIALIZED =>
        // Wait for base expression to be evaluated
        if stepNonBranesUntilState(Nyes.REFERENCES_IDENTIFIED) then
          setNyes(Nyes.REFERENCES_IDENTIFIED)

      case Nyes.REFERENCES_IDENTIFIED =>
        if stepNonBranesUntilState(Nyes.ALLOCATED) then
          setNyes(Nyes.ALLOCATED)

      case Nyes.ALLOCATED =>
        if stepNonBranesUntilState(Nyes.RESOLVED) then
          setNyes(Nyes.RESOLVED)

      case Nyes.RESOLVED =>
        // Wait for base brane to be fully CONSTANT before searching
        if stepNonBranesUntilState(Nyes.CONSTANT) then
          // Perform the search
          performSearch()
          setNyes(Nyes.CONSTANT)

      case _ =>
        super.step()

  private def stepNonBranesUntilState(targetState: Nyes): Boolean =
    if braneMind.isEmpty then
      return true

    val current = braneMind.removeFirst()
    current.step()

    if current.isNye then
      braneMind.addLast(current)

    current.getNyes.ordinal() >= targetState.ordinal()

  private def performSearch(): Unit =
    // Get the base brane from braneMemory (the evaluated base expression)
    if braneMemory.isEmpty then
      searchResult = NKFiroe()
      return

    val base = braneMemory.getLast

    // Base must be a brane
    base match
      case braneFiroe: BraneFiroe =>
        // Create a regexp query
        val query = BraneMemory.RegexpQuery(pattern)

        // Search from the last line of the brane backward
        val targetMemory = braneFiroe.braneMemory
        val searchFrom = targetMemory.size - 1

        // Use getLocal() for the ? operator (localized search, no parent search)
        // Use get() for ?? operator (globalized search with parent search)
        val result = if operator == "?" then
          targetMemory.getLocal(query, searchFrom)
        else
          targetMemory.get(query, searchFrom)

        result match
          case Some((_, fir)) => searchResult = fir
          case None => searchResult = NKFiroe()

      case _ =>
        // Can only search branes
        searchResult = NKFiroe()

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
