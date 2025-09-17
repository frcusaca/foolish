package org.foolish.fvm;

/**
 * INstruction Of Evaluation - executable unit within the FVM.
 */
@FunctionalInterface
public interface Insoe extends Targoe {
    Targoe execute(Environment env);
}

