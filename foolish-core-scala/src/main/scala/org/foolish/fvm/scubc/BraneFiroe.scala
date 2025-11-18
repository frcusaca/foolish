package org.foolish.fvm.scubc

import org.foolish.ast.AST
import org.foolish.fvm.Env
import scala.jdk.CollectionConverters.*

/**
 * BraneFiroe represents a brane in the UBC system.
 * The BraneFiroe processes its AST to create Expression Firoes,
 * which are enqueued into the braneMind and evaluated breadth-first.
 */
class BraneFiroe(override val ast: AST, val environment: Env = null)
  extends FiroeWithBraneMind(ast):

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
      case _ =>
        throw IllegalArgumentException("AST must be of type AST.Brane")

  override def isNye: Boolean =
    getNyes != Nyes.CONSTANT

  override def step(): Unit =
    if !isInitialized then
      initialize()
      return

    super.step()

  /** Returns the frozen environment after full evaluation */
  override def getEnvironment: Env =
    if getNyes != Nyes.CONSTANT then
      throw IllegalStateException("BraneFiroe not fully evaluated")
    environment

  /** Returns the list of expression Firoes in this brane */
  def getExpressionFiroes: List[FIR] = braneMemory.toList

  override def toString: String =
    Sequencer4Human().sequence(this)

  def cloneWithABEnv(newABEnv: Env): BraneFiroe =
    BraneFiroe(this.ast, if newABEnv == null then this.environment else newABEnv)

  def cloneAbstract(): BraneFiroe = cloneWithABEnv(null)

object BraneFiroe:
  def apply(ast: AST, environment: Env): BraneFiroe =
    new BraneFiroe(ast, environment)

  def apply(ast: AST): BraneFiroe =
    new BraneFiroe(ast, null)
