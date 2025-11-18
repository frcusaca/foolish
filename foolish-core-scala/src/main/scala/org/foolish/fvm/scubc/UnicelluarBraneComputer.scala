package org.foolish.fvm.scubc

import org.foolish.ast.AST
import org.foolish.fvm.Env
import org.foolish.fvm.v1.Insoe

/**
 * Unicellular Brane Computer (UBC).
 *
 * The UBC is a simple computational machine that has just enough capacity to hold
 * the AST of a brane and the ability to interpret and understand a single expression
 * at a time. It proceeds from the beginning of the brane to the end, evaluating and
 * creating new values.
 *
 * The UBC has two sources of information:
 * - Ancestral Brane (AB): The search context containing the parent brane's environment
 * - Immediate Brane (IB): The current context accumulated inside the UBC so far
 */
class UnicelluarBraneComputer(
  val rootBrane: BraneFiroe,
  val ancestralContext: Env,
  val immediateContext: Env
):

  /**
   * Creates a UBC with a Brane Insoe and AB context.
   */
  def this(braneInsoe: Insoe, ancestralContext: Env) =
    this(
      {
        if braneInsoe == null then
          throw IllegalArgumentException("Brane Insoe cannot be null")

        val ast = braneInsoe.ast()
        if !ast.isInstanceOf[AST.Brane] then
          throw IllegalArgumentException("Insoe must contain a Brane AST")

        BraneFiroe(ast)
      },
      if ancestralContext != null then ancestralContext else Env(),
      {
        val ab = if ancestralContext != null then ancestralContext else Env()
        Env(ab, 0)
      }
    )

  /** Creates a UBC with a Brane Insoe and no ancestral context */
  def this(braneInsoe: Insoe) = this(braneInsoe, null)

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

  /** Returns the ancestral context (AB) */
  def getAncestralContext: Env = ancestralContext

  /** Returns the immediate context (IB) */
  def getImmedateContext: Env = immediateContext

  /**
   * Gets the final environment after full evaluation.
   * This is the frozen Env representing the fully evaluated brane.
   *
   * @return the frozen environment, or null if evaluation is not complete
   */
  def getFinalEnvironment: Env =
    if !isComplete then
      null
    else
      rootBrane.getEnvironment

object UnicelluarBraneComputer:
  def apply(rootBrane: BraneFiroe, ancestralContext: Env, immediateContext: Env): UnicelluarBraneComputer =
    new UnicelluarBraneComputer(rootBrane, ancestralContext, immediateContext)

  def apply(braneInsoe: Insoe, ancestralContext: Env): UnicelluarBraneComputer =
    new UnicelluarBraneComputer(braneInsoe, ancestralContext)

  def apply(braneInsoe: Insoe): UnicelluarBraneComputer =
    new UnicelluarBraneComputer(braneInsoe)
