package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * FIR for assignment expressions.
 * An assignment evaluates its right-hand side expression and stores the result
 * with a coordinate name in the brane's environment.
 */
class AssignmentFiroe(assignment: AST.Assignment)
  extends FiroeWithBraneMind(assignment):

  val lhs = CharacterizedIdentifier(assignment.identifier())
  private var result: Option[FIR] = None

  override protected def initialize(): Unit =
    if isInitialized then return
    setInitialized()

    enqueueExprs(assignment.expr())

  override def step(): Unit =
    if result.isDefined then
      return

    if atConstantic then
      return

    if !isInitialized then
      initialize()
      return

    // Let parent class handle braneMind stepping and state transitions
    super.step()

    // Check if we can get the final result
    if !super.isNye && !braneMemory.isEmpty then
      val res = braneMemory.get(0)
      result = Some(res)
      if res.atConstantic then
        setNyes(Nyes.CONSTANTIC)

  override def isAbstract: Boolean =
    if atConstantic then
      true
    else
      result.map(_.isAbstract).getOrElse(super.isAbstract)

  override def isNye: Boolean =
    result.isEmpty && !atConstantic

  /** Gets the coordinate name for this assignment (without characterization) */
  def getId: String = lhs.getId

  /** Gets the LHS characterized identifier */
  def getLhs: CharacterizedIdentifier = lhs

  /** Gets the evaluated result FIR */
  def getResult: Option[FIR] = result

  override def getValue: Long =
    if atConstantic then
      throw IllegalStateException("AssignmentFiroe is constantic")
    result.map(_.getValue).getOrElse(
      throw IllegalStateException("AssignmentFiroe not fully evaluated"))

  override def toString: String =
    Sequencer4Human().sequence(this)

object AssignmentFiroe:
  def apply(assignment: AST.Assignment): AssignmentFiroe =
    new AssignmentFiroe(assignment)
