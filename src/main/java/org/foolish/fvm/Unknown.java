package org.foolish.fvm;

/**
 * Sentinel value representing an unknown result in the FVM.
 */
public final class Unknown {
    public static final Unknown INSTANCE = new Unknown();

    private Unknown() {}

    @Override
    public String toString() {
        return "???";
    }
}
