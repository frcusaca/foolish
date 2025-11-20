package org.foolish.fvm.v1;

/**
 * FINal EvAluation Result.  Pronounced "fine-ear" ("fah-ner" is acceptable).
 * Represents a concluded value in the FVM.  A {@code Finear} may either
 * contain a concrete value or represent a not knowable result.
 */
public final class Finear extends Firoe {
    /**
     * Constant representing a not knowable value.
     */
    public static final Finear NK = new Finear(null);
    private final Object value; // null signifies unknown

    private Finear(Object value) {
        super(null);
        progress_heap.add(this);
        this.value = value;
    }

    /**
     * Creates a {@code Finear} holding the given long value.
     */
    public static Finear of(long value) {
        return new Finear(value);
    }

    /**
     * Returns the underlying value, or {@code null} if unknown.
     */
    public Object value() {
        return value;
    }

    public long longValue() {
        if (isNK()) throw new IllegalStateException("Cannot get long value of not knowable Finear");
        return (Long) value;
    }

    /**
     * @return {@code true} if this finear represents a not knowable result.
     */
    public boolean isNK() {
        return this == NK || value == null;
    }

    public boolean equals(Object obj) {
        return obj != null && obj instanceof Finear f && (this.isNK() && f.isNK() || this.value.equals(f.value));
    }

    public int hashCode() {
        return isNK() ? 0 : value.hashCode();
    }

    public String toString() {
        return isNK() ? "NK" : value.toString();
    }
}
