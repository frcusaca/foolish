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
 */
public class IdentifierFiroe extends FiroeWithBraneMind {
    private final Query.StrictlyMatchingQuery identifier;
    FIR value = null; // Standard (stripped) value
    FIR raw = null;   // Raw value (for Constantic access)

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
     * An identifier is abstract if it hasn't been resolved yet or if its resolved value is abstract.
     */
    @Override
    public boolean isAbstract() {
        if (value == null) {
            return true; // Not yet resolved
        }
        return value.isAbstract();
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
            case INITIALIZED: {
                // Get raw value from memory
                raw = braneMemory.get(identifier, 0)
                        .map(p -> p.getValue())
                        .orElse(null);

                // Process standard value
                if (raw instanceof BraneFiroe brane) {
                    value = brane.clone(false); // Drop detachments
                } else {
                    value = raw;
                }

                setNyes(Nyes.RESOLVED);
            }
            default:
                super.step();

        }
    }

    @Override
    public long getValue() {
        if (value == null) throw new IllegalStateException("Identifier not resolved");
        return value.getValue();
    }

    @Override
    public String toString() {
        return ((AST.Identifier) ast).toString();
    }
}
