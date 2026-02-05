package org.foolish.fvm.ubc;

/**
 * This cursor is used to initialize search from an expression already in a brane.
 */
public class ExpressionSearchCursor extends Cursor.CursorImpl {

    public ExpressionSearchCursor(FIR expr) {
        super(expr.getMyBrane(), expr.getMyBraneStatementNumber());
    }
}
