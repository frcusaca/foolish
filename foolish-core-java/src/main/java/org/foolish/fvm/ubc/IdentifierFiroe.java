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
    public int step() {
        switch (getNyes()) {
            case UNINITIALIZED -> {
                initialize();
                setNyes(Nyes.INITIALIZED);
                return 1;
            }
            case INITIALIZED -> {
                var found = braneMemory.get(identifier, 0);
                if (found.isEmpty()) {
                    setNyes(Nyes.CONSTANIC);
                    return 1;
                }
                value = found
                        .map(r -> r.getRight())
                        .orElse(null);
                if (value == null) {
                    setNyes(Nyes.CONSTANIC);
                } else {
                    setNyes(Nyes.CHECKED);
                }
                return 1;
            }
            case CHECKED -> {
                // Identifier lookup is complete, check if value is constanic
                if (value.isConstanic()) {
                    setNyes(Nyes.CONSTANIC);
                } else if (value.isConstant()) {
                    setNyes(Nyes.CONSTANT);
                } else {
                    // Value is still evaluating - shouldn't happen for identifiers
                    // since we just store a reference, but handle gracefully
                    setNyes(Nyes.EVALUATING);
                }
                return 1;
            }
            case EVALUATING -> {
                // Check if value completed evaluation
                if (value.isConstanic()) {
                    setNyes(Nyes.CONSTANIC);
                } else if (value.isConstant()) {
                    setNyes(Nyes.CONSTANT);
                }
                return 1;
            }
            case CONSTANIC, CONSTANT -> {
                return 0;
            }
            default -> {
                return super.step();
            }
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
