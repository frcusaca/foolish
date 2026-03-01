package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.ast.SearchOperator;

/**
 * Foolish Internal Representation (FIR).
 * <p>
 * Base class for all FIRs. Holds an AST node and tracks evaluation progress via Nyes state.
 * <p>
 * See {@code projects/FIR-Invariances.md} for constraints, state machine, and method contracts.
 */
public abstract class FIR implements Cloneable {
    protected final AST ast;
    protected final String comment;
    private boolean initialized;
    protected Nyes nyes;
    private FIR parentFir = null;
    private final boolean ai;

    protected FIR(AST ast, String comment, boolean ai) {
        this.ast = ast;
        this.comment = comment;
        this.initialized = false;
        this.nyes = Nyes.UNINITIALIZED;
        this.ai = ai;
    }

    protected FIR(AST ast, String comment) {
        this(ast, comment, false);
    }

    protected FIR(AST ast) {
        this(ast, null, false);
    }

    protected FIR(String comment) {
        this(null, comment, true);
    }

    public AST ast() {
        return ast;
    }

    public String comment() {
        return comment;
    }

    public boolean isAi() {
        return ai;
    }

    public AST.SourceLocation sourceLocation() {
        return ast != null ? ast.sourceLocation() : AST.SourceLocation.UNKNOWN;
    }

    public AST.SourceLocation nearestSourceLocation() {
        AST.SourceLocation loc = sourceLocation();
        if (loc != AST.SourceLocation.UNKNOWN) {
            return loc;
        }
        FIR current = parentFir;
        while (current != null) {
            loc = current.sourceLocation();
            if (loc != AST.SourceLocation.UNKNOWN) {
                return loc;
            }
            current = current.parentFir;
        }
        return AST.SourceLocation.UNKNOWN;
    }

    public int getMyBraneStatementNumber() {
        BraneFiroe containingBrane = getMyBrane();
        if (containingBrane != null) {
            int directIndex = containingBrane.getStatementIndex(this);
            if (directIndex != -1) {
                return directIndex;
            }
            if (parentFir != null) {
                return parentFir.getMyBraneStatementNumber();
            }
        }
        return -1;
    }

    public String getLocationDescription() {
        StringBuilder sb = new StringBuilder();
        sb.append(nearestSourceLocation());
        int stmtNum = getMyBraneStatementNumber();
        if (stmtNum >= 0) {
            sb.append(" (statement #").append(stmtNum + 1).append(")");
        }
        if (isAi()) {
            sb.append(" [auto-generated]");
        }
        return sb.toString();
    }

    public String formatErrorMessage(String message) {
        StringBuilder sb = new StringBuilder();
        ExecutionContext ctx = ExecutionContext.getCurrent();
        sb.append(ctx != null ? ctx.getSourceFilename() : "unknown.foo").append(":");
        AST.SourceLocation loc = nearestSourceLocation();
        if (loc != AST.SourceLocation.UNKNOWN) {
            sb.append("line ").append(loc.line());
        } else if (isAi()) {
            sb.append("[ai]");
        } else {
            sb.append("unknown");
        }
        sb.append(" Brane@").append(getMyBraneStatementNumber()).append(" - ").append(message);
        return sb.toString();
    }

    protected boolean isInitialized() {
        return initialized;
    }

    protected void setInitialized() {
        this.initialized = true;
    }

    protected void markInitialized() {
        this.initialized = true;
        setNyes(Nyes.INITIALIZED);
    }

    protected Nyes getNyes() {
        return nyes;
    }

    protected void setNyes(Nyes nyes) {
        if (this.nyes == null) {
            this.nyes = nyes;
            return;
        }
        if (this.nyes == Nyes.CONSTANIC) {
            if (nyes != Nyes.CONSTANT && nyes != Nyes.CONSTANIC) {
                throw new IllegalStateException(formatErrorMessage(
                    "Cannot change state from CONSTANIC to " + nyes + " - FIR is immutable once CONSTANIC"));
            }
            if (nyes == Nyes.CONSTANT) {
                this.nyes = nyes;
            }
        } else if (this.nyes == Nyes.CONSTANT) {
            if (nyes != Nyes.CONSTANT) {
                throw new IllegalStateException(formatErrorMessage(
                    "Cannot change state from CONSTANT to " + nyes + " - FIR is immutable once CONSTANT"));
            }
        } else if (nyes.ordinal() < this.nyes.ordinal()) {
            throw new IllegalStateException(formatErrorMessage(
                "Cannot transition backward from " + this.nyes + " to " + nyes));
        } else {
            this.nyes = nyes;
        }
    }

    protected void setParentFir(FIR parent) {
        this.parentFir = parent;
    }

    protected FIR getParentFir() {
        return parentFir;
    }

    public boolean atConstant() {
        return getNyes() == Nyes.CONSTANT;
    }

    public boolean atConstanic() {
        return getNyes() == Nyes.CONSTANIC;
    }

    /**
     * Perform one step of evaluation.
     * @return 0 for no-op, 1 for meaningful work
     * @see "projects/FIR-Invariances.md#Step Method Contract"
     */
    public abstract int step();

    public boolean isNye() {
        return getNyes().ordinal() < Nyes.CONSTANIC.ordinal();
    }

    public boolean isConstanic() {
        return getNyes().ordinal() >= Nyes.CONSTANIC.ordinal();
    }

    public boolean isConstant() {
        return getNyes().ordinal() >= Nyes.CONSTANT.ordinal();
    }

