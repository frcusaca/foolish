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

    private Targoe translate(AST.Expr expr) {
        if (expr == null) {
            return Finer.UNKNOWN;
        }
        if (expr instanceof AST.IntegerLiteral lit) {
            return Finer.of(lit.value());
        } else if (expr instanceof AST.Identifier id) {
            return new Midoe(new IdentifierExpr(toCharacterizable(id)));
        } else if (expr instanceof AST.Assignment asg) {
            Characterizable id = new Characterizable(asg.id());
            return new Midoe(new Assignment(id, translate(asg.expr())));
        } else if (expr instanceof AST.BinaryExpr bin) {
            Targoe left = translate(bin.left());
            Targoe right = translate(bin.right());
            return new Midoe(new BinaryExpr(bin.op(), left, right));
        } else if (expr instanceof AST.UnaryExpr un) {
            return new Midoe(new UnaryExpr(un.op(), translate(un.expr())));
        } else if (expr instanceof AST.UnknownExpr) {
            return Finer.UNKNOWN;
        } else if (expr instanceof AST.Brane br) {
            return new Midoe(translate(br));
        } else if (expr instanceof AST.Branes brs) {
            return new Midoe(translate(brs));
        } else if (expr instanceof AST.IfExpr ifExpr) {
            Targoe cond = translate(ifExpr.condition());
            Targoe thenExpr = translate(ifExpr.thenExpr());
            Targoe elseExpr = translate(ifExpr.elseExpr());
            List<IfExpr> elseIfs = new ArrayList<>();
            for (AST.IfExpr e : ifExpr.elseIfs()) {
                Targoe translated = translate(e);
                if (translated instanceof Midoe m && m.heap().get(0) instanceof IfExpr elif) {
                    elseIfs.add(elif);
                }
            }
            return new Midoe(new IfExpr(cond, thenExpr, elseExpr, elseIfs));
        }
        throw new IllegalArgumentException("Unsupported AST expression: " + expr.getClass().getSimpleName());
    }

    private Characterizable toCharacterizable(AST.Identifier id) {
        if (id == null) return null;
        Characterizable parent = toCharacterizable(id.characterization());
        return new Characterizable(parent, id.id());
    }
}
