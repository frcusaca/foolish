package org.foolish.fvm;

import org.foolish.ast.AST;
import java.util.ArrayList;
import java.util.List;
/**
 * Utilities for constructing {@link Midoe} trees from arbitrary {@link Targoe}
 * instances.  Each resulting {@code Midoe} has its originating {@link Targoe}
 * at the bottom of its progress heap.
 */
public final class MidoeVm {
    private MidoeVm() {}

    /** Wraps the given target in a {@link Midoe}, attaching it to the progress heap. */
    public static Midoe wrap(Targoe targoe) {
        return wrap(targoe, 0);
    }
    public static Midoe wrap(Targoe targoe, int line) {
        if (targoe == null) return new Midoe();
        if (targoe instanceof Midoe m) return m;
        if (targoe instanceof Insoe in) {
            AST ast = in.ast();
            if (ast instanceof AST.Program p) {
                return new ProgramMidoe(in, wrap(new Insoe(p.branes())));
            }
            if (ast instanceof AST.Brane b) {
                List<Midoe> stmts = new ArrayList<>();
                stmts=b.statements()
                for (int line=0; line <stmts.size(); line++) {
                    AST.Expr expr = stmts.get(line);
                    stmts.add(wrap(new Insoe(expr), line));
                }
                return new BraneMidoe(in, stmts);
            }
            if (ast instanceof AST.Branes brs) {
                List<Midoe> stmts = new ArrayList<>();
                int line=0
                for (AST.Brane br : brs.branes()) {
                    for (AST.Expr expr : br.statements()) {
                        stmts.add(wrap(new Insoe(expr), line++));
                    }
                }
                return new BraneMidoe(in, stmts);
            }
            if (ast instanceof AST.Assignment a) {
                return new AssignmentMidoe(in, wrap(new Insoe(a.expr())));
            }
            if (ast instanceof AST.BinaryExpr be) {
                return new BinaryMidoe(in, wrap(new Insoe(be.left())), wrap(new Insoe(be.right())));
            }
            if (ast instanceof AST.UnaryExpr ue) {
                return new UnaryMidoe(in, wrap(new Insoe(ue.expr())));
            }
            if (ast instanceof AST.Identifier) {
                return new IdentifierMidoe(in);
            }
            if (ast instanceof AST.IfExpr iff) {
                Midoe condition = wrap(new Insoe(iff.condition()));
                Midoe thenExpr = wrap(new Insoe(iff.thenExpr()));
                Midoe elseExpr = wrap(new Insoe(iff.elseExpr()));
                List<IfMidoe> elseIfs = new ArrayList<>();
                for (AST.IfExpr e : iff.elseIfs()) {
                    elseIfs.add((IfMidoe) wrap(new Insoe(e)));
                }
                return new IfMidoe(in, condition, thenExpr, elseExpr, elseIfs);
            }
            if (ast instanceof AST.UnknownExpr) {
                return new Midoe(in);
            }
        }
        return new Midoe(targoe);
    }
}
