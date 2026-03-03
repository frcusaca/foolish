package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}

/**
 * AbstractSearchFiroe implements the common logic for search operations.
 */
abstract class AbstractSearchFiroe(ast: AST.Expr, val operator: SearchOperator) extends FiroeWithBraneMind(ast) {
  protected var searchResult: FIR = _
  protected var unwrapAnchor: FIR = _
  protected var searchPerformed: Boolean = false
  protected var found: Boolean = false

  override protected def initialize(): Unit = {
    setInitialized()
    // Subclasses must enqueue anchor
  }

  override def step(): Int = {
    getNyes match {
      case Nyes.INITIALIZED =>
        if (stepNonBranesUntilState(Nyes.CHECKED)) {
          setNyes(Nyes.CHECKED)
        }
        1
      case Nyes.CHECKED =>
        if (stepNonBranesUntilState(Nyes.CONSTANIC)) {
          if (isAnchorReady) {
            performSearchStep()
            if (atConstanic) {
              return 1
            }
            if (searchResult != null) {
              // Set found status based on result type
              searchResult match {
                case _: NKFiroe =>
                  // Search failed - result is NK (not found)
                  found = false
                  setNyes(Nyes.CONSTANIC)
                case _ =>
                  // Search succeeded - check if result is Constanic
                  found = true
                  if (searchResult.isNye) {
                    searchResult.step()
                  } else if (searchResult.atConstanic) {
                    // Result is Constanic - propagate it
                    setNyes(Nyes.CONSTANIC)
                  } else {
                    // Result is CONSTANT - we're done
                    setNyes(Nyes.CONSTANT)
                  }
              }
            } else {
              // searchResult is null means "found but constanic" (e.g., constanic identifier with no value)
              // This matches Java's valuableSelf() returning Optional.empty()
              found = true
              setNyes(Nyes.CONSTANIC)
            }
          }
        }
        1
      case _ =>
        super.step()
    }
  }

  protected def stepNonBranesUntilState(targetState: Nyes): Boolean = {
    if (braneMind.isEmpty) return true

    val current = braneMind.dequeue()
    current.step()

    if (current.isNye) {
      braneMind.enqueue(current)
    }

    current.getNyes.ordinal >= targetState.ordinal
  }

  protected def isAnchorReady: Boolean = {
    if (braneMemory.isEmpty) return false

    var anchor = braneMemory.getLast

    // Check if anchor is CONSTANIC
    if (anchor.atConstanic) {
      return true
    }

    // Unwrap identifier
    val resolvedAnchor = anchor match {
      case identifierFiroe: IdentifierFiroe =>
        if (identifierFiroe.atConstanic) return true
        if (identifierFiroe.value == null) return false
        identifierFiroe.value
      case _ => anchor
    }
    if (resolvedAnchor == null) return false
    anchor = resolvedAnchor

    // Unwrap assignment
    val resolvedAnchor2 = anchor match {
      case assignmentFiroe: AssignmentFiroe =>
        if (assignmentFiroe.atConstanic) return true
        if (assignmentFiroe.isNye) {
          assignmentFiroe.step()
          return false
        }
        val res = assignmentFiroe.getResult
        if (res == null) return false
        res
      case _ => anchor
    }
    if (resolvedAnchor2 == null) return false
    anchor = resolvedAnchor2

    // Check chained search
    anchor match {
      case abstractSearch: AbstractSearchFiroe =>
        if (abstractSearch.atConstanic) {
          return true
        }
        if (abstractSearch.isNye) {
          abstractSearch.step()
          false
        } else {
          true
        }
      case unanchoredSeek: UnanchoredSeekFiroe =>
        val seekValue = unanchoredSeek.getResult
        if (unanchoredSeek.atConstanic) {
          return true
        }
        if (unanchoredSeek.isNye) {
          unanchoredSeek.step()
          false
        } else {
          true
        }
      case _ =>
        true
    }
    end match
  }

