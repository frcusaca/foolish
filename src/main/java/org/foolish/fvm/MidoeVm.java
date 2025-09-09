package org.foolish.fvm;

/**
 * Utilities for constructing {@link Midoe} trees from arbitrary {@link Targoe}
 * instances.  Each resulting {@code Midoe} has its originating {@link Targoe}
 * at the bottom of its progress heap.
 */
public final class MidoeVm {
    private MidoeVm() {}

    /** Wraps the given target in a {@link Midoe}, attaching it to the progress heap. */
    public static Midoe wrap(Targoe targoe) {
        if (targoe == null) return new Midoe();
        if (targoe instanceof Midoe m) return m;
        if (targoe instanceof Finear f) return new Midoe(f);
        if (targoe instanceof Program p) {
            return new ProgramMidoe(p, wrap(p.brane()));
        }
        if (targoe instanceof Brane b) {
            java.util.List<Midoe> stmts = new java.util.ArrayList<>();
            for (Insoe stmt : b.statements()) {
                stmts.add(wrap(stmt));
            }
            return new BraneMidoe(b, stmts);
        }
        if (targoe instanceof Assignment a) {
            return new AssignmentMidoe(a, wrap(a.expr()));
        }
        if (targoe instanceof BinaryExpr be) {
            return new BinaryMidoe(be, wrap(be.left()), wrap(be.right()));
        }
        if (targoe instanceof UnaryExpr ue) {
            return new UnaryMidoe(ue, wrap(ue.expr()));
        }
        if (targoe instanceof IdentifierExpr id) {
            return new IdentifierMidoe(id);
        }
        if (targoe instanceof IfExpr iff) {
            Midoe condition = wrap(iff.condition());
            Midoe thenExpr = wrap(iff.thenExpr());
            Midoe elseExpr = wrap(iff.elseExpr());
            java.util.List<IfMidoe> elseIfs = new java.util.ArrayList<>();
            for (IfExpr e : iff.elseIfs()) {
                elseIfs.add((IfMidoe) wrap(e));
            }
            return new IfMidoe(iff, condition, thenExpr, elseExpr, elseIfs);
        }
        return new Midoe(targoe);
    }
}
