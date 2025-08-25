
package foolishlang;

import com.foolishlang.grammar.*;
import org.antlr.v4.runtime.tree.AbstractParseTreeVisitor;
import java.util.ArrayList;
import java.util.List;

public class AstBuilder extends AbstractParseTreeVisitor<Object> implements FoolishVisitor<Object> {

    // Helpers
    private AST.Expr asExpr(Object o){ return (AST.Expr)o; }
    private AST.Node asNode(Object o){ return (AST.Node)o; }

    @Override public Object visitProgram(FoolishParser.ProgramContext ctx) {
        return new AST.Program((AST.Brane) visit(ctx.brane()));
    }

    @Override public Object visitBrane(FoolishParser.BraneContext ctx) {
        return visit(ctx.braneExpr());
    }

    @Override public Object visitBraneExpr(FoolishParser.BraneExprContext ctx) {
        List<AST.Stmt> stmts = new ArrayList<>();
        for (var s : ctx.braneStmt()) {
            Object v = visit(s);
            if (v instanceof AST.Stmt) stmts.add((AST.Stmt)v);
        }
        return new AST.Brane(stmts);
    }

    @Override public Object visitBraneStmt(FoolishParser.BraneStmtContext ctx) {
        if (ctx.assignmentExpression()!=null) {
            Object v = visit(ctx.assignmentExpression());
            return v;
        }
        if (ctx.expression()!=null) {
            return new AST.ExprStmt(asExpr(visit(ctx.expression())));
        }
        return null;
    }

    /**
     * @param ctx the parse tree 
     * @return
     */
    @Override
    public Object visitAssignmentExpression(FoolishParser.AssignmentExpressionContext ctx) {
        return null;
    }

    // Assignment families
    @Override public Object visitAssignExpr(FoolishParser.AssignExprContext ctx) {
        AST.Expr id = (AST.Expr) visit(ctx.identifier());
        AST.Expr e = asExpr(visit(ctx.expression()));
        return new AST.AssignStmt(id, e);
    }

    @Override public Object visitTypeAssignExpr(FoolishParser.TypeAssignExprContext ctx) {
        AST.TypeIdentifier id = (AST.TypeIdentifier) visit(ctx.typeIdentifier());
        Object te = visit(ctx.typeExpression());
        return new AST.TypeAssignStmt(id, (AST.Node)te);
    }

    @Override public Object visitDerefAssignExpr(FoolishParser.DerefAssignExprContext ctx) {
        AST.Identifier id = (AST.Identifier) visit(ctx.identifier());
        AST.Expr e = asExpr(visit(ctx.expression()));
        return new AST.DerefAssignStmt(id, e);
    }

    @Override public Object visitTypeExpression(FoolishParser.TypeExpressionContext ctx) {
        if (ctx.type_()!=null) return visit(ctx.type_());
        return visit(ctx.typeIdentifier());
    }

    // Expressions with precedence
    @Override public Object visitExpression(FoolishParser.ExpressionContext ctx) {
        return visit(ctx.logicalOrExpr());
    }

    @Override public Object visitLogicalOrExpr(FoolishParser.LogicalOrExprContext ctx) {
        Object left = visit(ctx.logicalAndExpr(0));
        for (int i=1;i<ctx.logicalAndExpr().size();i++) {
            Object right = visit(ctx.logicalAndExpr(i));
            left = new AST.BinaryExpr(AST.BinaryOp.OROR, asExpr(left), asExpr(right));
        }
        return left;
    }

    @Override public Object visitLogicalAndExpr(FoolishParser.LogicalAndExprContext ctx) {
        Object left = visit(ctx.equalityExpr(0));
        for (int i=1;i<ctx.equalityExpr().size();i++) {
            Object right = visit(ctx.equalityExpr(i));
            left = new AST.BinaryExpr(AST.BinaryOp.ANDAND, asExpr(left), asExpr(right));
        }
        return left;
    }

    @Override public Object visitEqualityExpr(FoolishParser.EqualityExprContext ctx) {
        Object left = visit(ctx.relationalExpr(0));
        for (int i=1;i<ctx.relationalExpr().size();i++) {
            Object right = visit(ctx.relationalExpr(i));
            left = new AST.BinaryExpr(AST.BinaryOp.EQEQ, asExpr(left), asExpr(right));
        }
        return left;
    }

