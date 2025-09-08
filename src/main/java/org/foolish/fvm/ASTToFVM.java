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
        List<Instruction> stmts = new ArrayList<>();
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

    private Instruction translate(AST.Expr expr) {
        if (expr instanceof AST.IntegerLiteral lit) {
            return new IntegerLiteral(lit.value());
        } else if (expr instanceof AST.Identifier id) {
            return new IdentifierExpr(toCharacterizable(id));
        } else if (expr instanceof AST.Assignment asg) {
            Characterizable id = new Characterizable(asg.id());
            return new Assignment(id, translate(asg.expr()));
        } else if (expr instanceof AST.BinaryExpr bin) {
            return new BinaryExpr(bin.op(), translate(bin.left()), translate(bin.right()));
        } else if (expr instanceof AST.UnaryExpr un) {
            return new UnaryExpr(un.op(), translate(un.expr()));
        } else if (expr instanceof AST.Brane br) {
            return translate(br);
        } else if (expr instanceof AST.Branes brs) {
            return translate(brs);
        } else if (expr instanceof AST.IfExpr ifExpr) {
            Instruction cond = translate(ifExpr.condition());
            Instruction thenExpr = translate(ifExpr.thenExpr());
            Instruction elseExpr = translate(ifExpr.elseExpr());
            List<IfExpr> elseIfs = new ArrayList<>();
            for (AST.IfExpr e : ifExpr.elseIfs()) {
                elseIfs.add((IfExpr) translate(e));
            }
            return new IfExpr(cond, thenExpr, elseExpr, elseIfs);
        } else if (expr instanceof AST.UnknownExpr) {
            throw new IllegalArgumentException("Unknown expression encountered");
        }
        throw new IllegalArgumentException("Unsupported AST expression: " + expr.getClass().getSimpleName());
    }

    private Characterizable toCharacterizable(AST.Identifier id) {
        if (id == null) return null;
        Characterizable parent = toCharacterizable(id.characterization());
        return new Characterizable(parent, id.id());
    }
}
