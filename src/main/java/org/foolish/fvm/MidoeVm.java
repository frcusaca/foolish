package org.foolish.fvm;

/**
 * Evaluation utilities for {@link Midoe} instances.
 */
public final class MidoeVm {
    private MidoeVm() {}

    public static Finear evaluate(Midoe midoe, Environment env) {
        if (midoe.progress_heap().isEmpty()) {
            return Finear.UNKNOWN;
        }
        Targoe top = midoe.progress_heap().get(midoe.progress_heap().size() - 1);
        if (top instanceof Finear finer) {
            if (midoe.progress_heap().size() == 1) {
                return finer;
            }
            top = midoe.progress_heap().get(0);
        }
        if (top instanceof Insoe insoe) {
            Finear result = insoe.execute(env);
            midoe.progress_heap().add(result);
            return result;
        }
        if (top instanceof Midoe m) {
            Finear result = evaluate(m, env);
            midoe.progress_heap().add(result);
            return result;
        }
        return Finear.UNKNOWN;
    }
}
