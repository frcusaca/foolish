package org.foolish.reference.interpreter;

import org.foolish.ast.AST;
import org.foolish.reference.interpreter.ir.RuntimeSourceCodeNode;

public class RuntimeFactory {
    public static RuntimeSourceCodeNode.SourceBranes fromStringToSource(String code) {
        // Basic check: code must start with '{' and end with '}'
        if (code == null || !code.trim().startsWith("{") || !code.trim().endsWith("}")) {
            throw new IllegalArgumentException("Code must start with '{' and end with '}'");
        }
        // Parse code to AST (assume ASTBuilder or similar exists)
        AST.Branes branes = ASTBuilder.parseBranes(code);
        return new RuntimeSourceCodeNode.SourceBranes(branes);
    }
}
