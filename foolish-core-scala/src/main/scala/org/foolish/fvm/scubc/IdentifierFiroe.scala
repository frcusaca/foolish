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
        value = braneMemory.get(identifier, 0)
          .map(_._2)
          .orNull
        if value == null then
          setNyes(Nyes.CONSTANIC)
        else
          setNyes(Nyes.CHECKED)
        1
      case Nyes.CHECKED =>
        if value.isConstanic then
          storeFirs(value)
          setNyes(value.atConstanic match
            case true => Nyes.CONSTANIC
            case false => Nyes.CONSTANT
          )
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
    copy

  override def toString: String = ast.toString
