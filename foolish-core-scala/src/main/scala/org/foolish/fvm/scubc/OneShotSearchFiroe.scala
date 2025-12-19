package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}

/**
 * OneShotSearchFiroe performs a one-shot search on a brane (head/tail).
 */
class OneShotSearchFiroe(oneShotSearch: AST.OneShotSearchExpr) extends AbstractSearchFiroe(oneShotSearch, oneShotSearch.operator()) {

  override protected def initialize(): Unit = {
    super.initialize()
    val searchExpr = ast.asInstanceOf[AST.OneShotSearchExpr]
    enqueueExprs(searchExpr.anchor())
  }

  override protected def executeSearch(target: BraneFiroe): FIR = {
    val targetMemory = target.braneMemory
    if (targetMemory.isEmpty) {
      return new NKFiroe()
    }

    operator match {
      case SearchOperator.HEAD => targetMemory.get(0)
      case SearchOperator.TAIL => targetMemory.getLast
      case _ => throw new IllegalStateException("Unknown one-shot operator: " + operator)
    }
  }

  override def toString: String = ast.toString
}
