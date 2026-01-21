package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}

/**
 * AbstractSearchFiroe implements the common logic for search operations.
 */
abstract class AbstractSearchFiroe(ast: AST.Expr, val operator: SearchOperator) extends FiroeWithBraneMind(ast) {
  protected var searchResult: FIR = _
  protected var unwrapAnchor: FIR = _
  protected var searchPerformed: Boolean = false

  override protected def initialize(): Unit = {
    setInitialized()
    // Subclasses must enqueue anchor
  }

  override def step(): Unit = {
    getNyes match {
      case Nyes.INITIALIZED =>
        if (stepNonBranesUntilState(Nyes.REFERENCES_IDENTIFIED)) {
          setNyes(Nyes.REFERENCES_IDENTIFIED)
        }
      case Nyes.REFERENCES_IDENTIFIED =>
        if (stepNonBranesUntilState(Nyes.ALLOCATED)) {
          setNyes(Nyes.ALLOCATED)
        }
      case Nyes.ALLOCATED =>
        if (stepNonBranesUntilState(Nyes.RESOLVED)) {
          setNyes(Nyes.RESOLVED)
        }
      case Nyes.RESOLVED =>
        if (stepNonBranesUntilState(Nyes.CONSTANT)) {
          if (isAnchorReady) {
            performSearchStep()
            if (atConstanic) {
              return
            }
            if (searchResult != null) {
              setNyes(Nyes.CONSTANT)
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

        if (result.isInstanceOf[IdentifierFiroe] || result.isInstanceOf[AssignmentFiroe] || result.isInstanceOf[AbstractSearchFiroe]) {
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
