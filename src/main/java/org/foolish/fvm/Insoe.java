package org.foolish.fvm;

/**
 * INstruction Of Evaluation.  An {@code Insoe} performs some work within an
 * {@link Environment} and ultimately produces a {@link Finear}.  The interface
 * extends {@link Targoe} so that instructions can participate in the new
 * evaluation model where they are wrapped in {@link Midoe} instances.
 */
@FunctionalInterface
public interface Insoe extends Targoe {
    Finear execute(Environment env);
}
