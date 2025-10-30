package org.foolish.ast;

import org.foolish.grammar.FoolishBaseVisitor;
import org.foolish.grammar.FoolishParser;

import java.util.ArrayList;
import java.util.List;

import static org.foolish.ast.AST.setCharacterization;

public class ASTBuilder extends FoolishBaseVisitor<AST> {

    @Override
    public AST visitProgram(FoolishParser.ProgramContext ctx) {
        AST.Branes brns = (AST.Branes) visit(ctx.branes());
        return new AST.Program(brns);
    }

    @Override
    public AST visitBrane(FoolishParser.BraneContext ctx) {
        if (ctx.brane_search() != null) {
            return visit(ctx.brane_search());
        }
        List<AST.Expr> exprs = new ArrayList<>();
        for (var s : ctx.stmt()) {
            AST st = visit(s);
            if (st instanceof AST.Expr expr) exprs.add(expr);
            else throw new RuntimeException("Expected statement, got: " + st);
        }
        return new AST.Brane(exprs);
    }

    @Override
    public AST visitBranes(FoolishParser.BranesContext ctx) {
        List<AST.Characterizable> brns = new ArrayList<>();
        for (var s : ctx.brane()) {
            AST st = (AST) visit(s);
            if (st instanceof AST.Characterizable brn) brns.add(brn);
            else throw new RuntimeException("Expected characterizable brane, got: " + st);
        }
        return new AST.Branes(brns);
    }

    @Override
    public AST visitBrane_search(FoolishParser.Brane_searchContext ctx) {
        return new AST.SearchUP();
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
        if (ctx.ifExpr() != null) return visit(ctx.ifExpr());
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
        } else if (ctx.MUL() != null) {
            String op = "*";
            AST.Expr expr = (AST.Expr) visit(ctx.primary());
            return new AST.UnaryExpr(op, expr);
        } else {
            return visit(ctx.primary());
        }
    }

    @Override
    public AST visitPrimary(FoolishParser.PrimaryContext ctx) {
        if (ctx.characterizable() != null) return visit(ctx.characterizable());
        if (ctx.expr() != null) return visit(ctx.expr());
        return AST.UnknownExpr.INSTANCE;
    }

    @Override
    public AST visitCharacterizable(FoolishParser.CharacterizableContext ctx) {
        String characterization = ""; // Default to empty string is the characterization
        int identifierIndex = 0;
        if (ctx.APOSTROPHE() != null) {
            if (ctx.IDENTIFIER(0) != null) {
                characterization = ctx.IDENTIFIER(0).getText();
                identifierIndex = 1;
            }
        }

        AST ret = null;
        if (ctx.literal() != null) {
            ret = visit(ctx.literal());
        } else if (ctx.brane() != null) {
            ret = visit(ctx.brane());
        } else {
            ret = new AST.Identifier(ctx.IDENTIFIER(identifierIndex).getText());
        }
        return setCharacterization(characterization, ret);
    }

    @Override
    public AST visitLiteral(FoolishParser.LiteralContext ctx) {
        // Get characterization from parent context
        if (ctx.INTEGER() != null) {
            return new AST.IntegerLiteral(Long.parseLong(ctx.INTEGER().getText()));
        }
        throw new RuntimeException("Unknown literal type");
    }

    @Override
    public AST visitIfExpr(FoolishParser.IfExprContext ctx) {
        FoolishParser.IfExprHelperIfContext theIfCtx = ctx.ifExprHelperIf();
        AST.Expr condition = (AST.Expr) visit(theIfCtx.expr(0));
        AST.Expr theThen = (AST.Expr) visit(theIfCtx.expr(1));

        AST.Expr theElse = AST.UnknownExpr.INSTANCE;
        if (ctx.ifExprHelperElse() != null)
            theElse = (AST.Expr) (visit(ctx.ifExprHelperElse().expr()));

        List<AST.IfExpr> elseIfs = new ArrayList<>();
        ctx.ifExprHelperElif().forEach(elseIfCtx -> {
            AST.Expr elseIfCondition = (AST.Expr) visit(elseIfCtx.expr(0));
            AST.Expr elseIfThen = (AST.Expr) visit(elseIfCtx.expr(1));
            // Transform else-if into nested if-else
            elseIfs.add(new AST.IfExpr(elseIfCondition, elseIfThen, null, null));
        });
        return new AST.IfExpr(condition, theThen, theElse, elseIfs);
    }
}
