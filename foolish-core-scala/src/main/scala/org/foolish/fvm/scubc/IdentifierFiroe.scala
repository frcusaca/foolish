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
  extends FiroeWithBraneMind(ast):

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

  override protected def initialize(): Unit = ()

  /**
   * Implement the step method, during checking phase we use the branemind
   * to find the value of the identifier and store a reference to it for getValue()
   */
  override def step(): Unit =
    getNyes match
      case Nyes.INITIALIZED =>
        value = braneMemory.get(identifier, 0)
          .map(_._2)
          .orNull
        if value == null then
          setNyes(Nyes.CONSTANIC)
        else
          setNyes(Nyes.CHECKED)
      case _ =>
        super.step()

  /**
   * Get the value of the resolved identifier.
   */
  override def getValue: Long =
    if atConstanic then
      throw UnsupportedOperationException("Cannot get value from constanic identifier")
    value.getValue

  override def toString: String = ast.toString
