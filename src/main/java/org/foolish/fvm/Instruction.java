package org.foolish.fvm;

/**
 * An executable unit within the FVM.  Instructions are simply specialized
 * {@link Targoe} instances.
 */
public abstract class Instruction extends Targoe {
    protected Instruction(TargoeType type) {
        super(type);
    }
}

