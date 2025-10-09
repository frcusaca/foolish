package org.foolish.fvm;

import org.foolish.ast.AST;
import java.util.List;

/** Midoe wrapper for an if expression. */
class IfMidoe extends Midoe {
    private final Midoe condition;
    private final Midoe thenExpr;
    private final Midoe elseExpr;
    private final List<IfMidoe> elseIfs;

    IfMidoe(Insoe base, Midoe condition, Midoe thenExpr, Midoe elseExpr, List<IfMidoe> elseIfs) {
        super(base);
        if (base != null) {
            base.as(AST.IfExpr.class);
        }
        this.condition = condition;
        this.thenExpr = thenExpr;
        this.elseExpr = elseExpr;
        this.elseIfs = elseIfs == null ? List.of() : List.copyOf(elseIfs);
    }

    public Midoe condition() { return condition; }
    public Midoe thenExpr() { return thenExpr; }
    public Midoe elseExpr() { return elseExpr; }
    public List<IfMidoe> elseIfs() { return elseIfs; }
    public String toString() {
        StringBuilder sb = new StringBuilder("MidoeIf(" + condition + ", " + thenExpr);
        for (IfMidoe elif : elseIfs)
            sb.append(", ").append(elif);
        if (elseExpr != null)
            sb.append(", ").append(elseExpr);
        sb.append(")");
        return sb.toString();
    }
}
