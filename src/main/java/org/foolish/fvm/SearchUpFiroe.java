package org.foolish.fvm;

import org.foolish.ast.AST;

/**
 * Firoe wrapper for SearchUP (`↑`) expression.
 *
 * According to the UBC (Unicellular Brane Computer) design, SearchUP provides
 * access to the parent brane context, enabling upward search in the Ancestral
 * Brane (AB) hierarchy.
 */
class SearchUpFiroe extends Firoe {
    private final Firoe parent;

    /**
     * Creates a SearchUpFiroe with a parent Firoe pointer.
     *
     * @param base The Insoe containing the AST.SearchUP node
     * @param parent The parent Firoe context, or null if this is at the topmost level
     */
    SearchUpFiroe(Insoe base, Firoe parent) {
        super(base);
        if (base != null) {
            base.as(AST.SearchUP.class);
        }
        this.parent = parent;
    }

    /**
     * Returns the parent Firoe context for upward search.
     * Returns null if this SearchUP is at the topmost program level.
     */
    public Firoe parent() {
        return parent;
    }

    public String toString() {
        return FormatterFactory.verbose().format(this);
    }
}
