package org.foolish.fvm;

/**
 * Simple printer that renders the most evaluated result of a targoe.  For now
 * this printer only renders the last evaluation result using the {@code toString}
 * of the resulting {@link Resoe}.  Unknown results therefore appear as
 * {@code ???}.
 */
public final class TargoePrinter {
    private TargoePrinter() {}

    public static String print(Targoe t, Environment env) {
        EvalResult er = t.execute(env);
        return er.value().toString();
    }
}

