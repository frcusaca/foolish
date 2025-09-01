package org.foolish.reference.interpreter.ir;

import org.foolish.ast.AST;

/** Nodes that simply wrap AST nodes without evaluation. */
public sealed interface RuntimeSourceCodeNode extends RuntimeNode permits RuntimeSourceCodeNode.SourceExpr, RuntimeSourceCodeNode.SourceStmt, RuntimeSourceCodeNode.SourceBrane, RuntimeSourceCodeNode.SourceBranes {
    @Override
    default NodeKind kind() { return NodeKind.SOURCE; }

    AST ast();

    record SourceExpr(AST.Expr ast) implements RuntimeSourceCodeNode, RuntimeExpr {}
    record SourceStmt(AST.Stmt ast) implements RuntimeSourceCodeNode, RuntimeStatement {}
    record SourceBrane(AST.Brane ast) implements RuntimeSourceCodeNode, RuntimeBrane {}
    record SourceBranes(AST.Branes ast) implements RuntimeSourceCodeNode {}
}

