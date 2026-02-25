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
        if (stepNonBranesUntilState(Nyes.CONSTANT)) {
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
                  setNyes(Nyes.CONSTANT)
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
        if (abstractSearch.atConstanic) return true
        if (abstractSearch.isNye) {
          abstractSearch.step()
          false
        } else {
          true
        }
      case _ => true
    }
  }

  protected def performSearchStep(): Unit = {
    if (unwrapAnchor == null && !searchPerformed) {
      if (braneMemory.isEmpty) {
        searchResult = new NKFiroe()
        return
      }
      unwrapAnchor = braneMemory.getLast
    }

    if (searchResult != null) return

    // Check for constanic anchor
    if (unwrapAnchor.atConstanic) {
        searchResult = new NKFiroe()
        return
    }

    unwrapAnchor match {
      case identifierFiroe: IdentifierFiroe =>
        if (identifierFiroe.atConstanic) {
            searchResult = new NKFiroe()
            return
        }
        unwrapAnchor = identifierFiroe.value
        if (unwrapAnchor == null) searchResult = new NKFiroe()
        return

      case assignmentFiroe: AssignmentFiroe =>
        println(s"DEBUG: assignmentFiroe=$assignmentFiroe, atConstanic=${assignmentFiroe.atConstanic}, isNye=${assignmentFiroe.isNye}, result=${assignmentFiroe.getResult}")
        if (assignmentFiroe.atConstanic) {
            searchResult = new NKFiroe()
            return
        }
        if (assignmentFiroe.isNye) {
          println(s"DEBUG: assignment is NYE, stepping...")
          assignmentFiroe.step()
          return
        }
        val res = assignmentFiroe.getResult
        println(s"DEBUG: assignment result=$res")
        if (res == null) searchResult = new NKFiroe()
        else unwrapAnchor = res
        return

      case abstractSearch: AbstractSearchFiroe =>
        if (abstractSearch.atConstanic) {
            searchResult = new NKFiroe()
            return
        }
        if (abstractSearch.isNye) {
          abstractSearch.step()
          return
        }
        unwrapAnchor = abstractSearch.getResult
        if (unwrapAnchor == null) searchResult = new NKFiroe()
        return

      case unanchoredSeekFiroe: UnanchoredSeekFiroe =>
        if (unanchoredSeekFiroe.atConstanic) {
            searchResult = new NKFiroe()
            return
        }
        if (unanchoredSeekFiroe.isNye) {
          unanchoredSeekFiroe.step()
          return
        }
        val result = unanchoredSeekFiroe.getResult
        if (result == null) {
            searchResult = new NKFiroe()
            return
        }
        unwrapAnchor = result
        return

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
            val res = assignment.getResult
            if (res == null) result = new NKFiroe()
            else result = res
          case _ =>
        }

        if (result.isInstanceOf[IdentifierFiroe] || result.isInstanceOf[AssignmentFiroe] || result.isInstanceOf[AbstractSearchFiroe] || result.isInstanceOf[UnanchoredSeekFiroe]) {
          unwrapAnchor = result
          return
        }

        searchResult = result
        return

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
                  val res = assignment.getResult
                  if res == null then result = new NKFiroe()
                  else result = res
                case _ =>
              end match

              if result.isInstanceOf[IdentifierFiroe] || result.isInstanceOf[AssignmentFiroe] || result.isInstanceOf[AbstractSearchFiroe] || result.isInstanceOf[UnanchoredSeekFiroe] then
                unwrapAnchor = result
                return
              end if

              searchResult = result
              return
            case _ =>
              // No containing brane found - return NK
              searchResult = new NKFiroe()
        }
        return
    }
  }

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

  override def getValue: Long = {
    if (searchResult == null) {
      throw new IllegalStateException("Search not yet evaluated")
    }
    searchResult.getValue
  }
}
