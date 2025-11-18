package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * IdentifierFiroe represents a characterized identifier reference in the UBC system.
 *
 * Uses CharacterizedIdentifier to hold the identifier name and characterization.
 * Currently, identifier lookup is not yet implemented in UBC, so this
 * returns NK (not-known) values.
 */
class IdentifierFiroe(
  override val ast: AST.Identifier,
  val identifier: CharacterizedIdentifier
) extends FiroeWithoutBraneMind(ast):

  def this(identifier: AST.Identifier) =
    this(identifier, CharacterizedIdentifier(identifier))

  def this(id: String, characterization: String) =
    this(null, CharacterizedIdentifier(id, characterization))

  /** Gets the identifier name as a string (without characterization) */
  def getId: String = identifier.getId

  /** Gets the characterization as a flattened string */
  def getCharacterization: String = identifier.getCharacterization

  /** Checks if this identifier has a characterization */
  def hasCharacterization: Boolean = identifier.hasCharacterization

  /** Identifier lookup is not yet implemented, so identifiers are abstract */
  def isAbstract: Boolean = true

  /** Identifier lookup is not yet implemented */
  override def getValue: Long =
    throw UnsupportedOperationException(
      s"Identifier lookup not yet implemented in UBC. Cannot get value for: $this")

  override def toString: String = ast.toString
