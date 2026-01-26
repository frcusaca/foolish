package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;

/**
 * BraneFiroe represents a brane in the UBC system.
 * The BraneFiroe processes its AST to create Expression Firoes,
 * which are enqueued into the braneMind and evaluated breadth-first.
 */
public class BraneFiroe extends FiroeWithBraneMind {

    public BraneFiroe(AST ast) {
        super(ast);
    }

    /**
     * Initialize the BraneFiroe by converting AST statements to Expression Firoes.
     */
    @Override
    protected void initialize() {
        if (isInitialized()) return;
        setInitialized();

        if (ast instanceof AST.Brane brane) {
            for (AST.Expr expr : brane.statements()) {
                FIR firoe = createFiroeFromExpr(expr);
                enqueueFirs(firoe);
            }
        }else{
            throw new IllegalArgumentException("AST must be of type AST.Brane");
        }
    }

    @Override
    public boolean isNye() {
        return !isInitialized() || super.isNye();
    }

    @Override
    public void step() {
        if (!isInitialized()) {
            initialize();
            return;
        }

        super.step();
    }

    /**
     * Returns the list of expression Firoes in this brane.
     * Includes both completed (in braneMemory) and pending (in braneMind) FIRs.
     */
    public List<FIR> getExpressionFiroes() {
        List<FIR> allFiroes = new ArrayList<>();
        braneMemory.forEach(allFiroes::add);
        return allFiroes;
    }

    /**
     * Returns the brane's memory.
     */
    public BraneMemory getMemory() {
        return braneMemory;
    }

    @Override
    public String toString() {
        return new Sequencer4Human().sequence(this);
    }
}
