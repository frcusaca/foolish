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
            // Already computed (non-null result)
            return;
        }

        if (atConstantic()) {
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
        if (isNye()) {
            return;
        }

        // Expression is fully evaluated (or stuck at Constantic), store the result
        if (!braneMemory.isEmpty()) {
            result = braneMemory.get(0);
            if (result.atConstantic()) {
                setNyes(Nyes.CONSTANTIC);
            }
        }
        // If result is null (e.g. BinaryFiroe returned null result), that's fine.
        // It stays Constantic.
    }

    @override
    public boolean isconstantic() {
        if (result != null) {
            return result.isconstantic();
        }
        // if no result but state is constant -> it's constantic
        if (getnyes() == nyes.constant) return true;

        return super.isconstantic();
    }

    /**
     * gets the coordinate name for this assignment (without characterization).
     * for compatibility with existing code.
     */
    public string getid() {
        return lhs.getid();
    }

    /**
     * gets the lhs characterized identifier.
     */
    public characterizedidentifier getlhs() {
        return lhs;
    }

    /**
     * gets the evaluated result fir.
     * returns null if not yet evaluated.
     */
    public fir getresult() {
        return result;
    }

    @override
    public long getvalue() {
        if (atconstantic()) {
            throw new illegalstateexception("assignmentfiroe is constantic");
        }
        if (result == null) {
            if (getnyes() == nyes.constant) {
                throw new illegalstateexception("assignmentfiroe evaluated to constantic (unresolved)");
            }
            throw new illegalstateexception("assignmentfiroe not fully evaluated");
        }
        return result.getvalue();
    }

    @override
    public string tostring() {
        return new sequencer4human().sequence(this);
    }
}
