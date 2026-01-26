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

  override def step(): Unit = {
    getNyes match {
      case Nyes.INITIALIZED =>
        if (stepNonBranesUntilState(Nyes.CHECKED)) {
          setNyes(Nyes.CHECKED)
        }
      case Nyes.CHECKED =>
        if (stepNonBranesUntilState(Nyes.CONSTANT)) {
          if (isAnchorReady) {
            performSearchStep()
            if (atConstanic) {
              return
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
      case _ => super.step()
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
        if (assignmentFiroe.getResult.isEmpty) return false
        assignmentFiroe.getResult.get
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
        if (assignmentFiroe.atConstanic) {
            searchResult = new NKFiroe()
            return
        }
        if (assignmentFiroe.isNye) {
          assignmentFiroe.step()
          return
        }
        val res = assignmentFiroe.getResult
        if (res.isEmpty) searchResult = new NKFiroe()
        else unwrapAnchor = res.get
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
        unwrapAnchor = unanchoredSeekFiroe.getResult
        if (unwrapAnchor == null) searchResult = new NKFiroe()
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
            if (res.isEmpty) result = new NKFiroe()
            else result = res.get
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

      case _ =>
        if (searchPerformed) {
          searchResult = unwrapAnchor
        } else {
          searchResult = new NKFiroe()
        }
        return
    }
  }

  protected def executeSearch(target: BraneFiroe): FIR

  def getResult: FIR = searchResult

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
