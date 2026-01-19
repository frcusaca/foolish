package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * SearchUpFiroe is a Brane wrapper for the search-up (↑) operation.
 * It has a reference to a brane Firoe that should already exist at this point.
 * The brane may have only AST initially.
 */
public class SearchUpFiroe extends FiroeWithoutBraneMind {
    private BraneFiroe referencedBrane;

    public SearchUpFiroe(AST.SearchUP searchUp) {
        super(searchUp);
        this.referencedBrane = null;
    }

    /**
     * Gets the referenced brane.
     */
    public BraneFiroe getReferencedBrane() {
        return referencedBrane;
    }

    /**
     * Sets the brane that this SearchUpFiroe references.
     * This should be called during initialization when the parent brane is known.
     */
    public void setReferencedBrane(BraneFiroe brane) {
        this.referencedBrane = brane;
    }

    /**
     * SearchUpFiroe is Constantic if the referenced brane is Constantic or not set.
     */
    @Override
    public boolean isConstantic() {
        if (referencedBrane == null) {
            return true;
        }
        return referencedBrane.isConstantic();
    }

    @Override
    public void step() {
        // Nothing to do
    }

    public String toString() {
        return "↑";
    }
}
