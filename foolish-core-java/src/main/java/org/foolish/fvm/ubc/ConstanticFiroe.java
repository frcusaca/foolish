package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

public class ConstanticFiroe extends FiroeWithBraneMind {
    private final AST.AngleBracketExpr ast;
    private final FIR inner;
    private FIR result = null;

    public ConstanticFiroe(AST.AngleBracketExpr ast, FIR inner) {
        super(ast);
        this.ast = ast;
        this.inner = inner;
    }

    @Override
    protected void initialize() {
        setInitialized();
        enqueueFirs(inner);
    }

    @Override
    public boolean isAbstract() {
        return result == null || result.isAbstract();
    }

    @Override
    public void step() {
        super.step(); // Steps inner

        if (result == null && inner.getNyes().ordinal() >= Nyes.RESOLVED.ordinal()) {
            // Inner is resolved.

            FIR val = inner;

            // Unwrap IdentifierFiroe to get RAW value
            if (val instanceof IdentifierFiroe idFiroe) {
                 val = idFiroe.raw;
            } else if (val instanceof AssignmentFiroe assFiroe) {
                 val = assFiroe.getResult();
            }

            if (val instanceof BraneFiroe brane) {
                 // Clone WITH detachments (keep predecessor)
                 result = brane.clone(true);
            } else {
                 result = val;
            }
        }
    }

    public FIR getResult() {
         return result;
    }

    @Override
    public long getValue() {
        if (result == null) throw new IllegalStateException("ConstanticFiroe not evaluated");
        return result.getValue();
    }

    @Override
    public String toString() {
        return ast.toString();
    }
}
