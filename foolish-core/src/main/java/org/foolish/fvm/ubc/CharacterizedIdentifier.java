package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * CharacterizedIdentifier represents an identifier with its optional characterization.
 *
 * In Foolish, identifiers can be characterized (typed) using the apostrophe syntax:
 * - Simple identifier: `x` → id="x", characterization=""
 * - Characterized: `type'x` → id="x", characterization="type"
 * - Chained: `outer'inner'x` → id="x", characterization="outer'inner"
 *
 * This class is used throughout the UBC FIR system to represent identifiable entities
 * with their characterizations, aiding in:
 * - Identifier resolution and lookup
 * - Type checking (future)
 * - Scope disambiguation
 * - Coordinate access in branes
 *
 * CharacterizedIdentifier is immutable and can be easily created from AST.Identifier
 * or constructed directly with strings.
 */
public class CharacterizedIdentifier {
    private final String id;
    private final String characterization;

    /**
     * Creates a CharacterizedIdentifier from an AST.Identifier.
     * Flattens any characterization chain to a single string.
     */
    public CharacterizedIdentifier(AST.Identifier identifier) {
        this.id = identifier.id();
        this.characterization = identifier.canonicalCharacterization();
    }

    /**
     * Creates a CharacterizedIdentifier with explicit id and characterization strings.
     *
     * @param id The identifier name
     * @param characterization The characterization string (null or "" for no characterization)
     */
    public CharacterizedIdentifier(String id, String characterization) {
        this.id = id != null ? id : "";
        this.characterization = characterization != null ? characterization : "";
    }

    /**
     * Creates a CharacterizedIdentifier with no characterization.
     *
     * @param id The identifier name
     */
    public CharacterizedIdentifier(String id) {
        this(id, "");
    }

    /**
     * Gets the identifier name.
     */
    public String getId() {
        return id;
    }

    /**
     * Gets the characterization as a flattened string.
     * Returns empty string "" if no characterization.
     */
    public String getCharacterization() {
        return characterization;
    }

    /**
     * Checks if this identifier has a non-empty characterization.
     */
    public boolean hasCharacterization() {
        return characterization != null && !characterization.isEmpty();
    }

    /**
     * Returns the full characterized identifier string in Foolish syntax.
     * E.g., "type'x" for characterized, "x" for plain.
     */
    public String toFoolishString() {
        if (hasCharacterization()) {
            return characterization + "'" + id;
        }
        return id;
    }

    @Override
    public String toString() {
        return toFoolishString();
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (!(obj instanceof CharacterizedIdentifier other)) return false;
        return id.equals(other.id) && characterization.equals(other.characterization);
    }

    @Override
    public int hashCode() {
        return 31 * id.hashCode() + characterization.hashCode();
    }
}
