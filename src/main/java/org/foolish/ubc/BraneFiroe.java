package org.foolish.ubc;

import org.foolish.ast.AST;
import org.foolish.fvm.Env;

import java.util.ArrayList;
import java.util.List;

/**
 * BraneFiroe represents a brane in the UBC system.
 * The BraneFiroe processes its AST to create Expression Firoes,
 * which are enqueued into the braneMind and evaluated breadth-first.
 */
public class BraneFiroe extends FiroeWithBraneMind {
    private final List<FIR> expressionFiroes;
    private Env environment;
    private boolean initialized;

    public BraneFiroe(AST ast) {
        super(ast);
        this.expressionFiroes = new ArrayList<>();
        this.environment = null;
        this.initialized = false;
    }

    /**
     * Initialize the BraneFiroe by converting AST statements to Expression Firoes.
     */
    private void initialize() {
        if (initialized) return;
        initialized = true;

        if (ast instanceof AST.Brane brane) {
            for (AST.Expr expr : brane.statements()) {
                FIR firoe = createFiroeFromExpr(expr);
                expressionFiroes.add(firoe);
                if (firoe.underevaluated()) {
                    braneMind.offer(firoe);
                }
            }
        }
    }

    /**
     * Creates a FIR from an AST expression.
     */
    private FIR createFiroeFromExpr(AST.Expr expr) {
        if (expr instanceof AST.IntegerLiteral literal) {
            return new ValueFiroe(expr, literal.value());
        } else if (expr instanceof AST.BinaryExpr binary) {
            return new BinaryFiroe(binary);
        } else if (expr instanceof AST.UnaryExpr unary) {
            return new UnaryFiroe(unary);
        } else if (expr instanceof AST.IfExpr ifExpr) {
            return new IfFiroe(ifExpr);
        } else if (expr instanceof AST.SearchUP searchUp) {
            return new SearchUpFiroe(searchUp);
        } else if (expr instanceof AST.Brane brane) {
            return new BraneFiroe(brane);
        } else {
            // For unsupported expressions, create a generic FIR
            return new UnsupportedFiroe(expr);
        }
    }

    @Override
    public boolean underevaluated() {
        // BraneFiroe is underevaluated until it's initialized and all items in braneMind are processed
        return !initialized || !braneMind.isEmpty();
    }

    @Override
    public void step() {
        if (!initialized) {
            initialize();
            return;
        }

        if (braneMind.isEmpty()) {
            return;
        }

        FIR current = braneMind.peek();
        if (current instanceof FiroeWithBraneMind firoeWithMind) {
            firoeWithMind.step();
            if (!current.underevaluated()) {
                braneMind.poll();
            }
        } else {
            // Should not happen as FiroeWithoutBraneMind shouldn't be in queue
            braneMind.poll();
        }
    }

    /**
     * Returns the frozen environment after full evaluation.
     * This is the value of a fully evaluated BraneFiroe.
     */
    @Override
    public Env getEnvironment() {
        if (!isComplete()) {
            throw new IllegalStateException("BraneFiroe not fully evaluated");
        }
        return environment;
    }

    /**
     * Returns true if this brane has completed evaluation.
     */
    private boolean isComplete() {
        return initialized && !underevaluated();
    }

    /**
     * Returns the list of expression Firoes in this brane.
     */
    public List<FIR> getExpressionFiroes() {
        return expressionFiroes;
    }

    @Override
    public boolean isAbstract() {
        // A brane is abstract if any of its expressions are abstract
        for (FIR firoe : expressionFiroes) {
            if (firoe.isAbstract()) {
                return true;
            }
        }
        return false;
    }

    @Override
    public String toString() {
        return new Sequencer4Human().sequence(this);
    }

    /**
     * Placeholder for unsupported expression types.
     */
    private static class UnsupportedFiroe extends FiroeWithoutBraneMind {
        UnsupportedFiroe(AST ast) {
            super(ast);
        }

        @Override
        public boolean isAbstract() {
            return true;
        }
    }
}
