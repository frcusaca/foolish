package org.foolish.reference.interpreter.ir;

import org.foolish.ast.AST;

public abstract class RuntimeSourceCodeNode implements RuntimeNode {
    public static class SourceBranes extends RuntimeSourceCodeNode {
        private final AST.Branes ast;
        public SourceBranes(AST.Branes ast) { this.ast = ast; }
        public AST.Branes ast() { return ast; }
    }
    // Add more source code node types as needed
}
