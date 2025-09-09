package org.foolish.fvm;

/**
 * FINal Evaluation Result.  Represents a concluded value in the FVM.  A
 * {@code Finer} may either contain a concrete value or represent an unknown
 * result.
 */
public final class Finer implements Targoe {
    private final Object value; // null signifies unknown

    private Finer(Object value) {
        this.value = value;
    }

    /** Constant representing an unknown value. */
    public static final Finer UNKNOWN = new Finer(null);

    /** Creates a {@code Finer} holding the given long value. */
    public static Finer of(long value) {
        return new Finer(value);
    }

    /** Returns the underlying value, or {@code null} if unknown. */
    public Object value() {
        return value;
    }

    /**
     * @return {@code true} if this finer represents an unknown result.
     */
    public boolean isUnknown() {
        return this == UNKNOWN || value == null;
    }
}
