package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * DerefSearchFiroe is a specialized RegexpSearchFiroe for exact match searches.
 * It is used when the search pattern contains no wildcards (and thus is an exact match)
 * or when explicitly created from a DereferenceExpr.
 */
class DerefSearchFiroe(regexpSearch: AST.RegexpSearchExpr, originalAst: AST = null)
  extends RegexpSearchFiroe(regexpSearch) {

  override def toString: String = {
    if (originalAst != null) {
      originalAst.toString
    } else {
      super.toString
    }
  }
}

object DerefSearchFiroe {
  /**
   * Checks if the pattern contains any regex wildcards or special characters.
   * If false, the pattern is considered an exact match string.
   */
  def isExactMatch(pattern: String): Boolean = {
    if (pattern == null) return true
    for (c <- pattern.toCharArray) {
      // Check for standard regex metacharacters
      if ("*+?^$[](){} |\\.".indexOf(c) >= 0) {
        return false
      }
    }
    true
  }
}
