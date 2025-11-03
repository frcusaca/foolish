package org.foolish.ubc;

import org.foolish.ast.AST;

import java.util.ArrayDeque;
import java.util.Collection;
import java.util.LinkedList;
import java.util.Queue;

/**
 * FIR with a braneMind queue for managing evaluation tasks.
 * The braneMind enables breadth-first execution of nested expressions.
 * 
 * This class centrally manages the braneMind queue operations:
 * - Enqueuing sub-FIRs that need evaluation
 * - Stepping through underevaluated FIRs
 * - Dequeuing FIRs when they complete evaluation
 * 
 * Derived classes should override step() and underevaluated() for their specific logic.
 */
public class FiroeWithBraneMind extends FIR {
    protected final LinkedList<FIR> braneMind;
    protected final LinkedList<FIR> braneMemory;

    protected FiroeWithBraneMind(AST ast, String comment) {
        super(ast, comment);
        this.braneMind = new LinkedList<>();
        this.braneMemory = new LinkedList<>();
    }
    protected FiroeWithBraneMind(AST ast, String comment, Collection<FIR> tasks) {
        this(ast, comment);
        this.braneMind.addAll(tasks);
    }

    static FiroeWithBraneMind of(FIR... tasks) {
        FiroeWithBraneMind result = new FiroeWithBraneMind(null,null);
        for (FIR task : tasks) {
            result.enqueueFir(task);
        }
        return result;
    }
    protected void enqueueFirOf(FIR... tasks) {
        enqueueFir(of(tasks));
    }

    @Override
    public boolean isAbstract(){
        for (FIR fir : braneMind) {
            if (fir.isAbstract()) {
                return true;
            }
        }
        return false;
    }


    protected FiroeWithBraneMind(AST ast) {
        this(ast, null);
    }

    /**
     * Enqueues a FIR into the braneMind if it's underevaluated.
     */
    protected void enqueueFir(FIR fir) {
        if (fir.underevaluated()) {
            braneMind.addLast(fir);
        }
    }

    /**
     * A FiroeWithBraneMind is underevaluated if its braneMind queue is not empty.
     * Derived classes should override this method and call super.underevaluated() as needed.
     */
    @Override
    public boolean underevaluated() {
        return !braneMind.isEmpty();
    }

    /**
     * Steps the next FIR in the braneMind queue and dequeues it if no longer underevaluated.
     * This is the core braneMind management logic.
     * Derived classes should override this method and call super.step() as needed.
     */
    public void step() {
        if (braneMind.isEmpty()) {
            return;
        }
        FIR current = braneMind.removeFirst();
        try {
            if (current instanceof FiroeWithBraneMind firoeWithMind) {
                firoeWithMind.step();
            }

            if (current.underevaluated()) {
                braneMind.addLast(current);
            }else{
                braneMemory.addLast(current);
            }
        }catch (Exception e) {
            braneMind.addFirst(current); // Re-enqueue on error
            throw new RuntimeException("Error during braneMind step execution", e);
            //TODO: Handle this exception Foolishly
        }
    }
}
