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

public class IdentifierFiroe extends FiroeWithBraneMind implements Constanicable {
    private final Query.StrictlyMatchingQuery identifier;
    FIR value = null; // Package-private for access by RegexpSearchFiroe

    public IdentifierFiroe(AST.Identifier identifier) {
        super(identifier);
        this.identifier = new Query.StrictlyMatchingQuery(identifier.id(), identifier.canonicalCharacterization());
    }

    /**
     * Copy constructor for cloneConstanic.
     */
    protected IdentifierFiroe(IdentifierFiroe original, FIR newParent) {
        super(original, newParent);
        this.identifier = original.identifier;  // Query is immutable, can share
        this.value = null;  // Reset value for re-resolution in new context
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
     * Gets the resolved FIR value for this identifier.
     * Returns null if not yet resolved or if resolution failed.
     */
    public FIR getResolvedFir() {
        return value;
    }

    @Override
    public FIR getResult() {
        return value;
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
        System.err.println("DEBUG STEP: Identifier " + identifier.getId() + " step() at state " + getNyes());
        switch (getNyes()) {
            case UNINITIALIZED -> {
                initialize();
                setNyes(Nyes.INITIALIZED);
                return 1;
            }
            case INITIALIZED -> {
                var found = memoryGet(identifier, 0);
                if (found.isEmpty()) {
                    System.err.println("DEBUG: Identifier " + identifier.getId() + " NOT FOUND");
                    setNyes(Nyes.CONSTANIC);
                    return 1;
                }
                value = found
                        .map(r -> r.getRight())
                        .orElse(null);
                if (value == null) {
                    setNyes(Nyes.CONSTANIC);
                } else {
                    System.err.println("DEBUG: Identifier " + identifier.getId() + " FOUND, transitioning to CHECKED");
                    setNyes(Nyes.CHECKED);
                }
                return 1;
            }
            case CHECKED -> {
                // Identifier lookup is complete, check if value has reached final state
                // Use atConstanic() to check for exactly CONSTANIC (unresolved)
                // Use atConstant() to check for exactly CONSTANT (fully resolved)
                System.err.println("DEBUG: Identifier " + identifier.getId() + " CHECKED value=" +
                    value.getClass().getSimpleName() + " atConstanic=" + value.atConstanic() +
                    " atConstant=" + value.atConstant() + " nyes=" + value.getNyes() + " isNye=" + value.isNye());
                if (value.atConstanic()) {
                    // CONSTANIC value needs to be coordinated (cloned and re-positioned)
                    // so it can re-resolve from this identifier's position.
                    System.err.println("DEBUG: Identifier " + identifier.getId() + " coordinating CONSTANIC value " + value.getClass().getSimpleName());
                    // Clone the value with INITIALIZED state so it will re-evaluate.
                    value = value.cloneConstanic(this, java.util.Optional.of(Nyes.INITIALIZED));
                    // storeFirs will ordinate the clone to this identifier's context
                    storeFirs(value);
                    setNyes(Nyes.PRIMED);
                } else if (value.atConstant()) {
                    setNyes(Nyes.CONSTANT);
                } else {
                    // Value is still evaluating - shouldn't happen for identifiers
                    // since we just store a reference, but handle gracefully
                    setNyes(Nyes.EVALUATING);
                }
                return 1;
            }
            case PRIMED -> {
                // Transition to EVALUATING to step the coordinated value
                setNyes(Nyes.EVALUATING);
                return 1;
            }
            case EVALUATING -> {
                // Step the coordinated value through evaluation
                if (isBraneEmpty()) {
                    // Value finished evaluating, check final state
                    if (value.atConstanic()) {
                        setNyes(Nyes.CONSTANIC);
                    } else {
                        setNyes(Nyes.CONSTANT);
                    }
                    return 1;
                }
                // Step the next FIR in braneMind
                FIR current = braneDequeue();
                current.step();
                if (current.isNye()) {
                    braneEnqueue(current);
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

    @Override
    protected FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes) {
        if (!isConstanic()) {
            throw new IllegalStateException(
                formatErrorMessage("cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, " +
                                  "but this FIR is in state: " + getNyes()));
        }

        if (isConstant()) {
            return this;  // Share CONSTANT identifiers completely
        }

        // CONSTANIC: use copy constructor
        IdentifierFiroe copy = new IdentifierFiroe(this, newParent);

        // Set target state if specified, otherwise copy from original
        if (targetNyes.isPresent()) {
            copy.nyes = targetNyes.get();
        } else {
            copy.nyes = this.nyes;
        }

        return copy;
    }
}
