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
        // Register this brane as the owner of its memory
        this.braneMemory.setOwningBrane(this);
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

    // Removed isNye override

    @Override
    public int step() {
        if (!isInitialized()) {
            initialize();
            return 1;
        }

        return super.step();
    }

    /**
     * Executes the given search firoe in the context of this brane.
     * The search runs as if it were appended to the end of the brane,
     * without modifying the brane itself.
     *
     * @param searchFiroe the search expression to evaluate
     * @return the result of the search (FIR)
     */
    public FIR search(AbstractSearchFiroe searchFiroe) {
        // Link the search firoe to this brane's memory (as parent)
        // Position -1 indicates it's floating/appended
        searchFiroe.ordinateToParentBraneMind(this, -1);

        // Run the search to completion
        while (searchFiroe.isNye()) {
            searchFiroe.step();

            // If the search firoe is initialized but has an NKFiroe anchor (from ???),
            // substitute 'this' brane as the anchor.
            if (searchFiroe.isInitialized() && !searchFiroe.braneMemory.isEmpty()) {
                FIR anchor = searchFiroe.braneMemory.getLast();
                if (anchor instanceof NKFiroe) {
                    // We inject 'this' as a correction for implicit anchor
                    searchFiroe.braneMemory.put(this);
                }
            }
        }

        // Return the result
        return searchFiroe.getResult();
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

    @Override
    public String toString() {
        return new Sequencer4Human().sequence(this);
    }
}
