package org.foolish.fvm;

/**
 * Represents a unit of executable code within the FVM.  Each instruction
 * operates within an {@link Environment} and returns a result which may be
 * {@code null}.
 */
@FunctionalInterface
public interface Instruction {
    Object execute(Environment env);
}
