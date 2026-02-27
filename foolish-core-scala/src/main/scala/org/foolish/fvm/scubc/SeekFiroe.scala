package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}

/**
 * SeekFiroe implements offset-based access to a brane.
 * Syntax: b#0 (first), b#-1 (last).
 */
class SeekFiroe(seekExpr: AST.SeekExpr) extends AbstractSearchFiroe(seekExpr, SearchOperator.SEEK) {

  private var offset: Int = seekExpr.offset()

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

  /**
   * Copy constructor for cloneConstanic.
   * Resets search state so the search can be re-executed in a new context.
   */
  private def this(original: SeekFiroe, newParent: FIR) =
    this(original.ast.asInstanceOf[AST.SeekExpr])
    setParentFir(newParent)
    offset = original.offset
    // Copy braneMemory contents with cloned items
    original.braneMemory.stream.foreach { fir =>
      val cloned = fir.asInstanceOf[Constanicable].cloneConstanic(this, Some(Nyes.INITIALIZED))
      braneMemory.put(cloned)
      indexLookup.put(cloned, braneMemory.size - 1)
      // For nested FiroeWithBraneMind instances, set up parent memory link
      if cloned.isInstanceOf[FiroeWithBraneMind] then
        cloned.asInstanceOf[FiroeWithBraneMind].ordinated = false
        cloned.asInstanceOf[FiroeWithBraneMind].ordinateToParentBraneMind(this, indexLookup.size - 1)
    }

  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    if !isConstanic then
      throw IllegalStateException(
        s"cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
        s"but this FIR is in state: ${getNyes}")
    if isConstant then
      return this  // Share CONSTANT seeks
    val copy = new SeekFiroe(this, newParent)
    targetNyes.foreach(copy.setNyes)
    copy
}
