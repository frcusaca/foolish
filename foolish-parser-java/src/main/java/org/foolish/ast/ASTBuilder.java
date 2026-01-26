package org.foolish.ast;

import org.antlr.v4.runtime.tree.TerminalNode;
import org.foolish.grammar.FoolishBaseVisitor;
import org.foolish.grammar.FoolishParser;

import java.util.ArrayList;
import java.util.List;
import java.util.regex.Pattern;

import static org.foolish.ast.AST.setCharacterization;

public class ASTBuilder extends FoolishBaseVisitor<AST> {
    static final Pattern ID_CANONICALIZER = Pattern.compile("[\u202F_\u02CD]");
    static final String INTRA_ID_SPACE = "\u02CD";

    public static final String canonicalizeIdentifierName(String name) {
        if (name == null)
            return null;

        return ID_CANONICALIZER.matcher(name).replaceAll(INTRA_ID_SPACE);
    }

    private List<String> extractCharacterizations(List<FoolishParser.CharacterizationContext> contexts) {
        if (contexts == null || contexts.isEmpty()) {
            return List.of();
        }
        List<String> result = new ArrayList<>();
        for (FoolishParser.CharacterizationContext ctx : contexts) {
            TerminalNode identifier = ctx.IDENTIFIER();
            result.add(identifier != null ? canonicalizeIdentifierName(identifier.getText()) : "");
        }
        return result;
    }

    @Override
    public AST visitProgram(FoolishParser.ProgramContext ctx) {
        AST ast = visit(ctx.branes());
        if (ast instanceof AST.Branes branes) {
            return new AST.Program(branes);
        }
        if (ast instanceof AST.Expr expr) {
            return new AST.Program(new AST.Branes(List.of(expr)));
        }
        throw new RuntimeException("Expected expression or branes from program body, got: " + ast);
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
        System.err.println("DEBUG: Unknown brane alternative");
        System.err.println("  ctx: " + ctx.getText());
        System.err.println("  standard_brane: " + ctx.standard_brane());
        System.err.println("  detach_brane: " + ctx.detach_brane());
        System.err.println("  brane_search: " + ctx.brane_search());
        System.err.println("  children: " + ctx.getChildCount());
        for (int i = 0; i < ctx.getChildCount(); i++) {
            System.err.println("    child " + i + ": " + ctx.getChild(i).getClass().getName() + " = " + ctx.getChild(i).getText());
        }
        throw new IllegalArgumentException("Unknown brane alternative");
    }

    private List<AST.Expr> collectStatements(List<FoolishParser.StmtContext> statements) {
        return statements.stream()
                .map(this::visit)
                .filter(java.util.Objects::nonNull) // Filter out comment-only statements
                .map(st -> {
                    if (st instanceof AST.Expr expr) {
                        return expr;
                    }
                    if (st instanceof AST.ConfirmStmt) {
                        return (AST.Expr) st;
                    }
                    throw new RuntimeException("Expected statement, got: " + st);
                })
                .toList();
    }

    @Override
    public AST visitStandard_brane(FoolishParser.Standard_braneContext ctx) {
        List<AST.Expr> statements = new ArrayList<>(collectStatements(ctx.stmt()));
        // Add the optional final statement without semicolon (stmt_body)
        if (ctx.stmt_body() != null) {
            AST stmtBody = visit(ctx.stmt_body());
            if (stmtBody instanceof AST.Expr expr) {
                statements.add(expr);
            }
        }
        return new AST.Brane(statements);
    }

    @Override
    public AST visitDetach_brane(FoolishParser.Detach_braneContext ctx) {
        if (ctx.detach_item_list() == null) {
            return new AST.DetachmentBrane(List.of());
        }

        List<AST.DetachmentStatement> items = new ArrayList<>();
        for (FoolishParser.Detach_itemContext itemCtx : ctx.detach_item_list().detach_item()) {
             items.add((AST.DetachmentStatement) visit(itemCtx));
        }
        return new AST.DetachmentBrane(items);
    }

