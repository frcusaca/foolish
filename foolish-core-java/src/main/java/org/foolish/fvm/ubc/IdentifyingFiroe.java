package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Minimal LHS-only identifier FIRoe.
 * <p>
 * - Holds the characterized identifier (CharacterizedIdentifier / Query.StrictlyMatchingQuery)
 * - Intended only for LHS placement (assignments, patterns)
 * - Refuses evaluation or cloning for RHS usage; RHS should be converted into backwards-search FIRs
 */
public class IdentifyingFiroe extends FiroeWithBraneMind implements Constanicable {
    private final Query.StrictlyMatchingQuery identifier;

    public IdentifyingFiroe(AST.Identifier identifier) {
        super(identifier);
        this.identifier = new Query.StrictlyMatchingQuery(identifier.id(), identifier.canonicalCharacterization());
    }

    public CharacterizedIdentifier getIdentifier() {
        return identifier;
    }

    public String getCharacterization() {
        return identifier.getCharacterization();
    }

    @Override
    public FIR getResult() {
        throw new UnsupportedOperationException("IdentifyingFiroe is LHS-only; RHS identifiers must be converted to backwards-search FIRs.");
    }

    @Override
    public boolean isConstanic() {
        // Treat LHS placeholders as constanic by default (no RHS resolution here)
        return true;
    }

    @Override
    protected void initialize() {
        setInitialized();
    }

    @Override
    public int step() {
        throw new UnsupportedOperationException("IdentifyingFiroe is LHS-only and should not be stepped/evaluated.");
    }

    @Override
    public long getValue() {
        throw new UnsupportedOperationException("IdentifyingFiroe is LHS-only and has no numeric value.");
    }

    @Override
    public String toString() {
        return ((AST.Identifier) ast).toString();
    }

    @Override
    protected FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes) {
        throw new UnsupportedOperationException("IdentifyingFiroe does not support cloning; it is LHS-only.");
    }
}