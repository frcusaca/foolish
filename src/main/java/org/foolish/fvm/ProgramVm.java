package org.foolish.fvm;

/**
 * Executes {@link Program} instructions.
 */
public final class ProgramVm {
    private ProgramVm() {}

    public static Finear execute(Program program, Environment env) {
        return program.brane().execute(env);
    }
}
