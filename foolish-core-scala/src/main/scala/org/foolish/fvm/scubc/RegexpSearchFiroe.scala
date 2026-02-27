package org.foolish.fvm.scubc

import org.foolish.ast.{AST, SearchOperator}

import java.util.Optional

/**
 * RegexpSearchFiroe performs a regular expression search on a brane.
 */
class RegexpSearchFiroe(regexpSearch: AST.RegexpSearchExpr) extends AbstractSearchFiroe(regexpSearch, regexpSearch.operator()) {
  private var pattern: String = regexpSearch.pattern()

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

  /**
   * Copy constructor for cloneConstanic.
   * Resets search state so the search can be re-executed in a new context.
   */
  private def this(original: RegexpSearchFiroe, newParent: FIR) =
    this(original.ast.asInstanceOf[AST.RegexpSearchExpr])
    setParentFir(newParent)
    pattern = original.pattern
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
    val copy = new RegexpSearchFiroe(this, newParent)
    targetNyes.foreach(copy.setNyes)
    copy
}
