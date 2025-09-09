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
        List<Midoe> stmts = new ArrayList<>();
        for (AST.Expr expr : brane.statements()) {
            stmts.add(ensureMidoe(translate(expr)));
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
            return Finear.UNKNOWN;
        }
        if (expr instanceof AST.IntegerLiteral lit) {
            return Finear.of(lit.value());
        } else if (expr instanceof AST.Identifier id) {
            return new Midoe(new IdentifierExpr(toCharacterizable(id)));
        } else if (expr instanceof AST.Assignment asg) {
            Characterizable id = new Characterizable(asg.id());
            return new Midoe(new Assignment(id, ensureMidoe(translate(asg.expr()))));
        } else if (expr instanceof AST.BinaryExpr bin) {
            Midoe left = ensureMidoe(translate(bin.left()));
            Midoe right = ensureMidoe(translate(bin.right()));
            return new Midoe(new BinaryExpr(bin.op(), left, right));
        } else if (expr instanceof AST.UnaryExpr un) {
            return new Midoe(new UnaryExpr(un.op(), ensureMidoe(translate(un.expr()))));
        } else if (expr instanceof AST.UnknownExpr) {
            return Finear.UNKNOWN;
        } else if (expr instanceof AST.Brane br) {
            return new Midoe(translate(br));
        } else if (expr instanceof AST.Branes brs) {
            return new Midoe(translate(brs));
        } else if (expr instanceof AST.IfExpr ifExpr) {
            Midoe cond = ensureMidoe(translate(ifExpr.condition()));
            Midoe thenExpr = ensureMidoe(translate(ifExpr.thenExpr()));
            Midoe elseExpr = ensureMidoe(translate(ifExpr.elseExpr()));
            List<IfExpr> elseIfs = new ArrayList<>();
            for (AST.IfExpr e : ifExpr.elseIfs()) {
                Midoe translated = ensureMidoe(translate(e));
                if (translated.progress_heap().get(0) instanceof IfExpr elif) {
                    elseIfs.add(elif);
                }
            }
            return new Midoe(new IfExpr(cond, thenExpr, elseExpr, elseIfs));
        }
        throw new IllegalArgumentException("Unsupported AST expression: " + expr.getClass().getSimpleName());
    }

    private Midoe ensureMidoe(Targoe t) {
        if (t == null) return new Midoe();
        if (t instanceof Midoe m) return m;
        if (t instanceof Insoe i) return new Midoe(i);
        Midoe m = new Midoe();
        m.progress_heap().add(t);
        return m;
    }

    private Characterizable toCharacterizable(AST.Identifier id) {
        if (id == null) return null;
        Characterizable parent = toCharacterizable(id.characterization());
        return new Characterizable(parent, id.id());
    }
}
