package org.foolish.fvm;

/**
 * Utilities for constructing {@link Midoe} trees from arbitrary {@link Targoe}
 * instances.  Each resulting {@code Midoe} has its originating {@link Targoe}
 * at the bottom of its progress heap.
 */
public final class MidoeVm {
    private MidoeVm() {}

    /** Wraps the given target in a {@link Midoe}, attaching it to the progress heap. */
    public static Midoe wrap(Targoe targoe) {
        if (targoe == null) return new Midoe();
        if (targoe instanceof Midoe m) return m;
        if (targoe instanceof Insoe i) return new Midoe(i);
        Midoe m = new Midoe();
        m.progress_heap().add(targoe);
        return m;
    }
}
