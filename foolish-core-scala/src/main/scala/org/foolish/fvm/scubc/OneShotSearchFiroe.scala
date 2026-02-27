package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}

/**
 * OneShotSearchFiroe performs a one-shot search on a brane (head/tail).
 */
class OneShotSearchFiroe(oneShotSearch: AST.OneShotSearchExpr) extends AbstractSearchFiroe(oneShotSearch, oneShotSearch.operator()) {

  override protected def initialize(): Unit =
    super.initialize()
    val searchExpr = ast.asInstanceOf[AST.OneShotSearchExpr]
    enqueueExprs(searchExpr.anchor())

  override protected def executeSearch(target: BraneFiroe): FIR =
    val targetMemory = target.braneMemory
    if targetMemory.isEmpty then
      return new NKFiroe()

    operator match
      case SearchOperator.HEAD => targetMemory.get(0)
      case SearchOperator.TAIL => targetMemory.getLast
      case _ => throw new IllegalStateException("Unknown one-shot operator: " + operator)

  override def toString: String = ast.toString

  /**
   * Copy constructor for cloneConstanic.
   * Resets search state so the search can be re-executed in a new context.
   */
  private def this(original: OneShotSearchFiroe, newParent: FIR) =
    this(original.ast.asInstanceOf[AST.OneShotSearchExpr])
    setParentFir(newParent)
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
      return this  // Share CONSTANT searches
    val copy = new OneShotSearchFiroe(this, newParent)
    targetNyes.foreach(copy.setNyes)
    copy
}
