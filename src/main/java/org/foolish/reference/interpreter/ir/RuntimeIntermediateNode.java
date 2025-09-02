package org.foolish.reference.interpreter.ir;

import java.util.List;

public abstract class RuntimeIntermediateNode implements RuntimeNode {
    // Marker for intermediate nodes containing mixtures of source/final nodes
    public static class EvaluatedBrane extends RuntimeIntermediateNode {
        private final List<RuntimeNode> statements;
        public EvaluatedBrane(List<RuntimeNode> statements) { this.statements = statements; }
        public List<RuntimeNode> getStatements() { return statements; }
    }
    // Add more intermediate node types as needed
}
