package org.foolish.ast;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

public sealed interface AST permits AST.Program,AST.Expr{
    record Program(Branes brane) implements AST {
    }


    sealed interface Expr extends AST permits Literal, VarRef, BinaryExpr, UnaryExpr, Brane, Branes, IfExpr, UnknownExpr, Stmt {

    }
    record Brane(List<Expr> statements) implements Expr {
        public String toString() {
            StringBuilder sb = new StringBuilder();
            sb.append("{\n");
            for (Expr expr : statements) {
                sb.append("  ").append(expr).append(";\n");
            }
            sb.append("}");
            return sb.toString();
        }
    }
    record Branes(List<Brane> branes) implements Expr {
        public String toString() {
            StringBuilder sb = new StringBuilder();
            for (Brane brane : branes) {
                sb.append(brane).append("\n");
            }
            return sb.toString();
        }
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


    sealed interface Stmt extends Expr permits Assignment{
    }


    record Assignment(String id, Expr expr) implements Stmt {
        public String toString() {
            return id + " = " + expr;
        }
    }

    record UnknownExpr() implements Expr {
        public static final UnknownExpr INSTANCE = new UnknownExpr();

        public String toString() {
            return "???";
        }
    }

    record IfExpr(Expr condition, Expr thenExpr, Expr elseExpr, List<IfExpr> elseIfs) implements Expr {
        public IfExpr(Expr condition, Expr thenExpr, Expr elseExpr, List<IfExpr> elseIfs) {
            this.condition = condition;
            this.thenExpr = Objects.requireNonNullElse(thenExpr,UnknownExpr.INSTANCE);
            this.elseExpr = Objects.requireNonNullElse(elseExpr, UnknownExpr.INSTANCE);
            this.elseIfs = Objects.requireNonNullElse(elseIfs, new ArrayList<>());
            for (IfExpr elseIf : this.elseIfs) {
                assert (elseIf.elseExpr == null ||
                        elseIf.elseExpr == UnknownExpr.INSTANCE) &&
                        elseIf.elseIfs.isEmpty() : "elif if cannot have else or elif";
            }
        }

        public String toString() {
            StringBuilder sb = new StringBuilder();
            sb.append("if ").append(condition).append(" then ").append(thenExpr);
            for (IfExpr elseIf : elseIfs) {
                sb.append(" elif ").append(elseIf.condition()).append(" then ").append(elseIf.thenExpr());
            }
            if (elseExpr != null && elseExpr != UnknownExpr.INSTANCE) {
                sb.append(" else ").append(elseExpr);
            }
            return sb.toString();
        }
    }
}