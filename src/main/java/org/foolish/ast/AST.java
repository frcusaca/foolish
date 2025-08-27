package org.foolish.ast;

import java.util.List;

public sealed interface AST permits AST.Brane, AST.Program, AST.Stmt, AST.Expr {
    record Program(Brane brane) implements AST {
    }

    record Brane(List<Stmt> statements) implements AST {
    }

    sealed interface Stmt extends AST permits Assignment, ExprStmt {
    }

    record Assignment(String id, Expr expr) implements Stmt {
    }

    record ExprStmt(Expr expr) implements Stmt {
    }

    sealed interface Expr extends AST permits Literal, VarRef, BinaryExpr, UnaryExpr {
    }

    record Literal(long value) implements Expr {
    }

    record VarRef(String id) implements Expr {
    }

    record BinaryExpr(String op, Expr left, Expr right) implements Expr {
    }

    record UnaryExpr(String op, Expr expr) implements Expr {
    }
}