    @Override
    public AST visitDetach_item(FoolishParser.Detach_itemContext ctx) {
        // Cases:
        // 1. (PLUS? characterizable_identifier (ASSIGN expr)?)
        // 2. (TILDE | TILDE_TILDE) characterizable_identifier
        // 3. HASH seek_index

        if (ctx.HASH() != null) {
            // Seek case
            String indexText = ctx.seek_index().getText();
            int index = Integer.parseInt(indexText);
            return new AST.DetachmentStatement(null, AST.UnknownExpr.INSTANCE, false, AST.DetachmentStatement.ForwardSearchType.NONE, index);
        }

        if (ctx.TILDE() != null || ctx.TILDE_TILDE() != null) {
            // Forward search case
            // The pattern is a regexp_expression, which is basically a sequence of elements.
            // We need to extract the text.
            String pattern = ctx.regexp_expression().getText();
            // We store it as an identifier for now, as DetachmentStatement uses Identifier
            AST.Identifier identifier = new AST.Identifier(pattern);

            AST.DetachmentStatement.ForwardSearchType type = ctx.TILDE() != null ?
                AST.DetachmentStatement.ForwardSearchType.LOCAL :
                AST.DetachmentStatement.ForwardSearchType.GLOBAL;
            return new AST.DetachmentStatement(identifier, AST.UnknownExpr.INSTANCE, false, type, 0);
        }

        // Standard detachment case
        boolean isPBrane = ctx.PLUS() != null;
        AST.Identifier identifier = null;
        AST.Expr expr = AST.UnknownExpr.INSTANCE;

        if (ctx.characterizable_identifier() != null) {
            identifier = (AST.Identifier) visit(ctx.characterizable_identifier());
            if (ctx.expr() != null) {
                expr = (AST.Expr) visit(ctx.expr());
            }
        }

        return new AST.DetachmentStatement(identifier, expr, isPBrane, AST.DetachmentStatement.ForwardSearchType.NONE, 0);
    }

    @Override
    public AST visitBranes(FoolishParser.BranesContext ctx) {
        List<AST.Expr> brns = new ArrayList<>();
        for (var s : ctx.primary()) {
            AST st = visit(s);
            if (st instanceof AST.Expr brn) brns.add(brn);
            else throw new RuntimeException("Expected expression in branes list, got: " + st);
        }
        if (brns.size() == 1) {
            return brns.get(0);
        }
        return new AST.Branes(brns);
    }

    @Override
    public AST visitBrane_search(FoolishParser.Brane_searchContext ctx) {
        return new AST.SearchUP();
    }


    @Override
    public AST visitStmt(FoolishParser.StmtContext ctx) {
        if (ctx.stmt_body() != null) {
            return visit(ctx.stmt_body());
        }
        return null; // Comment-only statement
    }

    @Override
    public AST visitStmt_body(FoolishParser.Stmt_bodyContext ctx) {
        if (ctx.expr() != null) {
            return visit(ctx.expr());
        } else if (ctx.confirm_stmt() != null) {
            return visit(ctx.confirm_stmt());
        } else {
            return visit(ctx.assignment());
        }
    }

    @Override
    public AST visitAssignment(FoolishParser.AssignmentContext ctx) {
        AST.Identifier identifier = (AST.Identifier) visit(ctx.characterizable_identifier());
        AST.Expr expr = (AST.Expr) visit(ctx.expr());

        if (ctx.CONST_ASSIGN() != null) { // <=>
            return new AST.Assignment(identifier, new AST.ConstanticExpr(expr));
        } else if (ctx.SFF_ASSIGN() != null) { // <<=>>
            return new AST.Assignment(identifier, new AST.SFFExpr(expr));
        } else if (ctx.ONE_SHOT_ASSIGN() != null) { // =$
            // Assuming default anchor behavior (TAIL)
            return new AST.Assignment(identifier, new AST.OneShotSearchExpr(expr, SearchOperator.TAIL));
        }

        return new AST.Assignment(identifier, expr);
    }

