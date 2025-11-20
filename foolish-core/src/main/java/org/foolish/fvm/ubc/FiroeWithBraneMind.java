package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedList;
import java.util.List;

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
    protected final ArrayList<FIR> braneMemory;

    protected FiroeWithBraneMind(AST ast, String comment) {
        super(ast, comment);
        this.braneMind = new LinkedList<>();
        this.braneMemory = new ArrayList<>();
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


    @Override
    public boolean isAbstract() {
        for (FIR fir : braneMind) {
            if (fir.isAbstract()) {
                return true;
            }
        }
        for (FIR fir : braneMemory) {
            if (fir.isAbstract()) {
                return true;
            }
        }
        return false;
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
            braneMemory.addLast(fir);
        }
    }
    protected void enqueueExprs(AST.Expr... exprs) {
        for (AST.Expr expr : exprs)
            enqueueFirs(FIR.createFiroeFromExpr(expr));
    }

    /**
     * A FiroeWithBraneMind is NYE (Not Yet Evaluated) if its braneMind queue is not empty.
     * Derived classes should override this method and call super.isNye() as needed.
     */
    @Override
    public boolean isNye() {
        return !braneMind.isEmpty();
    }

    /**
     * Steps the next FIR in the braneMind queue and dequeues it if no longer NYE (Not Yet Evaluated).
     * This is the core braneMind management logic.
     * Derived classes should override this method and call super.step() as needed.
     */
    public void step() {
        if (!isNye()) {
            return;
        }
        FIR current = braneMind.removeFirst();
        try {
            current.step();

            if (current.isNye()) {
                braneMind.addLast(current);
            }
        } catch (Exception e) {
            braneMind.addFirst(current); // Re-enqueue on error
            throw new RuntimeException("Error during braneMind step execution", e);
            //TODO: Handle this exception Foolishly
        }
    }
}