  protected def performSearchStep(): Unit = {
    if (unwrapAnchor == null && !searchPerformed) {
      if (braneMemory.isEmpty) {
        searchResult = new NKFiroe()
        return
      }
      val last = braneMemory.getLast
      unwrapAnchor = last
    }

    if (searchResult != null) return

    // Loop to fully resolve the anchor (unwrap through identifiers, assignments, searches)
    // This must happen BEFORE checking for constanic because a constanic identifier
    // might resolve to a brane that we can search
    var resolved = false
    while (!resolved) {
      resolved = true  // Assume resolved unless we find something to unwrap

      if (searchResult != null) return  // Check after unwrapping might have set result

      unwrapAnchor match {
      case identifierFiroe: IdentifierFiroe =>
        // Handle constanic identifiers first - they're already resolved
        if (identifierFiroe.atConstanic) {
          if identifierFiroe.value != null then
            // Identifier is constanic with a value - the value is the resolved result
            // (e.g., AA -> AssignmentFiroe that's constanic because its RHS is constanic)
            searchResult = identifierFiroe.value
          else
            // Identifier is constanic with no value - not found
            searchResult = null
          resolved = true
        }
        else if (identifierFiroe.isNye) {
          // Not yet resolved, step and wait
          identifierFiroe.step()
          if identifierFiroe.isNye then
            return  // Will retry on next step
          // After stepping, check if resolved
          unwrapAnchor = identifierFiroe.value
          if unwrapAnchor == null then
            searchResult = new NKFiroe()
          resolved = false  // Continue unwrapping if we got a value
        }
        else {
          // Identifier is resolved (not constanic, not NYE)
          unwrapAnchor = identifierFiroe.value
          if unwrapAnchor == null then
            searchResult = new NKFiroe()
          resolved = false  // Continue unwrapping if we got a value
        }

      case assignmentFiroe: AssignmentFiroe =>
        // Handle constanic assignments first - they're already resolved
        if assignmentFiroe.atConstanic then
          val res = assignmentFiroe.getResult
          if res == null then
            // Assignment is constanic but has no result yet
            searchResult = null
            resolved = true
          else if res.isInstanceOf[BraneFiroe] then
            // Assignment's result is a brane - continue unwrapping to let the BraneFiroe case handle it
            // This is critical for OneShotSearchFiroe (e.g., $ #-1) to properly extract tail/head from branes
            unwrapAnchor = res
            resolved = false
          else
            // Assignment's result is the resolved value (not a brane)
            searchResult = res
            resolved = true
        else if assignmentFiroe.isNye then
          // Not yet evaluated, step and wait
          assignmentFiroe.step()
          if assignmentFiroe.isNye then
            return  // Will retry on next step
          // After stepping, get result
          val res = assignmentFiroe.getResult
          if res == null then
            searchResult = new NKFiroe()
          else
            unwrapAnchor = res
          resolved = false
        else
          // Assignment is fully evaluated
          val res = assignmentFiroe.getResult
          if res == null then
            searchResult = new NKFiroe()
          else
            unwrapAnchor = res
          resolved = false

      case abstractSearch: AbstractSearchFiroe =>
        if (abstractSearch.isNye) {
          abstractSearch.step()
          if (abstractSearch.isNye) {
            return  // Will retry on next step
          }
        }
        val result = abstractSearch.getResult
        if (result == null && !abstractSearch.atConstanic) {
            searchResult = new NKFiroe()
        }
        else if (abstractSearch.atConstanic) {
            // For constanic searches, return the search itself as the result
            searchResult = abstractSearch
        }
        else {
            unwrapAnchor = result
        }
        resolved = false  // Continue unwrapping

      case unanchoredSeekFiroe: UnanchoredSeekFiroe =>
        if (unanchoredSeekFiroe.isNye) {
          unanchoredSeekFiroe.step()
          if (unanchoredSeekFiroe.isNye) {
            return  // Will retry on next step
          }
        }
        val result = unanchoredSeekFiroe.getResult
        if (result == null && !unanchoredSeekFiroe.atConstanic) {
            searchResult = new NKFiroe()
            return
        }
        // For constanic seeks, return the seek itself as the result
        if (unanchoredSeekFiroe.atConstanic) {
            searchResult = unanchoredSeekFiroe
            return
        }
        unwrapAnchor = result
        resolved = false  // Continue unwrapping

      case braneFiroe: BraneFiroe =>
        if (searchPerformed) {
          searchResult = braneFiroe
          return
        }

        var result = executeSearch(braneFiroe)
        searchPerformed = true

        if (result == null) {
          searchResult = new NKFiroe()
          return
        }

        if (result.atConstanic) {
            // Handled as result
        }

        // Unwrap assignment result
        result match {
          case assignment: AssignmentFiroe =>
            if assignment.isNye then
              assignment.step()
              if assignment.isNye then
                unwrapAnchor = assignment
                resolved = false  // Continue unwrapping
              else
                val res = assignment.getResult
                if res == null then
                  searchResult = new NKFiroe()
                else
                  unwrapAnchor = res
                  resolved = false  // Continue unwrapping
            else
              val res = assignment.getResult
              if res == null then
                searchResult = new NKFiroe()
              else
                unwrapAnchor = res
                resolved = false  // Continue unwrapping
          case _ =>
            // Unwrap result if it's another Firoe type
            if result.isInstanceOf[IdentifierFiroe] || result.isInstanceOf[AssignmentFiroe] || result.isInstanceOf[AbstractSearchFiroe] || result.isInstanceOf[UnanchoredSeekFiroe] then
              unwrapAnchor = result
              resolved = false  // Continue unwrapping
            else
              searchResult = result
              resolved = false  // Continue to check if result needs unwrapping
        }

      case nkFiroe: NKFiroe =>
        searchResult = new NKFiroe()
        return

      case value: ValueFiroe =>
        // For value types, the search result is the value itself
        // This handles cases like $4 which creates a OneShotSearch on a value
        searchResult = value
        return

      // Handle value types by finding their containing brane
      case _ =>
        if (searchPerformed) {
          searchResult = unwrapAnchor
        } else {
          // Try to find the containing brane for this value
          val containingBrane = unwrapAnchor.getMyBrane
          containingBrane match
            case braneFiroe: BraneFiroe =>
              // Found the containing brane - perform the search on it
              var result = executeSearch(braneFiroe)
              searchPerformed = true

              if (result == null) {
                searchResult = new NKFiroe()
                return
              }

              if (result.atConstanic) {
                // Handled as result
              }

              // Unwrap assignment result
              result match
                case assignment: AssignmentFiroe =>
                  if assignment.isNye then
                    assignment.step()
                    if assignment.isNye then
                      unwrapAnchor = assignment
                      resolved = false  // Continue unwrapping
                    else
                      val res = assignment.getResult
                      if res == null then
                        searchResult = new NKFiroe()
                      else
                        unwrapAnchor = res
                        resolved = false  // Continue unwrapping
                  else
                    val res = assignment.getResult
                    if res == null then
                      searchResult = new NKFiroe()
                    else
                      unwrapAnchor = res
                      resolved = false  // Continue unwrapping
                case _ =>
                  // Unwrap result if it's another Firoe type
                  if result.isInstanceOf[IdentifierFiroe] || result.isInstanceOf[AssignmentFiroe] || result.isInstanceOf[AbstractSearchFiroe] || result.isInstanceOf[UnanchoredSeekFiroe] then
                    unwrapAnchor = result
                    resolved = false  // Continue unwrapping
                  else
                    searchResult = result
              end match
            case _ =>
              // No containing brane found - return NK
              searchResult = new NKFiroe()
        }
        return
      }  // end of unwrapAnchor match
    }  // end of while loop
  }  // end of performSearchStep

