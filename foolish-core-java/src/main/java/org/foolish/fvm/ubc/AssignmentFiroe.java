package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * FIR for assignment expressions.
 * An assignment evaluates its right-hand side expression and stores the result
 * with a coordinate name in the brane's environment.
 * <p>
 * The LHS uses CharacterizedIdentifier to support characterized identifiers for
 * proper resolution and type checking. The RHS is evaluated as a FIR, and any
 * identifiers within the RHS expression tree will be represented as IdentifierFiroe
 * which internally uses CharacterizedIdentifier.
 */
public class AssignmentFiroe extends FiroeWithBraneMind {
    private final CharacterizedIdentifier lhs;
    private FiroeState result = new FiroeState.Unknown();

    public AssignmentFiroe(AST.Assignment assignment) {
        super(assignment);
        this.lhs = new CharacterizedIdentifier(assignment.identifier());
    }

    @Override
    protected void initialize() {
        if (isInitialized()) return;
        setInitialized();

        AST.Assignment assignment = (AST.Assignment) ast;
        // Enqueue the expression to be evaluated
        enqueueExprs(assignment.expr());
    }

    @Override
    public void step() {
        switch (result) {
            case FiroeState.Value _ -> { return; }
            default -> {}
        }

        if (!isInitialized()) {
            // Initialize and enqueue the expression
            initialize();
            return;
        }

        // Let the parent class handle braneMind stepping
        super.step();

        // Check if we can get the final result
        if (super.isNye()) {
            return;
        }

        // Expression is fully evaluated, store the result
        if (!braneMemory.isEmpty()) {
            result = new FiroeState.Value(braneMemory.get(0));
        } else {
            // Should usually not happen if expression was valid, but maybe for unit
            // or empty brane? Or maybe it's just unknown.
            // If braneMemory is empty after evaluation, we assume it produced nothing valid
            // or it's a void operation.
            // Keeping as Unknown if nothing produced, or maybe Constantic if it failed?
            // Existing logic had result=null which meant NYE, but here we are done stepping.
            // If we are done stepping but result is null, it means isNye was false.
            // So we need a state that says "Done but no value".
            // For now, let's assume if it finished, it has a value or it stays Unknown/Constantic?
            // If braneMemory is empty, let's leave it as Unknown for now unless we want Constantic.
        }
    }

    @Override
    public boolean isNye() {
        return switch (result) {
            case FiroeState.Value _ -> false;
            default -> true;
        };
    }

    @Override
    public boolean isAbstract() {
        /** check of the ID is abstract **/
        return switch (result) {
            case FiroeState.Value(FIR fir) -> fir.isAbstract();
            default -> super.isAbstract();
        };
    }

    /**
     * Gets the coordinate name for this assignment (without characterization).
     * For compatibility with existing code.
     */
    public String getId() {
        return lhs.getId();
    }

    /**
     * Gets the LHS characterized identifier.
     */
    public CharacterizedIdentifier getLhs() {
        return lhs;
    }

    /**
     * Gets the evaluated result FIR.
     * Returns null if not yet evaluated or result is not a Value.
     * Note: This method signature is kept compatible where possible, returning FIR.
     * If the state is not Value, it returns null, mimicking previous behavior.
     * Use getFiroeState() for full state access.
     */
    public FIR getResult() {
        return switch (result) {
            case FiroeState.Value(FIR fir) -> fir;
            default -> null;
        };
    }

    public FiroeState getFiroeState() {
        return result;
    }

    @Override
    public long getValue() {
        return switch (result) {
            case FiroeState.Value(FIR fir) -> fir.getValue();
            default -> throw new IllegalStateException("AssignmentFiroe not fully evaluated or constantic");
        };
    }

    @Override
    public String toString() {
        return new Sequencer4Human().sequence(this);
    }
}
