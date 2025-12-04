package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.fvm.ubc.BraneMemory.StrictlyMatchingQuery;
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
    private final StrictlyMatchingQuery identifier;
    private FIR value = null;

    public IdentifierFiroe(AST.Identifier identifier) {
        super(identifier);
        this.identifier = new StrictlyMatchingQuery(identifier.id(), identifier.canonicalCharacterization());
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
                value = braneMemory.get(identifier, 0)
                        .map(r -> r.getValue())
                        .orElse(null);
                setNyes(Nyes.RESOLVED);
            }
            default:
                super.step();

        }
    }
    /**
     * Identifier lookup is not yet implemented.
     * @throws UnsupportedOperationException always
     */
    @Override
    public long getValue() {
        return value.getValue();
    }

    @Override
    public String toString() {
        return ((AST.Identifier) ast).toString();
    }
}
