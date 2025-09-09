package org.foolish.fvm;

/**
 * Executes {@link IdentifierExpr} instructions.
 */
public final class IdentifierExprVm {
    private IdentifierExprVm() {}

    public static Finear execute(IdentifierExpr expr, Environment env) {
        return env.lookup(expr.id());
    }
}
