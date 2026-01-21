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
public class IdentifierFiroe extends FiroeWithBraneMind {
    private final Query.StrictlyMatchingQuery identifier;
    FIR value = null; // Package-private for access by RegexpSearchFiroe

    public IdentifierFiroe(AST.Identifier identifier) {
        super(identifier);
        this.identifier = new Query.StrictlyMatchingQuery(identifier.id(), identifier.canonicalCharacterization());
    }

    /**
     * Gets the CharacterizedIdentifier.
     */
    public CharacterizedIdentifier getIdentifier() {
        return identifier;
    }

    /**
     * Gets the characterization as a flattened string.
     * Returns empty string "" if no characterization.
     */
    public String getCharacterization() {
        return identifier.getCharacterization();
    }

    /**
     * An identifier is Constanic if it hasn't been resolved yet or if its resolved value is Constanic.
     */
    @Override
    public boolean isConstanic() {
        if (value == null) {
            return true; // Not yet resolved or missing
        }
        return value.isConstanic();
    }

    @Override
    protected void initialize() {
        setInitialized();
    }

    /**
     * Implement the step method only overriding resolution phase, during resolution
     * we use the branemind to find the value of the identifier and store a reference to
     * it for `getValue()`
     */
    @Override
    public void step() {
        switch (getNyes()) {
            case INITIALIZED -> {
                var found = braneMemory.get(identifier, 0);
                if (found.isEmpty()) {
                    setNyes(Nyes.CONSTANIC);
                    return;
                }
                value = found
                        .map(r -> r.getRight())
                        .orElse(null);
                if (value == null) {
                    setNyes(Nyes.CONSTANIC);
                } else {
                    setNyes(Nyes.RESOLVED);
                }
            }
            default -> super.step();
        }
    }
    /**
     * Identifier lookup is not yet implemented.
     * @throws UnsupportedOperationException always
     */
    @Override
    public long getValue() {
        if (value == null) {
            if (atConstanic()) throw new IllegalStateException("Identifier is Constanic (missing): " + identifier.getId());
            throw new IllegalStateException("Identifier not resolved or not found: " + identifier.getId());
        }
        return value.getValue();
    }

    @Override
    public String toString() {
        return ((AST.Identifier) ast).toString();
    }
}