  protected def executeSearch(target: BraneFiroe): FIR

  override def getResult: FIR = searchResult

  /**
   * Returns whether the search found a result.
   * A search is "found" if the result is not NKFiroe.
   *
   * Semantics:
   * - isFound() && CONSTANT: search found and result is fully evaluated
   * - isFound() && CONSTANIC: search found but result is unresolved
   * - !isFound() && CONSTANIC: search not found (only valid state for not found)
   * - !isFound() && CONSTANT: invalid - should not occur
   */
  def isFound: Boolean = found

  override def isAbstract: Boolean = {
    if (searchResult == null) return true
    searchResult.isAbstract
  }

  override def isNye: Boolean = {
    // searchResult == null means "found but constanic" (e.g., constanic identifier with no value)
    // In this case, isNye should return false if the search's state indicates it's done (CONSTANIC or CONSTANT)
    if (searchResult == null) {
      // Use the state-based isNye from parent class (FiroeWithBraneMind)
      // This returns false for CONSTANIC and CONSTANT states
      getNyes != Nyes.CONSTANT && getNyes != Nyes.CONSTANIC
    } else {
      searchResult.isNye
    }
  }

  override def getValue: Long = {
    if (searchResult == null) {
      throw new IllegalStateException("Search not yet evaluated")
    }
    searchResult.getValue
  }
}
