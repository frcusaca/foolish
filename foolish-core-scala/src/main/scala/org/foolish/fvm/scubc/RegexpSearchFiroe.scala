package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}

import java.util.Optional

/**
 * RegexpSearchFiroe performs a regular expression search on a brane.
 */
class RegexpSearchFiroe(regexpSearch: AST.RegexpSearchExpr) extends AbstractSearchFiroe(regexpSearch, regexpSearch.operator()) {
  private val pattern: String = regexpSearch.pattern()

  override protected def initialize(): Unit = {
    super.initialize()
    val searchExpr = ast.asInstanceOf[AST.RegexpSearchExpr]
    enqueueExprs(searchExpr.anchor())
  }

  override protected def executeSearch(target: BraneFiroe): FIR = {
    val query = new BraneMemory.RegexpQuery(pattern)
    val targetMemory = target.braneMemory

    val result = operator match {
      case SearchOperator.REGEXP_LOCAL =>
        // Backward search: search from end to start (find last match)
        val searchFrom = targetMemory.size - 1
        targetMemory.getLocal(query, searchFrom)
      case SearchOperator.REGEXP_FORWARD_LOCAL =>
        // Forward search: search from start to end (find first match)
        val searchFrom = 0
        targetMemory.getLocalForward(query, searchFrom)
      case SearchOperator.REGEXP_GLOBAL =>
        // Global backward search (find-all, not yet fully implemented)
        val searchFrom = targetMemory.size - 1
        targetMemory.get(query, searchFrom)
      case _ => throw new IllegalStateException("Unknown regexp operator: " + operator)
    }

    if (result.isDefined) result.get._2 else new NKFiroe()
  }

  override def toString: String = ast.toString
}
