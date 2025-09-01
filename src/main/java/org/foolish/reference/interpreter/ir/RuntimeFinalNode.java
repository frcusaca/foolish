package org.foolish.reference.interpreter.ir;

/** Final, immutable values. */
public sealed interface RuntimeFinalNode extends RuntimeNode, RuntimeNode.RuntimeExpr permits RuntimeFinalNode.IntValue, RuntimeFinalNode.UnknownValue, RuntimeFinalNode.BoolValue {
    @Override
    default NodeKind kind() { return NodeKind.FINAL; }

    record IntValue(long value) implements RuntimeFinalNode { }

    record BoolValue(boolean value) implements RuntimeFinalNode { }

    /** Represents an unknown value. */
    record UnknownValue() implements RuntimeFinalNode {
        public static final UnknownValue INSTANCE = new UnknownValue();
    }
}

