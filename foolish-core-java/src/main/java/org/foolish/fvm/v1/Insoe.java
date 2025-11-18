package org.foolish.fvm.v1;

import org.foolish.ast.AST;

/**
 * INstruction Of Evaluation.  A simple wrapper around an AST node so that the
 * node can participate in the evaluation pipeline.  The wrapped AST sits at the
 * bottom of a {@link Firoe}'s progress heap.
 */
public record Insoe(AST ast) implements Targoe {

    /**
     * Returns the underlying AST node.
     */
    @Override
    public AST ast() {
        return ast;
    }

    /**
     * Ensures the underlying AST node is of the expected type and returns it.
     */
    public <T extends AST> T as(Class<T> type) {
        if (!type.isInstance(ast)) {
            throw new IllegalArgumentException(
                    "Expected AST node of type " + type.getSimpleName() +
                            " but found " + (ast == null ? "null" : ast.getClass().getSimpleName()));
        }
        return type.cast(ast);
    }

}
