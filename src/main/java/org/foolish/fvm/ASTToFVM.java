package org.foolish.fvm;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;

/**
 * Translates the high level AST into executable FVM instructions.
 */
public class ASTToFVM {

    public Program translate(AST.Program program) {
        Brane brane = translate(program.brane());
        return new Program(brane);
    }

    private Brane translate(AST.Brane brane) {
        List<Targoe> stmts = new ArrayList<>();
        for (AST.Expr expr : brane.statements()) {
            stmts.add(translate(expr));
        }
        Characterizable chr = toCharacterizable(brane.characterization());
        return new SingleBrane(chr, stmts);
    }

    private Brane translate(AST.Branes branes) {
        List<Brane> list = new ArrayList<>();
        for (AST.Brane b : branes.branes()) {
            list.add(translate(b));
        }
        return new Branes(list);
    }

    private Insoe translateIf(AST.IfExpr ifExpr) {
        Targoe cond = translate(ifExpr.condition());
        Targoe thenExpr = translate(ifExpr.thenExpr());
        Targoe elseExpr = translate(ifExpr.elseExpr());
        List<IfExpr> elseIfs = new ArrayList<>();
        for (AST.IfExpr e : ifExpr.elseIfs()) {
            Insoe translated = translateIf(e);
            if (translated instanceof IfExpr ie) {
                elseIfs.add(ie);
            }
        }
        return new IfExpr(cond, thenExpr, elseExpr, elseIfs);
    }

    private Targoe translate(AST.Expr expr) {
        if (expr == null) {
            return Unknown.INSTANCE;
        }
        if (expr instanceof AST.IntegerLiteral lit) {
            return new IntegerLiteral(lit.value());
        } else if (expr instanceof AST.Identifier id) {
            return new Midoe(new IdentifierExpr(toCharacterizable(id)));
        } else if (expr instanceof AST.Assignment asg) {
            Characterizable id = new Characterizable(asg.id());
            Insoe insoe = new Assignment(id, translate(asg.expr()));
            return new Midoe(insoe);
        } else if (expr instanceof AST.BinaryExpr bin) {
            Insoe insoe = new BinaryExpr(bin.op(), translate(bin.left()), translate(bin.right()));
            return new Midoe(insoe);
        } else if (expr instanceof AST.UnaryExpr un) {
            Insoe insoe = new UnaryExpr(un.op(), translate(un.expr()));
            return new Midoe(insoe);
        } else if (expr instanceof AST.Brane br) {
            return new Midoe(translate(br));
        } else if (expr instanceof AST.Branes brs) {
            return new Midoe(translate(brs));
        } else if (expr instanceof AST.IfExpr ifExpr) {
            return new Midoe(translateIf(ifExpr));
        } else if (expr instanceof AST.UnknownExpr) {
            return Unknown.INSTANCE;
        }
        throw new IllegalArgumentException("Unsupported AST expression: " + expr.getClass().getSimpleName());
    }

    private Characterizable toCharacterizable(AST.Identifier id) {
        if (id == null) return null;
        Characterizable parent = toCharacterizable(id.characterization());
        return new Characterizable(parent, id.id());
    }
}