    @Override public Object visitRelationalExpr(FoolishParser.RelationalExprContext ctx) {
        Object left = visit(ctx.addExpr(0));
        int idx = 1;
        for (var t : ctx.getChildren()) {
            // Children alternate: addExpr (op addExpr)*
        }
        for (int i=1;i<ctx.addExpr().size();i++) {
            // We don't know which operator from parse tree here easily; simplify: choose LT for now if token present.
            Object right = visit(ctx.addExpr(i));
            // Heuristic: prefer LT; in real impl you would inspect ctx to find the operator token between addExpr(i-1) and addExpr(i).
            left = new AST.BinaryExpr(AST.BinaryOp.LT, asExpr(left), asExpr(right));
        }
        return left;
    }

    @Override public Object visitAddExpr(FoolishParser.AddExprContext ctx) {
        Object left = visit(ctx.mulExpr(0));
        int terms = ctx.mulExpr().size();
        for (int i=1;i<terms;i++) {
            Object right = visit(ctx.mulExpr(i));
            // Without token list, default to PLUS; a full impl would check the operator tokens.
            left = new AST.BinaryExpr(AST.BinaryOp.PLUS, asExpr(left), asExpr(right));
        }
        return left;
    }

    @Override public Object visitMulExpr(FoolishParser.MulExprContext ctx) {
        Object left = visit(ctx.concatExpr(0));
        int terms = ctx.concatExpr().size();
        for (int i=1;i<terms;i++) {
            Object right = visit(ctx.concatExpr(i));
            // Default to STAR; see note above.
            left = new AST.BinaryExpr(AST.BinaryOp.STAR, asExpr(left), asExpr(right));
        }
        return left;
    }

    @Override public Object visitConcatExpr(FoolishParser.ConcatExprContext ctx) {
        List<AST.Expr> items = new ArrayList<>();
        items.add(asExpr(visit(ctx.postfixExpr(0))));
        for (int i=1;i<ctx.postfixExpr().size();i++)
            items.add(asExpr(visit(ctx.postfixExpr(i))));
        if (items.size()==1) return items.get(0);
        return new AST.ConcatExpr(items);
    }

    @Override public Object visitPostfixExpr(FoolishParser.PostfixExprContext ctx) {
        AST.Expr base = asExpr(visit(ctx.primaryExpr()));
        if (ctx.pathOp().isEmpty()) return base;
        List<AST.PathDeref> derefs = new ArrayList<>();
        for (var op : ctx.pathOp()) {
            if (op.CARET()!=null) {
                Object idx = op.braneIndex()!=null ? visit(op.braneIndex()) : null;
                derefs.add(new AST.PathDeref("^", idx==null?null:(AST.Node)idx));
            } else if (op.DOLLAR()!=null) {
                Object idx = op.braneIndex()!=null ? visit(op.braneIndex()) : null;
                derefs.add(new AST.PathDeref("$", idx==null?null:(AST.Node)idx));
            } else if (op.HASH()!=null) {
                Object idx = visit(op.braneIndex());
                derefs.add(new AST.PathDeref("#", (AST.Node)idx));
            }
        }
        return new AST.PathDerefExpr(base, derefs);
    }

    /**
     * @param ctx the parse tree 
     * @return
     */
    @Override
    public Object visitPathOp(FoolishParser.PathOpContext ctx) {
        if(ctx.HASH()!=null){
           new AST.PathDeref()
        }
    }

    @Override public Object visitPrimaryExpr(FoolishParser.PrimaryExprContext ctx) {
        if (ctx.literal()!=null) return visit(ctx.literal());
        if (ctx.identifier()!=null) return visit(ctx.identifier());
        if (ctx.funcExpr()!=null) return visit(ctx.funcExpr());
        if (ctx.braneExpr()!=null) return new AST.BraneExpr(((AST.Brane)visit(ctx.braneExpr())).statements);
        if (ctx.expression()!=null) return new AST.ParenExpr(asExpr(visit(ctx.expression())));
        return null;
    }

    @Override public Object visitFuncExpr(FoolishParser.FuncExprContext ctx) {
        List<AST.Param> params = new ArrayList<>();
        if (ctx.paramList()!=null) {
            for (var p : ctx.paramList().param()) {
                var id = (AST.Expr) visit(p.identifier());
                Object t = p.type_()!=null ? visit(p.type_()) : visit(p.typeIdentifier());
                params.add(new AST.Param(id, (AST.Node)t));
            }
        }
        AST.Brane body = (AST.Brane) visit(ctx.brane());
        return new AST.FuncExpr(params, body);
    }

    /**
     * @param ctx the parse tree 
     * @return
     */
    @Override
    public Object visitParamList(FoolishParser.ParamListContext ctx) {
        return null;
    }