    public long getValue() {
        throw new UnsupportedOperationException("getValue not supported for " + getClass().getSimpleName());
    }

    public BraneFiroe getMyBrane() {
        if (parentFir instanceof BraneFiroe bf) {
            return bf;
        }
        return parentFir != null ? parentFir.getMyBrane() : null;
    }

    public FiroeWithBraneMind getMyBraneContainer() {
        if (parentFir instanceof BraneFiroe || parentFir instanceof ConcatenationFiroe) {
            return (FiroeWithBraneMind) parentFir;
        }
        return parentFir != null ? parentFir.getMyBraneContainer() : null;
    }

    public int getMyBraneContainerIndex() {
        if (parentFir instanceof BraneFiroe || parentFir instanceof ConcatenationFiroe) {
            return ((FiroeWithBraneMind) parentFir).getIndexOf(this);
        }
        return parentFir != null ? parentFir.getMyBraneContainerIndex() : -1;
    }

    public static FIR unwrapConstanicable(FIR fir) {
        FIR current = fir;
        while (current != null && current instanceof Constanicable constanicable) {
            FIR result = constanicable.getResult();
            if (result == null || result == current) {
                break;
            }
            current = result;
        }
        return current;
    }

    public FoolishIndex getMyIndex() {
        FoolishIndexBuilder builder = new FoolishIndexBuilder();
        FIR current = this;
        while (current != null) {
            int index = current.getMyBraneStatementNumber();
            if (index == -1) {
                builder.prepend(0);
                break;
            }
            builder.prepend(index);
            current = current.getMyBrane();
        }
        return builder.build();
    }

    public FIR copy(Nyes targetNyes) {
        if (targetNyes != null && nyes != targetNyes) {
            FIR fresh = this.clone();
            fresh.nyes = targetNyes;
            return fresh;
        }
        if (isConstanic()) {
            return this;
        }
        if (this.ast() instanceof AST.Expr expr) {
            return new CMFir(this.ast(), this);
        }
        return this.clone();
    }

    protected static final ThreadLocal<Integer> RECURSION_DEPTH = ThreadLocal.withInitial(() -> 0);

    public java.util.Optional<FIR> valuableSelf() {
        return java.util.Optional.of(this);
    }

    @Override
    protected FIR clone() {
        try {
            return (FIR) super.clone();
        } catch (CloneNotSupportedException e) {
            throw new RuntimeException("Clone not supported for " + getClass().getSimpleName(), e);
        }
    }

    protected FIR cloneConstanic(java.util.Optional<Nyes> targetNyes) {
        return cloneConstanic(this.getParentFir(), targetNyes);
    }

    /**
     * Clones a CONSTANIC/CONSTANT FIR with updated parent chain.
     * @see "projects/FIR-Invariances.md#cloneConstanic Contract"
     */
    protected FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes) {
        if (!isConstanic()) {
            throw new IllegalStateException(formatErrorMessage(
                "cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, but this FIR is in state: " + getNyes()));
        }
        if (isConstant()) {
            return this;
        }
        FIR copy = this.clone();
        copy.setParentFir(newParent);
        copy.nyes = targetNyes.orElse(this.nyes);
        return copy;
    }

    protected static FIR createFiroeFromExpr(AST.Expr expr) {
        return switch (expr) {
            case AST.IntegerLiteral literal -> new ValueFiroe(expr, literal.value());
            case AST.BinaryExpr binary -> new BinaryFiroe(binary);
            case AST.UnaryExpr unary -> new UnaryFiroe(unary);
            case AST.IfExpr ifExpr -> new IfFiroe(ifExpr);
            case AST.Brane brane -> new BraneFiroe(brane);
            case AST.Concatenation concatenation -> new ConcatenationFiroe(concatenation);
            case AST.DetachmentBrane detachBrane -> new DetachmentBraneFiroe(detachBrane);
            case AST.Assignment assignment -> new AssignmentFiroe(assignment);
            case AST.Identifier identifier -> new IdentifierFiroe(identifier);
            case AST.RegexpSearchExpr regexpSearch ->
                DerefSearchFiroe.isExactMatch(regexpSearch.pattern())
                    ? new DerefSearchFiroe(regexpSearch)
                    : new RegexpSearchFiroe(regexpSearch);
            case AST.OneShotSearchExpr oneShotSearch -> new OneShotSearchFiroe(oneShotSearch);
            case AST.DereferenceExpr dereferenceExpr -> {
                AST.RegexpSearchExpr synthetic = new AST.RegexpSearchExpr(
                    dereferenceExpr.anchor(), SearchOperator.REGEXP_LOCAL, dereferenceExpr.coordinate().toString());
                yield new DerefSearchFiroe(synthetic, dereferenceExpr);
            }
            case AST.SeekExpr seekExpr -> new SeekFiroe(seekExpr);
            case AST.UnanchoredSeekExpr unanchoredSeekExpr -> new UnanchoredSeekFiroe(unanchoredSeekExpr);
            case AST.StayFoolishExpr stayFoolish -> {
                FIR innerFir = createFiroeFromExpr(stayFoolish.expr());
                yield new SFMarkFiroe(stayFoolish, innerFir);
            }
            case AST.StayFullyFoolishExpr stayFullyFoolish ->
                new NKFiroe("SFF marker (<<==>> syntax) not yet implemented");
            default -> new NKFiroe();
        };
    }
}
