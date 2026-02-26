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
            println(s"DEBUG: Calling performSearchStep, searchPerformed=$searchPerformed, searchResult=$searchResult")
            performSearchStep()
            println(s"DEBUG: After performSearchStep, searchResult=$searchResult, searchPerformed=$searchPerformed, atConstanic=$atConstanic")
            if (atConstanic) {
              return 1
            }
            if (searchResult != null) {
              // Set found status based on result type
              println(s"DEBUG: searchResult=$searchResult, found=$found")
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
            }
            // If searchResult is null, the search is not completed yet (internal stepping).
            // Stay in CHECKED state to retry when anchor becomes ready.
          }
        }
        1
      case _ =>
        println(s"DEBUG: default step case, nyes=$getNyes, braneMemory.size=${braneMemory.size}")
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
    println(s"DEBUG isAnchorReady: anchor=$anchor (class=${anchor.getClass.getSimpleName})")

    // Check if anchor is CONSTANIC
    if (anchor.atConstanic) {
      println(s"DEBUG isAnchorReady: anchor is CONSTANIC, returning true")
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
        println(s"DEBUG isAnchorReady: abstractSearch=$abstractSearch, atConstanic=${abstractSearch.atConstanic}, isNye=${abstractSearch.isNye}, getNyes=${abstractSearch.getNyes}")
        if (abstractSearch.atConstanic) {
          println(s"DEBUG isAnchorReady: returning true (abstractSearch CONSTANIC)")
          return true
        }
        if (abstractSearch.isNye) {
          abstractSearch.step()
          println(s"DEBUG isAnchorReady: returning false (abstractSearch NYE after step)")
          false
        } else {
          println(s"DEBUG isAnchorReady: returning true (abstractSearch not NYE)")
          true
        }
      case unanchoredSeek: UnanchoredSeekFiroe =>
        val seekValue = unanchoredSeek.getResult
        println(s"DEBUG isAnchorReady: unanchoredSeek=$unanchoredSeek, atConstanic=${unanchoredSeek.atConstanic}, isNye=${unanchoredSeek.isNye}, getNyes=${unanchoredSeek.getNyes}, value=$seekValue")
        if (unanchoredSeek.atConstanic) {
          println(s"DEBUG isAnchorReady: returning true (unanchoredSeek CONSTANIC)")
          return true
        }
        if (unanchoredSeek.isNye) {
          unanchoredSeek.step()
          println(s"DEBUG isAnchorReady: returning false (unanchoredSeek NYE after step)")
          false
        } else {
          println(s"DEBUG isAnchorReady: returning true (unanchoredSeek not NYE)")
          true
        }
      case _ =>
        println(s"DEBUG isAnchorReady: returning true (default case)")
        true
    }
    end match
  }

  protected def performSearchStep(): Unit = {
    println(s"DEBUG performSearchStep: entered, searchPerformed=$searchPerformed, braneMemory.size=${braneMemory.size}")
    if (unwrapAnchor == null && !searchPerformed) {
      if (braneMemory.isEmpty) {
        searchResult = new NKFiroe()
        return
      }
      val last = braneMemory.getLast
      println(s"DEBUG performSearchStep: braneMemory.getLast=$last (class=${last.getClass.getSimpleName})")
      unwrapAnchor = last
      println(s"DEBUG performSearchStep: set unwrapAnchor=$unwrapAnchor (class=${unwrapAnchor.getClass.getSimpleName})")
    }

    if (searchResult != null) return

    // Check for constanic anchor
    if (unwrapAnchor.atConstanic) {
        searchResult = new NKFiroe()
        return
    }

    // Loop to fully resolve the anchor (unwrap through identifiers, assignments, searches)
    var resolved = false
    while (!resolved) {
      resolved = true  // Assume resolved unless we find something to unwrap

      unwrapAnchor match {
      case identifierFiroe: IdentifierFiroe =>
        if (identifierFiroe.atConstanic) {
            searchResult = new NKFiroe()
            return
        }
        if (identifierFiroe.isNye) {
          identifierFiroe.step()
          if (identifierFiroe.isNye) {
            return  // Will retry on next step
          }
        }
        unwrapAnchor = identifierFiroe.value
        if (unwrapAnchor == null) searchResult = new NKFiroe()
        resolved = false  // Continue unwrapping

      case assignmentFiroe: AssignmentFiroe =>
        println(s"DEBUG case assignmentFiroe: unwrapAnchor=$unwrapAnchor, atConstanic=${assignmentFiroe.atConstanic}, isNye=${assignmentFiroe.isNye}, getNyes=${assignmentFiroe.getNyes}")
        // For search operations, constanic assignments should still be unwrapped
        // to get their result (e.g., g = {...} where the brane is constanic)
        if (assignmentFiroe.isNye) {
          assignmentFiroe.step()
          println(s"DEBUG: After step, isNye=${assignmentFiroe.isNye}")
          if (assignmentFiroe.isNye) {
            return  // Will retry on next step
          }
        }
        val res = assignmentFiroe.getResult
        println(s"DEBUG: assignmentFiroe.getResult = $res (class=${res.getClass.getSimpleName})")
        if (res == null) {
          println(s"DEBUG: searchResult = NKFiroe (result is null)")
          searchResult = new NKFiroe()
        }
        else {
          unwrapAnchor = res
          println(s"DEBUG: unwrapAnchor = $unwrapAnchor")
        }
        resolved = false  // Continue unwrapping

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
        println(s"DEBUG: case unanchoredSeekFiroe, atConstanic=${unanchoredSeekFiroe.atConstanic}, isNye=${unanchoredSeekFiroe.isNye}, getNyes=${unanchoredSeekFiroe.getNyes}")
        if (unanchoredSeekFiroe.isNye) {
          unanchoredSeekFiroe.step()
          println(s"DEBUG: After step, isNye=${unanchoredSeekFiroe.isNye}, getNyes=${unanchoredSeekFiroe.getNyes}")
          if (unanchoredSeekFiroe.isNye) {
            return  // Will retry on next step
          }
        }
        val result = unanchoredSeekFiroe.getResult
        println(s"DEBUG: unanchoredSeekFiroe.getResult = $result (class=${if result != null then result.getClass.getSimpleName else "null"})")
        if (result == null && !unanchoredSeekFiroe.atConstanic) {
            println(s"DEBUG: searchResult = NKFiroe (result is null)")
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
        println(s"DEBUG: braneFiroe case, searchPerformed=$searchPerformed, braneMemory.size=${braneFiroe.braneMemory.size}")
        if (searchPerformed) {
          searchResult = braneFiroe
          return
        }

        var result = executeSearch(braneFiroe)
        println(s"DEBUG: After executeSearch, result=$result (class=${result.getClass.getSimpleName})")
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
              println(s"DEBUG: Setting searchResult = $result (class=${result.getClass.getSimpleName})")
              searchResult = result
        }

      case nkFiroe: NKFiroe =>
        searchResult = new NKFiroe()
        return

      // Handle value types by finding their containing brane
      case _ =>
        if (searchPerformed) {
          searchResult = unwrapAnchor
        } else {
          // Try to find the containing brane for this value
          val containingBrane = unwrapAnchor.getMyBrane
          println(s"DEBUG: unwrapAnchor=${unwrapAnchor.getClass.getSimpleName}, containingBrane=$containingBrane, searchPerformed=$searchPerformed")
          containingBrane match
            case braneFiroe: BraneFiroe =>
              // Found the containing brane - perform the search on it
              println(s"DEBUG: Calling executeSearch on brane $braneFiroe")
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
    val result = if (searchResult == null) true else searchResult.isNye
    if (result) println(s"DEBUG AbstractSearchFiroe.isNye: searchResult=$searchResult, searchResult.isNye=${if searchResult != null then searchResult.isNye else "N/A"}")
    result
  }

  override def getValue: Long = {
    if (searchResult == null) {
      throw new IllegalStateException("Search not yet evaluated")
    }
    searchResult.getValue
  }
}
