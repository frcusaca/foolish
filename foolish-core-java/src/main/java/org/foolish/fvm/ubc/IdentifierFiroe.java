package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

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
 *
 * Currently, identifier lookup is not yet implemented in UBC, so this
 * returns NK (not-known) values.
 */
public class IdentifierFiroe extends FiroeWithoutBraneMind {
    private final CharacterizedIdentifier identifier;

    public IdentifierFiroe(AST.Identifier identifier) {
        super(identifier);
        this.identifier = new CharacterizedIdentifier(identifier);
    }

    /**
     * Creates an IdentifierFiroe with explicit id and characterization strings.
     * Used when converting from other representations.
     */
    public IdentifierFiroe(String id, String characterization) {
        super(null);
        this.identifier = new CharacterizedIdentifier(id, characterization);
    }

    /**
     * Gets the CharacterizedIdentifier.
     */
    public CharacterizedIdentifier getIdentifier() {
        return identifier;
    }

    /**
     * Gets the identifier name as a string (without characterization).
     * For compatibility with existing code.
     */
    public String getId() {
        return identifier.getId();
    }

    /**
     * Gets the characterization as a flattened string.
     * Returns empty string "" if no characterization.
     */
    public String getCharacterization() {
        return identifier.getCharacterization();
    }

    /**
     * Checks if this identifier has a characterization.
     */
    public boolean hasCharacterization() {
        return identifier.hasCharacterization();
    }

    /**
     * Identifier lookup is not yet implemented, so identifiers are abstract (not-known).
     */
    @Override
    public boolean isAbstract() {
        return true;
    }

    /**
     * Identifier lookup is not yet implemented.
     * @throws UnsupportedOperationException always
     */
    @Override
    public long getValue() {
        throw new UnsupportedOperationException(
            "Identifier lookup not yet implemented in UBC. Cannot get value for: " + this);
    }

    @Override
    public String toString() {
        return ((AST.Identifier) ast).toString();
    }
}
