package org.foolish.fvm;

/**
 * Executes {@link Brane} instructions.
 */
public final class BraneVm {
    private BraneVm() {}

    public static Finear execute(Brane brane, Environment env) {
        Finear result = Finear.UNKNOWN;
        for (Midoe stmt : brane.statements()) {
            result = stmt.evaluate(env);
        }
        return result;
    }
}
