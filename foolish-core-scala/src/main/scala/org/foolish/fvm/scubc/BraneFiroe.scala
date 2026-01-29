package org.foolish.fvm.scubc

import org.foolish.ast.AST
import scala.jdk.CollectionConverters.*

/**
 * BraneFiroe represents a brane in the UBC system.
 * The BraneFiroe processes its AST to create Expression Firoes,
 * which are enqueued into the braneMind and evaluated breadth-first.
 */
class BraneFiroe(override val ast: AST)
  extends FiroeWithBraneMind(ast):

  // Register this brane as the owner of its memory
  braneMemory.setOwningBrane(this)

  /** Initialize the BraneFiroe by converting AST statements to Expression Firoes */
  override protected def initialize(): Unit =
    if isInitialized then return
    setInitialized()

    ast match
      case brane: AST.Brane =>
        brane.statements().asScala.foreach { expr =>
          val firoe = FIR.createFiroeFromExpr(expr)
          enqueueFirs(firoe)
        }
      case dBrane: AST.DetachmentBrane =>
        dBrane.statements().asScala.foreach { stmt =>
          // Detachment statements are not fully supported yet, represent as NK
          enqueueFirs(new NKFiroe())
        }
      case _ =>
        throw IllegalArgumentException("AST must be of type AST.Brane or AST.DetachmentBrane")

  override def isNye: Boolean =
    getNyes != Nyes.CONSTANT

  override def step(): Unit =
    if !isInitialized then
      initialize()
      return

    super.step()

  /** Returns the list of expression Firoes in this brane */
  def getExpressionFiroes: List[FIR] = braneMemory.stream.toList

  override def toString: String =
    Sequencer4Human().sequence(this)

object BraneFiroe:
  def apply(ast: AST): BraneFiroe =
    new BraneFiroe(ast)
