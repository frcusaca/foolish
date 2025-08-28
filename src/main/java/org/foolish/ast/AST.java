package org.foolish.ast;

import java.util.List;

public sealed interface AST permits AST.Program, AST.Stmt, AST.Expr, AST.Brane {
    record Program(Branes brane) implements AST {
    }

    record Brane(List<Stmt> statements) implements AST {
        public String toString() {
            StringBuilder sb = new StringBuilder();
            sb.append("{\n");
            for (Stmt stmt : statements) {
                sb.append("  ").append(stmt).append(";\n");
            }
            sb.append("}");
            return sb.toString();
        }
    }

    sealed interface Stmt extends AST permits Assignment, ExprStmt, Branes {
    }

    record Branes(List<Brane> branes) implements Stmt {
        public String toString() {
            StringBuilder sb = new StringBuilder();
            for (Brane brane : branes) {
                sb.append(brane).append("\n");
            }
            return sb.toString();
        }
    }

    record Assignment(String id, Expr expr) implements Stmt {
        public String toString() {
            return id + " = " + expr;
        }
    }

    record ExprStmt(Expr expr) implements Stmt {
    }

    sealed interface Expr extends AST permits Literal, VarRef, BinaryExpr, UnaryExpr {

    }

    record Literal(long value) implements Expr {
        public String toString() {
            return Long.toString(value);
        }
    }

    record VarRef(String id) implements Expr {
        public String toString() {
            return id;
        }
    }

    record BinaryExpr(String op, Expr left, Expr right) implements Expr {
        public String toString() {
            return "(" + left + " " + op + " " + right + ")";
        }
    }

    record UnaryExpr(String op, Expr expr) implements Expr {
        public String toString() {
            return op + expr;
        }
    }
}