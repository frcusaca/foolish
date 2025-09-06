package org.foolish.interpreter;

/** Runtime value produced by the interpreter. */
public sealed interface Value permits IntValue, BraneValue, UnknownValue {
    Value UNKNOWN = new UnknownValue();
}

/** Integer runtime value. */
record IntValue(long value) implements Value {
    @Override public String toString() { return Long.toString(value); }
}

/** Representation of a brane used as a value. */
record BraneValue(org.foolish.ast.AST.Brane brane) implements Value {
    @Override public String toString() {
        int n = brane.statements().size();
        return "<brane:" + n + (n == 1 ? " statement" : " statements") + ">";
    }
}

/** Unknown value placeholder. */
final class UnknownValue implements Value {
    @Override public String toString() { return "???"; }
}