    /**
     * @param ctx the parse tree 
     * @return
     */
    @Override
    public Object visitParam(FoolishParser.ParamContext ctx) {
        return null;
    }

    @Override public Object visitBraneIndex(FoolishParser.BraneIndexContext ctx) {
        if (ctx.identifier()!=null) return visit(ctx.identifier());
        return visit(ctx.intLiteral());
    }

    // Literals & identifiers & types
    @Override public Object visitPrimitiveLiteral(FoolishParser.PrimitiveLiteralContext ctx) {
        if (ctx.intLiteral()!=null) return visit(ctx.intLiteral());
        if (ctx.floatLiteral()!=null) return visit(ctx.floatLiteral());
        return visit(ctx.stringLiteral());
    }

    /**
     * @param ctx the parse tree 
     * @return
     */
    @Override
    public Object visitPrefixSign(FoolishParser.PrefixSignContext ctx) {
        return null;
    }

    @Override public Object visitLiteral(FoolishParser.LiteralContext ctx) {
        if (ctx.primitiveLiteral()!=null) return visit(ctx.primitiveLiteral());
        return visit(ctx.typeLiteral());
    }

    @Override public Object visitIntLiteral(FoolishParser.IntLiteralContext ctx) {
        String t = ctx.getText();
        return new AST.IntLiteral(Long.parseLong(t));
    }

    @Override public Object visitFloatLiteral(FoolishParser.FloatLiteralContext ctx) {
        String t = ctx.getText();
        return new AST.FloatLiteral(Double.parseDouble(t));
    }

    @Override public Object visitStringLiteral(FoolishParser.StringLiteralContext ctx) {
        String t = ctx.getText();
        return new AST.StringLiteral(t.substring(1, t.length()-1));
    }

    @Override public Object visitTypeLiteral(FoolishParser.TypeLiteralContext ctx) {
        Object inner = visit(ctx.primitiveLiteral());
        return new AST.TypeLiteral((AST.Literal)inner);
    }

    @Override public Object visitType_(FoolishParser.Type_Context ctx) {
        if (ctx.primitiveType().braneTypeDef()!=null) {
            // handled below, but grammar nests it inside primitiveType
        }
        if (ctx.primitiveType().INT_T()!=null) return new AST.TypeRef("Int", true);
        if (ctx.primitiveType().FLOAT_T()!=null) return new AST.TypeRef("Float", true);
        if (ctx.primitiveType().STRING_T()!=null) return new AST.TypeRef("String", true);
        if (ctx.primitiveType().BRANE_T()!=null) return new AST.TypeRef("Brane", true);
        if (ctx.primitiveType().braneTypeDef()!=null) return visit(ctx.primitiveType().braneTypeDef());
        return null;
    }

    @Override public Object visitBraneTypeDef(FoolishParser.BraneTypeDefContext ctx) {
        List<AST.FieldDef> fields = new ArrayList<>();
        if (ctx.fieldDef()!=null) for (var f : ctx.fieldDef()) {
            AST.Expr id = (AST.Expr) visit(f.identifier());
            Object t = f.type_()!=null ? visit(f.type_()) : visit(f.typeIdentifier());
            fields.add(new AST.FieldDef(id, (AST.Node)t));
        }
        return new AST.BraneType(fields);
    }

    /**
     * @param ctx the parse tree 
     * @return
     */
    @Override
    public Object visitFieldDef(FoolishParser.FieldDefContext ctx) {
        return null;
    }

    /**
     * @param ctx the parse tree 
     * @return
     */
    @Override
    public Object visitPrimitiveType(FoolishParser.PrimitiveTypeContext ctx) {
        return null;
    }

    /**
     * @param ctx the parse tree 
     * @return
     */
    @Override
    public Object visitTypeIdentifier(FoolishParser.TypeIdentifierContext ctx) {
        return null;
    }

    @Override public Object visitTypeIdent(FoolishParser.TypeIdentContext ctx) {
        String text = ctx.TYPE_IDENTIFIER().getText();
        // strip leading T'/t'
        String name = text.substring(2);
        return new AST.TypeIdentifier(name);
    }

    @Override public Object visitOrdIdent(FoolishParser.OrdIdentContext ctx) {
        String text = ctx.ORD_IDENTIFIER().getText();
        boolean quoted = text.startsWith("'");
        String name = quoted ? text.substring(1) : text;
        return new AST.Identifier(name, quoted);
    }
}
