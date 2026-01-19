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

    // Unwrap identifier
    val resolvedAnchor = anchor match {
      case identifierFiroe: IdentifierFiroe =>
        identifierFiroe.state match {
          case FiroeState.Constantic() => return true
          case FiroeState.Value(fir) => fir
          case _ => return false
        }
      case _ => anchor
    }
    if (resolvedAnchor == null) return false
    anchor = resolvedAnchor

    // Unwrap assignment
    val resolvedAnchor2 = anchor match {
      case assignmentFiroe: AssignmentFiroe =>
        if (assignmentFiroe.isNye) {
          assignmentFiroe.step()
          return false
        }
        assignmentFiroe.getFiroeState match {
          case FiroeState.Constantic() => return true
          case FiroeState.Value(fir) => fir
          case _ => return false
        }
      case _ => anchor
    }
    if (resolvedAnchor2 == null) return false
    anchor = resolvedAnchor2

    // Check chained search
    anchor match {
      case abstractSearch: AbstractSearchFiroe =>
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

    unwrapAnchor match {
      case identifierFiroe: IdentifierFiroe =>
        identifierFiroe.state match {
          case FiroeState.Constantic() =>
             searchResult = new NKFiroe()
             return
          case FiroeState.Value(fir) =>
             unwrapAnchor = fir
             return
          case _ => return
        }

      case assignmentFiroe: AssignmentFiroe =>
        if (assignmentFiroe.isNye) {
          assignmentFiroe.step()
          return
        }
        assignmentFiroe.getFiroeState match {
          case FiroeState.Constantic() =>
             searchResult = new NKFiroe()
             return
          case FiroeState.Value(fir) =>
             unwrapAnchor = fir
             return
          case _ => return
        }

      case abstractSearch: AbstractSearchFiroe =>
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
