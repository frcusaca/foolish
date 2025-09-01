package org.foolish.reference.interpreter;

import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.tree.ParseTree;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.foolish.reference.interpreter.ir.RuntimeSourceCodeNode;

/** Factory utilities to parse source and produce Runtime source-code nodes. */
public final class RuntimeFactory {
    private RuntimeFactory() {}

    public static AST parseToAST(String code) {
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        ParseTree tree = parser.program();
        return new ASTBuilder().visit(tree);
    }

    public static RuntimeSourceCodeNode.SourceBranes fromStringToSource(String code) {
        AST ast = parseToAST(code);
        if (ast instanceof AST.Branes branes) {
            return new RuntimeSourceCodeNode.SourceBranes(branes);
        }
        throw new IllegalStateException("Expected AST.Branes at top-level, got: " + ast.getClass());
    }

    public static RuntimeSourceCodeNode wrap(AST ast) {
        if (ast instanceof AST.Branes brs) return new RuntimeSourceCodeNode.SourceBranes(brs);
        if (ast instanceof AST.Brane b) return new RuntimeSourceCodeNode.SourceBrane(b);
        if (ast instanceof AST.Stmt s) return new RuntimeSourceCodeNode.SourceStmt(s);
        if (ast instanceof AST.Expr e) return new RuntimeSourceCodeNode.SourceExpr(e);
        throw new IllegalArgumentException("Unsupported AST type: " + ast.getClass());
    }
}

