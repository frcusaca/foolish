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
                enqueueFir(firoe);
            }
        }
    }

    @Override
    public boolean underevaluated() {
        return !initialized || super.underevaluated();
    }

    @Override
    public void step() {
        if (!initialized) {
            initialize();
            return;
        }

        super.step();
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
