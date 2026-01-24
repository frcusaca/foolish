package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.LinkedList;
import java.util.List;
import java.util.stream.Stream;

/**
 * FIR with a braneMind queue for managing evaluation tasks.
 * The braneMind enables breadth-first execution of nested expressions.
 * <p>
 * This class centrally manages the braneMind queue operations:
 * - Enqueuing sub-FIRs that need evaluation
 * - Stepping through NYE (Not Yet Evaluated) FIRs
 * - Dequeuing FIRs when they complete evaluation
 * <p>
 * Derived classes must implement initialize() and should override step() and isNye() for their specific logic.
 */
public abstract class FiroeWithBraneMind extends FIR {
    protected final LinkedList<FIR> braneMind;
    protected final BraneMemory braneMemory;
    protected boolean ordinated;

    protected FiroeWithBraneMind(AST ast, String comment) {
        super(ast, comment);
        this.braneMind = new LinkedList<>();
        this.braneMemory = new BraneMemory(null);
        this.ordinated = false;
    }

    public void ordinateToParentBraneMind(FiroeWithBraneMind parent, int myPos) {
        assert !this.ordinated;
        this.braneMemory.setParent(parent.braneMemory);
        this.braneMemory.setMyPos(myPos);
        this.ordinated = true;
    }

    protected FiroeWithBraneMind(AST ast) {
        this(ast, null);
    }

    static FiroeWithBraneMind ofExpr(AST.Expr... tasks) {
        return of(List.of(tasks).stream().map(FIR::createFiroeFromExpr).toArray(FIR[]::new));
    }

    static FiroeWithBraneMind of(FIR... tasks) {
        FiroeWithBraneMind result = new FiroeWithBraneMind(null, null) {
            @Override
            protected void initialize() {
                setInitialized();
            }
        };
        for (FIR task : tasks) {
            result.enqueueFirs(task);
        }
        return result;
    }

    protected void enqueueSubfirOfExprs(AST.Expr... tasks) {
        enqueueFirs(ofExpr(tasks));
    }

    /**
     * Initialize this FIR by setting up its state and enqueuing sub-FIRs.
     * This method should be called once during the first step().
     * Implementations should call setInitialized(true) when complete.
     */
    protected abstract void initialize();

    /**
     * Enqueues a FIR into the braneMind if it's NYE (Not Yet Evaluated).
     * If the FIR is already evaluated, place it directly into braneMemory.
     */
    protected void enqueueFirs(FIR... firs) {
        for (FIR fir : firs) {
            braneMind.addLast(fir);
            braneMemory.put(fir);
            switch (fir) {
                case FiroeWithBraneMind fwbm:
                    fwbm.ordinateToParentBraneMind(this, braneMind.size() - 1);
                default:
                    ;
            }
        }
    }

    protected void enqueueExprs(AST.Expr... exprs) {
        for (AST.Expr expr : exprs)
            enqueueFirs(FIR.createFiroeFromExpr(expr));
    }

    /**
     * Checks if a FIR is a brane (BraneFiroe).
     */
    protected boolean isBrane(FIR fir) {
        return fir instanceof BraneFiroe;
    }

    /**
     * Steps the next FIR in the braneMind queue with state-aware brane handling.
     * <p>
     * State transitions:
     * - UNINITIALIZED → INITIALIZED: Initialize this FIR
     * - INITIALIZED → CHECKED: Step non-branes only until all are CHECKED (type/reference checking)
     * - CHECKED → EVALUATING: Immediate transition when detected
     * - EVALUATING → CONSTANT: Step everything (including branes) until all complete
     * <p>
     * Derived classes should override this method and call super.step() as needed.
     *
     * @return 1 for meaningful work, 0 for empty transitions
     */
    public int step() {
        switch (getNyes()) {
            case UNINITIALIZED -> {
                // First step: initialize this FIR
                initialize();
                setNyes(Nyes.INITIALIZED);
                return 1;
            }
            case INITIALIZED -> {
                // Step non-brane expressions until all are CHECKED
                if (stepNonBranesUntilState(Nyes.CHECKED)) {
                    // All non-branes have reached CHECKED (type/reference checking complete)
                    setNyes(Nyes.CHECKED);
                }
                return 1; // Did work stepping sub-expressions
            }
            case CHECKED -> {
                // Immediate transition to EVALUATING when step() is called
                setNyes(Nyes.EVALUATING);
                return 1;
            }
            case EVALUATING -> {
                // Step everything including sub-branes
                if (braneMind.isEmpty()) {
                    // All expressions evaluated, transition to CONSTANT
                    setNyes(Nyes.CONSTANT);
                    return 1;
                }

                FIR current = braneMind.removeFirst();
                try {
                    int work = current.step();

                    if (current.isNye()) {
                        braneMind.addLast(current);
                    }
                    return work;
                } catch (Exception e) {
                    braneMind.addFirst(current); // Re-enqueue on error
                    throw new RuntimeException("Error during braneMind step execution", e);
                    //TODO: Handle this exception Foolishly
                }
            }
            case CONSTANIC, CONSTANT -> {
                // Already evaluated, nothing to do
                return 0;
            }
        }
        return 0; // Should not reach here
    }

    /**
     * Steps non-brane FIRs until they reach the target state.
     * Branes are re-enqueued without stepping.
     *
     * @param targetState The state that non-branes should reach
     * @return true if all non-branes have reached the target state, false otherwise
     */
    private boolean stepNonBranesUntilState(Nyes targetState) {
        if (braneMind.isEmpty() || allNonBranesReachedState(targetState)) {
            return true;
        }

        FIR current = braneMind.removeFirst();

        //Reach here only when we have found the first non-brane sub-targetSetate member
        try {
            // Step non-brane expression
            current.step();

            // re-enqueue if still NYE
            if (current.isNye()) {
                braneMind.addLast(current);
            }
            // Check if it has reached the target state
            if (current.getNyes().ordinal() < targetState.ordinal()) {
                return false; // don't need to detect further, at least one non-brane is not at target state
            }

        } catch (Exception e) {
            braneMind.addFirst(current); // Re-enqueue on error
            throw new RuntimeException("Error during braneMind step execution", e);
            //TODO: Handle this exception Foolishly
        }

        // Check if all non-branes in the queue have reached target state
        return allNonBranesReachedState(targetState);
    }

    /**
     * Checks if all non-brane FIRs in the braneMind have reached at least the target state.
     * The furst sub-target-state non-brane member is shifted to the front of the queue.
     */
    private boolean allNonBranesReachedState(Nyes targetState) {
        if (braneMind.isEmpty()) {
            return true;
        }
        FIR current = braneMind.getFirst();
        int seen = 1;
        // Let's skip branes and the sub expressions that has already reached desired state.
        while (isBrane(current) || (current.getNyes().ordinal() >= targetState.ordinal())) {
            // Re-enqueue brane without stepping - keep its place in line
            if (seen++ > braneMind.size()) {
                return true;
            }
            braneMind.addLast(braneMind.removeFirst());
            current = braneMind.getFirst();
        }
        return false;
    }

    public Stream<FIR> stream() {
        return braneMemory.stream();
    }

}
