package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * Unicellular Brane Computer (UBC).
 *
 * The UBC is a simple computational machine that has just enough capacity to hold
 * the AST of a brane and the ability to interpret and understand a single expression
 * at a time. It proceeds from the beginning of the brane to the end, evaluating and
 * creating new values.
 */
class UnicelluarBraneComputer(
  val rootBrane: BraneFiroe
):

  /**
   * Creates a UBC with a Brane AST.
   */
  def this(braneAst: AST) =
    this({
      if braneAst == null then
        throw IllegalArgumentException("Brane AST cannot be null")

      if !braneAst.isInstanceOf[AST.Brane] then
        throw IllegalArgumentException("AST must be a Brane")

      BraneFiroe(braneAst)
    })

  /**
   * Takes a single evaluation step.
   * Steps forward from the braneMind until it's empty, at which time it returns false.
   *
   * @return true if more steps are needed, false if evaluation is complete
   */
  def step(): Boolean =
    if !rootBrane.isNye then
      return false

    rootBrane.step()
    rootBrane.isNye

  /**
   * Runs the UBC until evaluation is complete.
   *
   * @return the number of steps taken
   */
  def runToCompletion(): Int =
    var steps = 0
    while step() do
      steps += 1

      // Safety check to prevent infinite loops
      if steps > 100000 then
        throw RuntimeException("Evaluation exceeded maximum step count (possible infinite loop)")

    steps

  /** Returns true if the UBC has completed evaluation */
  def isComplete: Boolean = !rootBrane.isNye

  /** Returns the root BraneFiroe being evaluated */
  def getRootBrane: BraneFiroe = rootBrane

object UnicelluarBraneComputer:
  def apply(rootBrane: BraneFiroe): UnicelluarBraneComputer =
    new UnicelluarBraneComputer(rootBrane)

  def apply(braneAst: AST): UnicelluarBraneComputer =
    new UnicelluarBraneComputer(braneAst)
