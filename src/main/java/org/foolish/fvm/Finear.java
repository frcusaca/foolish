package org.foolish.fvm;

/**
 * FINal EvAluation Result.  Pronounced "fine-ear" ("fah-ner" is acceptable).
 * Represents a concluded value in the FVM.  A {@code Finear} may either
 * contain a concrete value or represent an unknown result.
 */
public final class Finear extends Midoe {
    private final Object value; // null signifies unknown

    private Finear(Object value) {
        super(null);
        progress_heap.add(this);
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
    public long longValue() {
        if (isUnknown()) throw new IllegalStateException("Cannot get long value of unknown Finear");
        return (Long)value;
    }

    /**
     * @return {@code true} if this finear represents an unknown result.
     */
    public boolean isUnknown() {
        return this == UNKNOWN || value == null;
    }

    public boolean equals(Object obj) {
        return obj!=null && obj instanceof Finear f && (this.isUnknown() && f.isUnknown() || this.value.equals(f.value));
    }
    public int hashCode() {
        return isUnknown() ? 0 : value.hashCode();
    }
    public String toString() {
        return isUnknown() ? "UNKNOWN" : value.toString();
    }
}
