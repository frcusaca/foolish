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
    val searchFrom = targetMemory.size - 1

    val result = operator match {
      case SearchOperator.REGEXP_LOCAL => targetMemory.getLocal(query, searchFrom)
      case SearchOperator.REGEXP_GLOBAL => targetMemory.get(query, searchFrom)
      case _ => throw new IllegalStateException("Unknown regexp operator: " + operator)
    }

    if (result.isDefined) result.get._2 else new NKFiroe()
  }

  override def toString: String = ast.toString
}
