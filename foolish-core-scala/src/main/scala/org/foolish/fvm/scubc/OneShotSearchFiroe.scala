package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}

/**
 * OneShotSearchFiroe performs a one-shot search on a brane (head/tail).
 */
class OneShotSearchFiroe(oneShotSearch: AST.OneShotSearchExpr) extends AbstractSearchFiroe(oneShotSearch, oneShotSearch.operator()) {

  override protected def initialize(): Unit = {
    println(s"DEBUG OneShotSearchFiroe.initialize: entering, searchPerformed=$searchPerformed, braneMemory.size=${braneMemory.size}")
    super.initialize()
    val searchExpr = ast.asInstanceOf[AST.OneShotSearchExpr]
    println(s"DEBUG OneShotSearchFiroe.initialize: after super.initialize, braneMemory.size=${braneMemory.size}")
    enqueueExprs(searchExpr.anchor())
    println(s"DEBUG OneShotSearchFiroe.initialize: after enqueueExprs, braneMemory.size=${braneMemory.size}, braneMind.size=${braneMind.size}")
  }

  override protected def executeSearch(target: BraneFiroe): FIR = {
    val targetMemory = target.braneMemory
    println(s"DEBUG OneShotSearchFiroe: executeSearch operator=$operator, targetMemory.size=${targetMemory.size}")
    if (targetMemory.isEmpty) {
      val nk = new NKFiroe()
      println(s"DEBUG OneShotSearchFiroe: returning NK (empty memory)")
      return nk
    }

    val result = operator match {
      case SearchOperator.HEAD =>
        val head = targetMemory.get(0)
        println(s"DEBUG OneShotSearchFiroe: HEAD returning $head (class=${head.getClass.getSimpleName})")
        head
      case SearchOperator.TAIL =>
        val last = targetMemory.getLast
        println(s"DEBUG OneShotSearchFiroe: TAIL returning $last (class=${last.getClass.getSimpleName})")
        last
      case _ => throw new IllegalStateException("Unknown one-shot operator: " + operator)
    }
    println(s"DEBUG OneShotSearchFiroe: executeSearch returning $result (class=${result.getClass.getSimpleName})")
    result
  }

  override def toString: String = ast.toString
}
