package org.foolish.fvm;

import org.foolish.ast.AST;

/**
 * Midoe wrapper for an identifier expression.
 */
class IdentifierMidoe extends Midoe {
    private final Characterizable id;

    IdentifierMidoe(String id) {
        super(null);
        this.id = new Characterizable(id);
    }
        IdentifierMidoe(Insoe base) {
        super(base);
        AST.Identifier ast = base.as(AST.Identifier.class);
        this.id = Characterizable.fromAst(ast);
    }

    public Characterizable id() {
        return id;
    }

    public String toString() {
        String value = "MidoeId(";
        if (progress_heap.size() > 1) {
            // Return a versioned id if in a function context
            value = "@" + progress_heap.getLast();
        }
        return id.toString() + value + ")";

    }
}
