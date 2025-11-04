package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * IdentifierFiroe represents a characterizable identifier reference in the UBC system.
 *
 * In FIR, identifiers are always represented as strings:
 * - The identifier name: a String
 * - The characterization: a String (flattened from any chain in AST)
 *
 * Examples:
 * - Simple identifier: `x` → id="x", characterization=""
 * - Characterized: `type'x` → id="x", characterization="type"
 * - Chained: `outer'inner'x` → id="x", characterization="outer'inner"
 *
 * The characterization string is used when resolving identifiers to:
 * - Disambiguate between multiple bindings of the same name
 * - Type checking (future)
 * - Scope resolution (future)
 *
 * Currently, identifier lookup is not yet implemented in UBC, so this
 * returns NK (not-known) values.
 */
public class IdentifierFiroe extends FiroeWithoutBraneMind {
    private final String id;
    private final String characterization;

    public IdentifierFiroe(AST.Identifier identifier) {
        super(identifier);
        this.id = identifier.id();
        // Flatten the characterization chain to a string
        this.characterization = identifier.canonicalCharacterization();
    }

    /**
     * Creates an IdentifierFiroe with explicit id and characterization strings.
     * Used when converting from other representations.
     */
    public IdentifierFiroe(String id, String characterization) {
        super(null);
        this.id = id;
        this.characterization = characterization != null ? characterization : "";
    }

    /**
     * Gets the identifier name as a string.
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
     * Checks if this identifier has a characterization.
     */
    public boolean hasCharacterization() {
        return characterization != null && !characterization.isEmpty();
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
