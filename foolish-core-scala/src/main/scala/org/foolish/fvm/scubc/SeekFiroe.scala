package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}

/**
 * SeekFiroe implements offset-based access to a brane.
 * Syntax: b#0 (first), b#-1 (last).
 */
class SeekFiroe(seekExpr: AST.SeekExpr) extends AbstractSearchFiroe(seekExpr, SearchOperator.SEEK) {

  private val offset: Int = seekExpr.offset()

  override protected def initialize(): Unit = {
    super.initialize()
    val expr = ast.asInstanceOf[AST.SeekExpr]
    enqueueExprs(expr.anchor())
  }

  override protected def executeSearch(target: BraneFiroe): FIR = {
    val targetMemory = target.braneMemory
    val size = targetMemory.size
    var idx = offset

    // Handle negative indexing
    if (idx < 0) {
      idx = size + idx
    }

    // Check bounds
    if (idx >= 0 && idx < size) {
      targetMemory.get(idx)
    } else {
      new NKFiroe()
    }
  }

  override def toString: String = ast.toString
}
