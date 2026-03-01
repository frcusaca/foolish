package org.foolish.fvm.scubc

import org.foolish.ast.AST
import org.foolish.fvm.scubc.BraneMemory.StrictlyMatchingQuery

/**
 * IdentifierFiroe represents a characterized identifier reference in the UBC system.
 *
 * Uses CharacterizedIdentifier to hold the identifier name and characterization:
 * - Simple identifier: `x` → id="x", characterization=""
 * - Characterized: `type'x` → id="x", characterization="type"
 *
 * The characterization is used when resolving identifiers to:
 * - Disambiguate between multiple bindings of the same name
 * - Type checking (future)
 * - Scope resolution (future)
 */
class IdentifierFiroe(override val ast: AST.Identifier)
  extends FiroeWithBraneMind(ast) with Constanicable:

  private val identifier = new StrictlyMatchingQuery(
    ast.id(),
    ast.canonicalCharacterization()
  )
  private[scubc] var value: FIR = null // Package-private for access by RegexpSearchFiroe

  def this(id: String, characterization: String) =
    this(new AST.Identifier(
      if characterization == null || characterization.isEmpty then java.util.List.of() else java.util.List.of(characterization),
      id
    ))

  /** Gets the CharacterizedIdentifier */
  def getIdentifier: CharacterizedIdentifier = identifier

  /** Gets the characterization as a flattened string */
  def getCharacterization: String = identifier.getCharacterization

  /**
   * An identifier is abstract if it hasn't been resolved yet or if its resolved value is abstract.
   */
  override def isAbstract: Boolean =
    if atConstanic then
      true
    else if value == null then
      true // Not yet resolved
    else
      value.isAbstract

  /**
   * An identifier is Constanic only when its state is CONSTANIC or CONSTANT.
   * A null value in earlier states means the identifier hasn't been evaluated yet.
   */
  override def isConstanic: Boolean =
    if getNyes != Nyes.CONSTANIC && getNyes != Nyes.CONSTANT then
      false
    else if value == null then
      true  // Not yet resolved or out of bounds
    else
      value.isConstanic

  override protected def initialize(): Unit = ()

  /**
   * Implement the step method, during checking phase we use the branemind
   * to find the value of the identifier and store a reference to it for getValue()
   */
  override def step(): Int =
    getNyes match
      case Nyes.INITIALIZED =>
        val parentMem = braneMemory.getParent
        val parentStr = if parentMem != null then
          s"parentSize=${parentMem.size} parentHash=${System.identityHashCode(parentMem)}"
        else "noParent"
        System.out.println(s"DEBUG IdentifierFiroe.step INITIALIZED: this=${System.identityHashCode(this)} identifier=$identifier braneMemory=${braneMemory.size} $parentStr")
        value = braneMemory.get(identifier, 0)
          .map(_._2)
          .orNull
        System.out.println(s"DEBUG IdentifierFiroe.step INITIALIZED: value=$value valueClass=${if (value != null) value.getClass.getSimpleName else "null"}")
        if value == null then
          setNyes(Nyes.CONSTANIC)
        else
          setNyes(Nyes.CHECKED)
        1
      case Nyes.CHECKED =>
        if value.isConstanic then
          // For CONSTANIC values, we DON'T clone them here.
          // Instead, we keep a reference to the original value and let the
          // parent (e.g., ConcatenationFiroe in performJoin) handle the cloning.
          // This ensures that when the value is cloned, its parent chain is set
          // correctly (to the ConcatenationFiroe, not the IdentifierFiroe).
          //
          // We still store the value in braneMemory for unwrapConstanicable to work,
          // but we don't enqueue it for re-evaluation (it will be re-evaluated after
          // the parent clones it).
          storeFirs(value)
          setNyes(Nyes.PRIMED)
        else if value.isConstant then
          storeFirs(value)
          setNyes(Nyes.CONSTANT)
        else
          storeFirs(value)
          braneEnqueue(value)
          setNyes(Nyes.PRIMED)
        1
      case _ =>
        super.step()

  /**
   * Get the value of the resolved identifier.
   */
  override def getValue: Long =
    if atConstanic then
      throw UnsupportedOperationException("Cannot get value from constanic identifier")
    value.getValue

  /** Returns the resolved value for unwrapping in search operations */
  override def getResult: FIR = value


  override def valuableSelf(): java.util.Optional[FIR] =
    if value == null then
      if atConstanic then
        java.util.Optional.empty[FIR]()
      else
        null  // Not ready yet
    else if atConstanic then
      java.util.Optional.empty[FIR]()
    else
      value.valuableSelf()

  override def cloneConstanic(newParent: FIR, targetNyes: Option[Nyes]): FIR =
    if !isConstanic then
      throw IllegalStateException(
        s"cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
        s"but this FIR is in state: ${getNyes}")
    if isConstant then
      return this  // Share CONSTANT identifiers
    // CONSTANIC: create fresh copy from AST
    val copy = new IdentifierFiroe(ast.asInstanceOf[AST.Identifier])
    copy.setParentFir(newParent)
    copy.value = null  // Reset for re-evaluation
    copy.setNyes(getNyes)
    // Set target state if specified
    targetNyes.foreach(copy.setNyes)
    // Ordinate to parent so identifier resolution works correctly
    // This is critical for cloned identifiers to find their values in the new context
    if !copy.ordinated then
      copy.ordinateToParentBraneMind(newParent.asInstanceOf[FiroeWithBraneMind])
    copy

  override def toString: String = ast.toString
