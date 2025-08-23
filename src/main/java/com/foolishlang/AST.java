
package com.foolishlang;

import java.util.*;

public final class AST {
    public sealed interface Node permits Program, Brane, Stmt, Expr, TypeExpr, Comment {}

    public static final class Program implements Node {
        public final Brane body;
        public Program(Brane body) { this.body = body; }
    }

    public static final class Brane implements Node {
        public final List<Stmt> statements;
        public Brane(List<Stmt> statements) { this.statements = statements; }
    }

    public sealed interface Stmt extends Node permits AssignStmt, TypeAssignStmt, DerefAssignStmt, ExprStmt, Comment {}

    public static final class AssignStmt implements Stmt {
        public final Expr id; // Identifier or TypeIdentifier
        public final Expr expr;
        public AssignStmt(Expr id, Expr expr) { this.id = id; this.expr = expr; }
    }

    public static final class TypeAssignStmt implements Stmt {
        public final TypeIdentifier id;
        public final Node typeExpr; // TypeExpr or TypeIdentifier
        public TypeAssignStmt(TypeIdentifier id, Node typeExpr) { this.id = id; this.typeExpr = typeExpr; }
    }

    public static final class DerefAssignStmt implements Stmt {
        public final Identifier id;
        public final Expr expr;
        public DerefAssignStmt(Identifier id, Expr expr) { this.id = id; this.expr = expr; }
    }

    public static final class ExprStmt implements Stmt {
        public final Expr expr;
        public ExprStmt(Expr expr) { this.expr = expr; }
    }

    public static final class Comment implements Stmt {
        public final String text;
        public Comment(String text) { this.text = text; }
    }

    public sealed interface Expr extends Node permits Identifier, TypeIdentifier, Literal, FuncExpr, BraneExpr, ConcatExpr, BinaryExpr, PathDerefExpr, ParenExpr {}

    public static final class ParenExpr implements Expr {
        public final Expr expr;
        public ParenExpr(Expr expr) { this.expr = expr; }
    }

    public static final class Identifier implements Expr {
        public final String name;
        public final boolean isOrdinaryQuoted;
        public Identifier(String name, boolean isOrdinaryQuoted) { this.name = name; this.isOrdinaryQuoted = isOrdinaryQuoted; }
        @Override public String toString(){ return "Id("+name+")"; }
    }

    public static final class TypeIdentifier implements Expr {
        public final String name;
        public TypeIdentifier(String name) { this.name = name; }
        @Override public String toString(){ return "TId("+name+")"; }
    }

    public sealed interface Literal extends Expr permits IntLiteral, FloatLiteral, StringLiteral, TypeLiteral {}

    public static final class IntLiteral implements Literal { public final long value; public IntLiteral(long v){ this.value=v; } }
    public static final class FloatLiteral implements Literal { public final double value; public FloatLiteral(double v){ this.value=v; } }
    public static final class StringLiteral implements Literal { public final String value; public StringLiteral(String v){ this.value=v; } }
    public static final class TypeLiteral implements Literal { public final Literal inner; public TypeLiteral(Literal inner){ this.inner=inner; } }

    public static final class FuncExpr implements Expr {
        public final List<Param> params;
        public final Brane body;
        public FuncExpr(List<Param> params, Brane body){ this.params=params; this.body=body; }
    }

    public static final class Param implements Node {
        public final Expr id; // Identifier or TypeIdentifier
        public final Node type; // TypeExpr or TypeIdentifier
        public Param(Expr id, Node type){ this.id=id; this.type=type; }
    }

    public sealed interface TypeExpr extends Node permits TypeRef, BraneType {}

    public static final class TypeRef implements TypeExpr {
        public final String name;
        public final boolean primitive;
        public TypeRef(String name, boolean primitive){ this.name=name; this.primitive=primitive; }
    }

    public static final class BraneType implements TypeExpr {
        public final List<FieldDef> fields;
        public BraneType(List<FieldDef> fields){ this.fields=fields; }
    }

    public static final class FieldDef implements Node {
        public final Expr id; // Identifier or TypeIdentifier
        public final Node type; // TypeExpr or TypeIdentifier
        public FieldDef(Expr id, Node type){ this.id=id; this.type=type; }
    }

    public static final class BraneExpr implements Expr {
        public final List<Stmt> statements;
        public BraneExpr(List<Stmt> statements){ this.statements=statements; }
    }

    public static final class ConcatExpr implements Expr {
        public final List<Expr> items; // left-to-right sequence (RPN style adjacency)
        public ConcatExpr(List<Expr> items){ this.items=items; }
    }

    public enum BinaryOp { OROR, ANDAND, EQEQ, LT, GT, LE, GE, PLUS, MINUS, STAR, SLASH }

    public static final class BinaryExpr implements Expr {
        public final BinaryOp op;
        public final Expr left, right;
        public BinaryExpr(BinaryOp op, Expr left, Expr right){ this.op=op; this.left=left; this.right=right; }
    }

    public static final class PathDeref implements Node {
        public final String op; // "^", "$", "#"
        public final Node index; // Identifier/IntLiteral or null
        public PathDeref(String op, Node index){ this.op=op; this.index=index; }
    }

    public static final class PathDerefExpr implements Expr {
        public final Expr base;
        public final List<PathDeref> derefs;
        public PathDerefExpr(Expr base, List<PathDeref> derefs){ this.base=base; this.derefs=derefs; }
    }
}
