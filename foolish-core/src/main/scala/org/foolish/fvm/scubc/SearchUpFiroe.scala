package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * SearchUpFiroe is a Brane wrapper for the search-up (↑) operation.
 * It has a reference to a brane Firoe that should already exist at this point.
 */
class SearchUpFiroe(
  override val ast: AST.SearchUP,
  var referencedBrane: Option[BraneFiroe] = None
) extends FiroeWithoutBraneMind(ast):

  def this(searchUp: AST.SearchUP) = this(searchUp, None)

  /** Sets the brane that this SearchUpFiroe references */
  def setReferencedBrane(brane: BraneFiroe): Unit =
    referencedBrane = Some(brane)

  /** SearchUpFiroe is abstract if the referenced brane is abstract or not set */
  def isAbstract: Boolean =
    referencedBrane.forall(_.isAbstract)

  override def toString: String = "↑"
