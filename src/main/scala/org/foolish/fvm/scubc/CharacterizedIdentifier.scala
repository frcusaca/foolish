package org.foolish.fvm.scubc

import org.foolish.ast.AST

/**
 * CharacterizedIdentifier represents an identifier with its optional characterization.
 *
 * In Foolish, identifiers can be characterized (typed) using the apostrophe syntax:
 * - Simple identifier: `x` → id="x", characterization=""
 * - Characterized: `type'x` → id="x", characterization="type"
 * - Chained: `outer'inner'x` → id="x", characterization="outer'inner"
 */
case class CharacterizedIdentifier(id: String, characterization: String = ""):

  /** Creates a CharacterizedIdentifier from an AST.Identifier */
  def this(identifier: AST.Identifier) =
    this(identifier.id(), identifier.canonicalCharacterization())

  /** Gets the identifier name (Java-style getter for compatibility) */
  def getId: String = id

  /** Gets the characterization (Java-style getter for compatibility) */
  def getCharacterization: String = characterization

  /** Checks if this identifier has a non-empty characterization */
  def hasCharacterization: Boolean = characterization.nonEmpty

  /** Returns the full characterized identifier string in Foolish syntax */
  def toFoolishString: String =
    if hasCharacterization then s"$characterization'$id"
    else id

  override def toString: String = toFoolishString

object CharacterizedIdentifier:
  def apply(identifier: AST.Identifier): CharacterizedIdentifier =
    CharacterizedIdentifier(identifier.id(), identifier.canonicalCharacterization())
