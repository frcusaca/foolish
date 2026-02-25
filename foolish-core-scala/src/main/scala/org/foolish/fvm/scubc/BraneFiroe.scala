package org.foolish.fvm.scubc

import org.foolish.ast.AST
import scala.jdk.CollectionConverters.*

/**
 * BraneFiroe represents a brane in the UBC system.
 * The BraneFiroe processes its AST to create Expression Firoes,
 * which are enqueued into the braneMind and evaluated breadth-first.
 */
class BraneFiroe(override val ast: AST)
  extends FiroeWithBraneMind(ast) with Constanicable:

  // Register this brane as the owner of its memory
  braneMemory.setOwningBrane(this)

  /**
   * Copy constructor for cloneConstanic.
   * Creates a copy with independent braneMemory and updated parent chain.
   *
   * @param original the BraneFiroe to copy
   * @param newParent the new parent for this clone
   */
  private def this(original: BraneFiroe, newParent: FIR) =
    this(original.ast)
    setParentFir(newParent)
    // Re-register this brane as the owner of the new braneMemory
    braneMemory.setOwningBrane(this)
    // Copy braneMemory contents with cloned items using the parent's copy mechanism
    original.braneMemory.stream.foreach { fir =>
      val cloned = fir.asInstanceOf[Constanicable].cloneConstanic(this, Some(Nyes.INITIALIZED))
      braneMemory.put(cloned)
      indexLookup.put(cloned, braneMemory.size - 1)
      // For nested FiroeWithBraneMind instances, set up parent memory link
      if cloned.isInstanceOf[FiroeWithBraneMind] then
        // Reset ordinated flag so we can re-ordinate in new context
        cloned.asInstanceOf[FiroeWithBraneMind].ordinated = false
        cloned.asInstanceOf[FiroeWithBraneMind].ordinateToParentBraneMind(this, indexLookup.size - 1)
    }
    // Set state to INITIALIZED so children can re-evaluate
    setNyes(Nyes.INITIALIZED)

  /** Initialize the BraneFiroe by converting AST statements to Expression Firoes */
  override protected def initialize(): Unit =
    if isInitialized then return
    setInitialized()

    ast match
      case brane: AST.Brane =>
        brane.statements().asScala.foreach { expr =>
          val firoe = FIR.createFiroeFromExpr(expr)
          storeFirs(firoe)
        }
      case _ =>
        throw IllegalArgumentException("AST must be of type AST.Brane")

  override def isNye: Boolean =
    getNyes != Nyes.CONSTANT && getNyes != Nyes.CONSTANIC

  override def step(): Int =
    if !isInitialized then
      initialize()
      return 1

    super.step()

  /** Returns the list of expression Firoes in this brane */
  def getExpressionFiroes: List[FIR] = braneMemory.stream.toList

  override def toString: String =
    Sequencer4Human().sequence(this)

  /**
   * Clones this BraneFiroe for use in a new context.
   * Used during brane concatenation.
   */
  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    if !isConstanic then
      throw IllegalStateException(
        s"cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
        s"but this FIR is in state: ${getNyes}")

    if isConstant then
      return this  // Share CONSTANT branes completely

    // CONSTANIC: use copy constructor
    val copy = new BraneFiroe(this, newParent)

    // Set target state if specified, otherwise reset to INITIALIZED
    // because children were reset to INITIALIZED by the copy constructor.
    if targetNyes.isDefined then
      copy.setNyes(targetNyes.get)
    else
      // Cannot copy this.nyes (CONSTANIC) because children are INITIALIZED.
      // Must allow re-evaluation.
      copy.setNyes(Nyes.INITIALIZED)

    copy

object BraneFiroe:
  def apply(ast: AST): BraneFiroe =
    new BraneFiroe(ast)
