package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * FIR for assignment expressions.
 * An assignment evaluates its right-hand side expression and stores the result
 * with a coordinate name in the brane's environment.
 */
public class AssignmentFiroe extends FiroeWithBraneMind {
    private final String id;
    private FIR result;

    public AssignmentFiroe(AST.Assignment assignment) {
        super(assignment);
        this.id = assignment.id();
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
        if (!braneMemory.isEmpty()) {
            result = braneMemory.get(0);
        }
    }

    @Override
    public boolean isNye() {
        return result == null;
    }

    @Override
    public boolean isAbstract() {
        if (result != null) {
            return result.isAbstract();
        }
        return super.isAbstract();
    }

    /**
     * Gets the coordinate name for this assignment.
     */
    public String getId() {
        return id;
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
