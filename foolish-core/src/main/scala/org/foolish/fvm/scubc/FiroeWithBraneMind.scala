package org.foolish.fvm.scubc

import org.foolish.ast.AST
import scala.collection.mutable

/**
 * FIR with a braneMind queue for managing evaluation tasks.
 * The braneMind enables breadth-first execution of nested expressions.
 */
abstract class FiroeWithBraneMind(val ast: AST, val comment: Option[String] = None) extends FIR:

  protected val braneMind = mutable.Queue[FIR]()
  protected val braneMemory = mutable.ArrayBuffer[FIR]()
  private var initialized = false

  /** Initialize this FIR by setting up its state and enqueuing sub-FIRs */
  protected def initialize(): Unit

  protected def setInitialized(): Unit = initialized = true
  protected def isInitialized: Boolean = initialized

  /** Enqueues FIRs into the braneMind */
  protected def enqueueFirs(firs: FIR*): Unit =
    firs.foreach { fir =>
      braneMind.enqueue(fir)
      braneMemory.addOne(fir)
    }

  protected def enqueueExprs(exprs: AST.Expr*): Unit =
    exprs.foreach(expr => enqueueFirs(FIR.createFiroeFromExpr(expr)))

  protected def enqueueSubfirOfExprs(exprs: AST.Expr*): Unit =
    enqueueFirs(FiroeWithBraneMind.ofExpr(exprs*))

  /** A FiroeWithBraneMind is NYE if its braneMind queue is not empty */
  def isNye: Boolean = braneMind.nonEmpty

  /** Check if any FIRs in braneMind or braneMemory are abstract */
  def isAbstract: Boolean =
    braneMind.exists(_.isAbstract) || braneMemory.exists(_.isAbstract)

  /** Steps the next FIR in the braneMind queue */
  def step(): Unit =
    if isNye then
      val current = braneMind.dequeue()
      try
        current.step()
        if current.isNye then
          braneMind.enqueue(current)
      catch
        case e: Exception =>
          braneMind.prepend(current) // Re-enqueue on error
          throw RuntimeException("Error during braneMind step execution", e)

object FiroeWithBraneMind:

  def ofExpr(tasks: AST.Expr*): FiroeWithBraneMind =
    of(tasks.map(FIR.createFiroeFromExpr)*)

  def of(tasks: FIR*): FiroeWithBraneMind =
    val result = new FiroeWithBraneMind(null, None):
      protected def initialize(): Unit = setInitialized()
    result.enqueueFirs(tasks*)
    result
