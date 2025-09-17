package org.foolish.fvm;

/**
 * Sentinel value representing an unknown result in the FVM.
 */
public final class Unknown extends Finer implements Insoe {
    public static final Unknown INSTANCE = new Unknown();

    private Unknown() {}

    @Override
    public Object value() { return null; }

    @Override
    public Targoe execute(Environment env) { return this; }

    @Override
    public String toString() {
        return "???";
    }
}