    @Override
    public AST visitConfirm_stmt(FoolishParser.Confirm_stmtContext ctx) {
        AST.Expr left = (AST.Expr) visit(ctx.expr(0));
        AST.Expr right = (AST.Expr) visit(ctx.expr(1));
        return new AST.ConfirmStmt(left, right);
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
        AST.Expr anchor = (AST.Expr) visit(ctx.primary());

        // Process all postfix operations in order
        for (var postfixOp : ctx.postfix_op()) {
            if (postfixOp.DOT() != null && postfixOp.characterizable_identifier() != null) {
                // Handle dereference: anchor.identifier
                AST.Identifier coordinate = (AST.Identifier) visit(postfixOp.characterizable_identifier());
                anchor = new AST.DereferenceExpr(anchor, coordinate);
            } else if (postfixOp.regexp_operator() != null && postfixOp.regexp_expression() != null) {
                // Handle regexp search: anchor ? pattern or anchor ?? pattern
                String opText = postfixOp.regexp_operator().getText();
                SearchOperator operator = "?".equals(opText) ? SearchOperator.REGEXP_LOCAL : SearchOperator.REGEXP_GLOBAL;
                String pattern = canonicalizeIdentifierName(postfixOp.regexp_expression().getText());
                anchor = new AST.RegexpSearchExpr(anchor, operator, pattern);
            } else if (postfixOp.HASH() != null && postfixOp.seek_index() != null) {
                // Handle seek: anchor#N or anchor#-N
                String indexText = postfixOp.seek_index().getText();
                int offset = Integer.parseInt(indexText);
                anchor = new AST.SeekExpr(anchor, offset);
            } else if (postfixOp.CARET() != null) {
                // Handle head: anchor^
                anchor = new AST.OneShotSearchExpr(anchor, SearchOperator.HEAD);
            } else if (postfixOp.DOLLAR() != null) {
                // Handle tail: anchor$
                anchor = new AST.OneShotSearchExpr(anchor, SearchOperator.TAIL);
            }
        }

        return anchor;
    }

    @Override
    public AST visitPrimary(FoolishParser.PrimaryContext ctx) {
        if (ctx.characterizable() != null) return visit(ctx.characterizable());
        if (ctx.expr() != null) {
            if (ctx.LT() != null && ctx.GT() != null) { // <expr>
                 return new AST.ConstanticExpr((AST.Expr) visit(ctx.expr()));
            } else if (ctx.L_DOUBLE_ANGLE() != null && ctx.R_DOUBLE_ANGLE() != null) { // <<expr>>
                 return new AST.SFFExpr((AST.Expr) visit(ctx.expr()));
            }
            return visit(ctx.expr()); // (expr)
        }
        return AST.UnknownExpr.INSTANCE;
    }

    @Override
    public AST visitCharacterizable(FoolishParser.CharacterizableContext ctx) {
        if (ctx.characterizable_identifier() != null) {
            return visit(ctx.characterizable_identifier());
        }

        List<String> characterizations = extractCharacterizations(ctx.characterization());

        AST ret;
        if (ctx.literal() != null) {
            ret = visit(ctx.literal());
        } else if (ctx.brane() != null) {
            ret = visit(ctx.brane());
        } else {
            throw new IllegalStateException("Characterizable must be literal or brane when not identifier");
        }
        return !characterizations.isEmpty() ? setCharacterization(characterizations, ret) : ret;
    }

    @Override
    public AST visitCharacterizable_identifier(FoolishParser.Characterizable_identifierContext ctx) {
        List<String> characterizations = extractCharacterizations(ctx.characterization());

        // Handle case where IDENTIFIER might be null (should not happen per grammar, but defensive)
        if (ctx.IDENTIFIER() == null) {
            throw new IllegalStateException("characterizable_identifier requires an IDENTIFIER token");
        }

        String id = canonicalizeIdentifierName(ctx.IDENTIFIER().getText());

        if (characterizations.isEmpty()) {
            return new AST.Identifier(id);
        } else {
            return setCharacterization(characterizations, new AST.Identifier(id));
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
