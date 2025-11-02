package org.foolish.ubc;

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
     * Sets the brane that this SearchUpFiroe references.
     * This should be called during initialization when the parent brane is known.
     */
    public void setReferencedBrane(BraneFiroe brane) {
        this.referencedBrane = brane;
    }

    /**
     * Gets the referenced brane.
     */
    public BraneFiroe getReferencedBrane() {
        return referencedBrane;
    }

    /**
     * SearchUpFiroe is abstract if the referenced brane is abstract or not set.
     */
    @Override
    public boolean isAbstract() {
        if (referencedBrane == null) {
            return true;
        }
        return referencedBrane.isAbstract();
    }

    @Override
    public String toString() {
        return "↑";
    }
}
