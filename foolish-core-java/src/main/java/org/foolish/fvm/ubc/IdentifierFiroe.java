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
    FiroeState state = new FiroeState.Unknown(); // Package-private for access by RegexpSearchFiroe

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
        return switch (state) {
            case FiroeState.Value(FIR fir) -> fir.isAbstract();
            case FiroeState.Unknown(), FiroeState.Constantic() -> true;
        };
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
                var found = braneMemory.get(identifier, 0)
                        .map(r -> r.getValue())
                        .orElse(null);
                if (found != null) {
                    state = new FiroeState.Value(found);
                } else {
                    state = new FiroeState.Constantic();
                }
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
        return switch (state) {
            case FiroeState.Value(FIR fir) -> fir.getValue();
            case FiroeState.Unknown(), FiroeState.Constantic() -> throw new UnsupportedOperationException("Cannot get value from identifier in state " + state);
        };
    }

    @Override
    public String toString() {
        return ((AST.Identifier) ast).toString();
    }
}
