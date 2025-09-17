package org.foolish.fvm;

import org.foolish.ast.AST;

/** Midoe wrapper for a literal integer expression. */
class LiteralMidoe extends Midoe {
    private final long value;

    LiteralMidoe(Insoe base, long value) {
        super(base);
        base.as(AST.IntegerLiteral.class);
        this.value = value;
    }

    public long value() { return value; }
}
