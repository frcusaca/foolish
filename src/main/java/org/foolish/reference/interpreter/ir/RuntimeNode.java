package org.foolish.reference.interpreter.ir;

/**
 * Base runtime node kind used by the quick interpreter. Nodes are immutable unless
 * otherwise noted. We distinguish nodes that wrap source AST, final values, and
 * intermediate partially-evaluated nodes.
 */
public interface RuntimeNode {
    NodeKind kind();

    enum NodeKind { SOURCE, FINAL, INTERMEDIATE }

    interface RuntimeExpr extends RuntimeNode {}
    interface RuntimeStatement extends RuntimeNode {}
    interface RuntimeBrane extends RuntimeNode {}
}

