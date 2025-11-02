package org.foolish.ubc;

import org.foolish.ast.AST;

import java.util.ArrayDeque;
import java.util.Queue;

/**
 * FIR with a braneMind queue for managing evaluation tasks.
 * The braneMind enables breadth-first execution of nested expressions.
 */
public abstract class FiroeWithBraneMind extends FIR {
    protected final Queue<FIR> braneMind;

    protected FiroeWithBraneMind(AST ast, String comment) {
        super(ast, comment);
        this.braneMind = new ArrayDeque<>();
    }

    protected FiroeWithBraneMind(AST ast) {
        this(ast, null);
    }

    /**
     * Returns the braneMind queue for managing evaluation tasks.
     */
    public Queue<FIR> braneMind() {
        return braneMind;
    }

    /**
     * A FiroeWithBraneMind is underevaluated if its braneMind queue is not empty.
     */
    @Override
    public boolean underevaluated() {
        return !braneMind.isEmpty();
    }

    /**
     * Takes a single evaluation step on this Firoe.
     * Finds the first underevaluated Firoe in the braneMind and steps it.
     * Dequeues the Firoe if it's no longer underevaluated after the step.
     */
    public abstract void step();
}
