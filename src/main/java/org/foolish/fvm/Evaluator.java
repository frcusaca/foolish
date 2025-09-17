package org.foolish.fvm;

/**
 * Utility to evaluate a {@link Targoe} to a {@link Finer} result.
 */
public final class Evaluator {
    private Evaluator() {}

    public static Finer eval(Targoe t, Environment env) {
        if (t instanceof Finer f) return f;
        if (t instanceof Midoe m) {
            if (m.isFinal()) {
                return m.finalResult();
            }
            return m.evaluate(env);
        }
        if (t instanceof Insoe i) return eval(i.execute(env), env);
        return Unknown.INSTANCE;
    }
}
