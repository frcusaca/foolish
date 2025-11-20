package org.foolish.ast;

import org.antlr.v4.runtime.tree.TerminalNode;
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
        } else if (ctx.standard_brane() != null) {
            return visit(ctx.standard_brane());
        } else if (ctx.detach_brane() != null) {
            return visit(ctx.detach_brane());
        }
        throw new IllegalArgumentException("Unknown brane alternative");
    }

    private List<AST.Expr> collectStatements(List<FoolishParser.StmtContext> statements) {
        return statements.stream()
                .map(this::visit)
                .map(st -> {
                    if (st instanceof AST.Expr expr) {
                        return expr;
                    }
                    throw new RuntimeException("Expected statement, got: " + st);
                })
                .toList();
    }

    private List<AST.DetachmentStatement> collectDetachmentStatements(List<FoolishParser.Detach_stmtContext> statements) {
        return statements.stream()
                .map(this::visit)
                .map(st -> {
                    if (st instanceof AST.DetachmentStatement assignment) {
                        return assignment;
                    }
                    throw new RuntimeException("Expected detachment assignment, got: " + st);
                })
                .toList();
    }

    @Override
    public AST visitStandard_brane(FoolishParser.Standard_braneContext ctx) {
        return new AST.Brane(collectStatements(ctx.stmt()));
    }

    @Override
    public AST visitDetach_brane(FoolishParser.Detach_braneContext ctx) {
        return new AST.DetachmentBrane(collectDetachmentStatements(ctx.detach_stmt()));
    }

    @Override
    public AST visitBranes(FoolishParser.BranesContext ctx) {
        List<AST.Characterizable> brns = new ArrayList<>();
        for (var s : ctx.brane()) {
            AST st = visit(s);
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
        AST.Identifier identifier = (AST.Identifier) visit(ctx.characterizable_identifier());
        AST.Expr expr = (AST.Expr) visit(ctx.expr());
        return new AST.Assignment(identifier, expr);
    }

    @Override
    public AST visitDetach_stmt(FoolishParser.Detach_stmtContext ctx) {
        AST.Identifier identifier = (AST.Identifier) visit(ctx.characterizable_identifier());
        AST.Expr expr = ctx.expr() != null ? (AST.Expr) visit(ctx.expr()) : AST.UnknownExpr.INSTANCE;
        return new AST.DetachmentStatement(identifier, expr);
    }

    @Override
    public AST visitExpr(FoolishParser.ExprContext ctx) {
        if (ctx.ifExpr() != null) return visit(ctx.ifExpr());
        if (ctx.branes() != null) return visit(ctx.branes());
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
            AST.Expr expr = (AST.Expr) visit(ctx.postfixExpr());
            return new AST.UnaryExpr(op, expr);
        } else if (ctx.MINUS() != null) {
            String op = "-";
            AST.Expr expr = (AST.Expr) visit(ctx.postfixExpr());
            return new AST.UnaryExpr(op, expr);
        } else if (ctx.MUL() != null) {
            String op = "*";
            AST.Expr expr = (AST.Expr) visit(ctx.postfixExpr());
            return new AST.UnaryExpr(op, expr);
        } else {
            return visit(ctx.postfixExpr());
        }
    }

    @Override
    public AST visitPostfixExpr(FoolishParser.PostfixExprContext ctx) {
        AST.Expr base = (AST.Expr) visit(ctx.primary());
        for (int i = 0; i < ctx.characterizable_identifier().size(); i++) {
            AST.Identifier coordinate = (AST.Identifier) visit(ctx.characterizable_identifier(i));
            base = new AST.DereferenceExpr(base, coordinate);
        }
        return base;
    }

    @Override
    public AST visitPrimary(FoolishParser.PrimaryContext ctx) {
        if (ctx.characterizable() != null) return visit(ctx.characterizable());
        if (ctx.expr() != null) return visit(ctx.expr());
        return AST.UnknownExpr.INSTANCE;
    }

    @Override
    public AST visitCharacterizable(FoolishParser.CharacterizableContext ctx) {
        if (ctx.characterizable_identifier() != null) {
            return visit(ctx.characterizable_identifier());
        }

        String characterization = "";
        if (ctx.APOSTROPHE() != null) {
            TerminalNode prefixIdentifier = ctx.IDENTIFIER();
            characterization = prefixIdentifier != null ? prefixIdentifier.getText() : "";
        }

        AST ret;
        if (ctx.literal() != null) {
            ret = visit(ctx.literal());
        } else if (ctx.brane() != null) {
            ret = visit(ctx.brane());
        } else {
            throw new IllegalStateException("Characterizable must be literal or brane when not identifier");
        }
        return ctx.APOSTROPHE() != null ? setCharacterization(characterization, ret) : ret;
    }

    @Override
    public AST visitCharacterizable_identifier(FoolishParser.Characterizable_identifierContext ctx) {
        List<TerminalNode> identifiers = ctx.IDENTIFIER();
        if (ctx.APOSTROPHE() != null) {
            String id = identifiers.get(identifiers.size() - 1).getText();
            String characterization = identifiers.size() > 1 ? identifiers.get(0).getText() : "";
            return setCharacterization(characterization, new AST.Identifier(id));
        } else {
            return new AST.Identifier(identifiers.get(0).getText());
        }
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
