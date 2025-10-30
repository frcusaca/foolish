package org.foolish.fvm;

import org.foolish.ast.AST;
import java.util.List;

/** Firoe wrapper for an if expression. */
class IfFiroe extends Firoe {
    private final Firoe condition;
    private final Firoe thenExpr;
    private final Firoe elseExpr;
    private final List<IfFiroe> elseIfs;

    IfFiroe(Insoe base, Firoe condition, Firoe thenExpr, Firoe elseExpr, List<IfFiroe> elseIfs) {
        super(base);
        if (base != null) {
            base.as(AST.IfExpr.class);
        }
        this.condition = condition;
        this.thenExpr = thenExpr;
        this.elseExpr = elseExpr;
        this.elseIfs = elseIfs == null ? List.of() : List.copyOf(elseIfs);
    }

    public Firoe condition() { return condition; }
    public Firoe thenExpr() { return thenExpr; }
    public Firoe elseExpr() { return elseExpr; }
    public List<IfFiroe> elseIfs() { return elseIfs; }
    public String toString() {
        return FormatterFactory.verbose().format(this);
    }
}
