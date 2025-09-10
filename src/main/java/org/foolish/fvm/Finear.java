package org.foolish.fvm;

/**
 * FINal EvAluation Result.  Pronounced "fine-ear" ("fah-ner" is acceptable).
 * Represents a concluded value in the FVM.  A {@code Finear} may either
 * contain a concrete value or represent an unknown result.
 */
public final class Finear implements Targoe {
    private final Object value; // null signifies unknown

    private Finear(Object value) {
        this.value = value;
    }

    /** Constant representing an unknown value. */
    public static final Finear UNKNOWN = new Finear(null);

    /** Creates a {@code Finear} holding the given long value. */
    public static Finear of(long value) {
        return new Finear(value);
    }

    /** Returns the underlying value, or {@code null} if unknown. */
    public Object value() {
        return value;
    }

    /**
     * @return {@code true} if this finear represents an unknown result.
     */
    public boolean isUnknown() {
        return this == UNKNOWN || value == null;
    }

    public Finear execute(Environment env) {
        return this;
    }
}
