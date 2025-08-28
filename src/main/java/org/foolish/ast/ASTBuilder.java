package org.foolish.ast;

import org.foolish.grammar.FoolishBaseVisitor;
import org.foolish.grammar.FoolishParser;

import java.util.ArrayList;
import java.util.List;

public class ASTBuilder extends FoolishBaseVisitor<AST> {

    @Override
    public AST visitProgram(FoolishParser.ProgramContext ctx) {
        return visit(ctx.branes());
    }

    @Override
    public AST visitBrane(FoolishParser.BraneContext ctx) {
        List<AST.Stmt> stmts = new ArrayList<>();
        for (var s : ctx.stmt()) {
            AST st = (AST) visit(s);
            if (st instanceof AST.Stmt stmt) stmts.add(stmt);
            else throw new RuntimeException("Expected statement, got: " + st);
        }
        return new AST.Brane(stmts);
    }

    @Override
    public AST visitBranes(FoolishParser.BranesContext ctx) {
        List<AST.Brane> brns = new ArrayList<>();
        for (var s : ctx.brane()) {
            AST st = (AST) visit(s);
            if (st instanceof AST.Brane brn) brns.add(brn);
            else throw new RuntimeException("Expected brane, got: " + st);
        }
        return new AST.Branes(brns);
    }


    @Override
    public AST visitStmt(FoolishParser.StmtContext ctx) {
        if (ctx.expr() != null) {
            return visit(ctx.expr());
        } else {
            return visit(ctx.assignment());
        }
    }

    @Override
    public AST visitAssignment(FoolishParser.AssignmentContext ctx) {
        String id = ctx.IDENTIFIER().getText();
        AST.Expr expr = (AST.Expr) visit(ctx.expr());
        return new AST.Assignment(id, expr);
    }

    @Override
    public AST visitExpr(FoolishParser.ExprContext ctx) {
        return visit(ctx.addExpr());
    }

    @Override
    public AST visitAddExpr(FoolishParser.AddExprContext ctx) {
        AST.Expr left = (AST.Expr) visit(ctx.mulExpr(0));
        for (int i = 1; i < ctx.mulExpr().size(); i++) {
            String op = ctx.getChild(2 * i - 1).getText();
            AST.Expr right = (AST.Expr) visit(ctx.mulExpr(i));
            left = new AST.BinaryExpr(op, left, right);
        }
        return left;
    }

    @Override
    public AST visitMulExpr(FoolishParser.MulExprContext ctx) {
        AST.Expr left = (AST.Expr) visit(ctx.unaryExpr(0));
        for (int i = 1; i < ctx.unaryExpr().size(); i++) {
            String op = ctx.getChild(2 * i - 1).getText();
            AST.Expr right = (AST.Expr) visit(ctx.unaryExpr(i));
            left = new AST.BinaryExpr(op, left, right);
        }
        return left;
    }

    @Override
    public AST visitUnaryExpr(FoolishParser.UnaryExprContext ctx) {
        if (ctx.PLUS() != null) {
            String op = "+";
            AST.Expr expr = (AST.Expr) visit(ctx.primary());
            return new AST.UnaryExpr(op, expr);
        } else if (ctx.MINUS() != null) {
            String op = "-";
            AST.Expr expr = (AST.Expr) visit(ctx.primary());
            return new AST.UnaryExpr(op, expr);
        } else {
            return visit(ctx.primary());
        }
    }

    @Override
    public AST visitPrimary(FoolishParser.PrimaryContext ctx) {
        if (ctx.INTEGER() != null) return new AST.Literal(Long.parseLong(ctx.INTEGER().getText()));
        if (ctx.IDENTIFIER() != null) return new AST.VarRef(ctx.IDENTIFIER().getText());
        return (AST.Expr) visit(ctx.expr());
    }
}
