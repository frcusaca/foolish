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
    private FIR result;

    public AssignmentFiroe(AST.Assignment assignment) {
        super(assignment);
        this.lhs = new CharacterizedIdentifier(assignment.identifier());
        this.result = null;
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
        if (result != null) {
            // Already computed
            return;
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
        braneMemory.get(0).ifPresent(fir -> {
            result = fir;
        });
    }

    @Override
    public boolean isNye() {
        return result == null;
    }

    @Override
    public boolean isAbstract() {
        /** check of the ID is abstract **/
        if (result != null) {
            return result.isAbstract();
        }
        return super.isAbstract();
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
     * Returns null if not yet evaluated.
     */
    public FIR getResult() {
        return result;
    }

    @Override
    public long getValue() {
        if (result == null) {
            throw new IllegalStateException("AssignmentFiroe not fully evaluated");
        }
        return result.getValue();
    }

    @Override
    public String toString() {
        return new Sequencer4Human().sequence(this);
    }
}
