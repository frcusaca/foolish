package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.apache.commons.lang3.tuple.Pair;

import java.util.LinkedList;
import java.util.List;
import java.util.Optional;
import java.util.stream.Stream;
import java.util.IdentityHashMap;

/**
 * FIR with braneMind work queue for breadth-first evaluation.
 * <p>
 * See {@code projects/FIR-Invariances.md} for constraints C5-C8.
 */
public abstract class FiroeWithBraneMind extends FIR {
    private final LinkedList<FIR> braneMind;
    private final BraneMemory braneMemory;
    protected boolean ordinated;
    protected IdentityHashMap indexLookup = new IdentityHashMap<FIR, Integer>();

    protected FiroeWithBraneMind(AST ast, String comment) {
        super(ast, comment);
        this.braneMind = new LinkedList<>();
        this.braneMemory = new BraneMemory(null);
        this.ordinated = false;
    }

    protected FiroeWithBraneMind(AST ast) {
        this(ast, null);
    }

    public void ordinateToParentBraneMind(FiroeWithBraneMind parent) {
        assert !this.ordinated;
        linkMemoryParent(parent);
        setMemoryOwner(this);
        this.ordinated = true;
    }

    /**
     * Copy constructor for cloneConstanic. Clones braneMemory with updated parent chains.
     */
    protected FiroeWithBraneMind(FiroeWithBraneMind original, FIR newParent) {
        super(original.ast(), original.comment);
        setParentFir(newParent);

        if (!original.braneMind.isEmpty()) {
            throw new IllegalStateException(formatErrorMessage(
                "cloneConstanic requires empty braneMind, but found " + original.braneMind.size() + " items"));
        }

        this.braneMind = new LinkedList<>();
        this.braneMemory = new BraneMemory(null);

        int index = 0;
        for (FIR fir : original.braneMemory) {
            FIR clonedFir = fir.cloneConstanic(this, Optional.of(Nyes.INITIALIZED));
            this.braneMemory.put(clonedFir);
            this.indexLookup.put(clonedFir, index);

            if (clonedFir instanceof FiroeWithBraneMind fwbm) {
                fwbm.ordinated = false;
                fwbm.ordinateToParentBraneMind(this);
            }
            index++;
        }

        this.ordinated = original.ordinated;
        setInitialized();
    }

    static FiroeWithBraneMind ofExpr(AST.Expr... tasks) {
        return of(List.of(tasks).stream().map(FIR::createFiroeFromExpr).toArray(FIR[]::new));
    }

    static FiroeWithBraneMind of(FIR... tasks) {
        FiroeWithBraneMind result = new FiroeWithBraneMind((AST) null, null) {
            @Override
            protected void initialize() {
                setInitialized();
            }
        };
        for (FIR task : tasks) {
            result.enqueueFirs(task);
        }
        return result;
    }

    protected void storeSubfirOfExprs(AST.Expr... tasks) {
        storeFirs(ofExpr(tasks));
    }

    @Deprecated
    protected void enqueueSubfirOfExprs(AST.Expr... tasks) {
        enqueueFirs(ofExpr(tasks));
    }

    protected abstract void initialize();

    protected void prime() {
        for (FIR fir : braneMemory) {
            if (!fir.isConstanic()) {
                braneMind.add(fir);
            }
        }
    }

    protected void storeFirs(FIR... firs) {
        for (FIR fir : firs) {
            braneMemory.put(fir);
            fir.setParentFir(this);
            int index = braneMemory.size() - 1;
            indexLookup.put(fir, index);
            if (fir instanceof FiroeWithBraneMind fwbm && !fwbm.ordinated) {
                fwbm.ordinateToParentBraneMind(this);
            }
        }
    }

    @Deprecated
    protected void enqueueFirs(FIR... firs) {
        for (FIR fir : firs) {
            braneMind.addLast(fir);
            braneMemory.put(fir);
            fir.setParentFir(this);
            int index = braneMind.size() - 1;
            indexLookup.put(fir, index);
            if (fir instanceof FiroeWithBraneMind fwbm && !fwbm.ordinated) {
                fwbm.ordinateToParentBraneMind(this);
            }
        }
    }

    protected int getIndexOf(FIR f) {
        Integer idx = (Integer) indexLookup.get(f);
        return idx != null ? idx : -1;
    }

    protected void storeExprs(AST.Expr... exprs) {
        for (AST.Expr expr : exprs) {
            storeFirs(FIR.createFiroeFromExpr(expr));
        }
    }

    @Deprecated
    protected void enqueueExprs(AST.Expr... exprs) {
        for (AST.Expr expr : exprs) {
            enqueueFirs(FIR.createFiroeFromExpr(expr));
        }
    }

    protected boolean isBrane(FIR fir) {
        return fir instanceof BraneFiroe;
    }

    /**
     * Main evaluation loop with state machine transitions.
     * @see "projects/FIR-Invariances.md#State Machine"
     */
    public int step() {
        return switch (getNyes()) {
            case UNINITIALIZED -> {
                initialize();
                setNyes(Nyes.INITIALIZED);
                yield 1;
            }
            case INITIALIZED -> {
                if (stepNonBranesUntilState(Nyes.CHECKED)) {
                    setNyes(Nyes.CHECKED);
                }
                yield 1;
            }
            case CHECKED -> {
                prime();
                setNyes(Nyes.PRIMED);
                yield 1;
            }
            case PRIMED -> {
                setNyes(Nyes.EVALUATING);
                yield 1;
            }
            case EVALUATING -> {
                if (isBraneEmpty()) {
                    boolean anyConstanic = braneMemory.stream().anyMatch(FIR::atConstanic);
                    setNyes(anyConstanic ? Nyes.CONSTANIC : Nyes.CONSTANT);
                    yield 1;
                }
                FIR current = braneMind.removeFirst();
                try {
                    int work = current.step();
                    if (current.isNye()) {
                        braneMind.addLast(current);
                    }
                    yield work;
                } catch (Exception e) {
                    braneMind.addFirst(current);
                    org.foolish.fvm.AlarmSystem.raise(braneMemory, "Error during braneMind step: " + e.getMessage(), org.foolish.fvm.AlarmSystem.PANIC);
                    throw new RuntimeException("Error during braneMind step", e);
                }
            }
            case CONSTANIC, CONSTANT -> 0;
        };
    }

    private boolean stepNonBranesUntilState(Nyes targetState) {
        if (braneMind.isEmpty() || allNonBranesReachedState(targetState)) {
            return true;
        }
        FIR current = braneDequeue();
        try {
            current.step();
            if (current.isNye()) {
                braneMind.addLast(current);
            }
            if (current.getNyes().ordinal() < targetState.ordinal()) {
                return false;
            }
        } catch (Exception e) {
            braneMind.addFirst(current);
            org.foolish.fvm.AlarmSystem.raise(braneMemory, "Error during braneMind step: " + e.getMessage(), org.foolish.fvm.AlarmSystem.PANIC);
            throw new RuntimeException("Error during braneMind step", e);
        }
        return allNonBranesReachedState(targetState);
    }

    private boolean allNonBranesReachedState(Nyes targetState) {
        if (isBraneEmpty()) {
            return true;
        }
        FIR current = branePeek();
        int seen = 1;
        while (isBrane(current) || current.getNyes().ordinal() >= targetState.ordinal()) {
            if (seen++ > braneSize()) {
                return true;
            }
            braneEnqueue(braneDequeue());
            current = branePeek();
        }
        return false;
    }

    public Stream<FIR> stream() {
        return braneMemory.stream();
    }

    public int getStatementIndex(FIR fir) {
        return getIndexOf(fir);
    }

    // ========== ACCESSOR METHODS ==========

    protected void braneEnqueue(FIR fir) {
        braneMind.addLast(fir);
    }

    protected void braneEnqueueFirst(FIR fir) {
        braneMind.addFirst(fir);
    }

    protected FIR braneDequeue() {
        return braneMind.removeFirst();
    }

    protected FIR branePeek() {
        return braneMind.getFirst();
    }

    protected boolean isBraneEmpty() {
        return braneMind.isEmpty();
    }

    protected int braneSize() {
        return braneMind.size();
    }

    protected FIR memoryGet(int index) {
        return braneMemory.get(index);
    }

    protected boolean isMemoryEmpty() {
        return braneMemory.isEmpty();
    }

    protected FIR memoryGetLast() {
        return braneMemory.getLast();
    }

    public int memorySize() {
        return braneMemory.size();
    }

    protected Optional<Pair<Integer, FIR>> memoryGet(Query query, int fromLine) {
        return braneMemory.get(query, fromLine);
    }

    protected void linkMemoryParent(FiroeWithBraneMind parent) {
        braneMemory.setParentBrane(parent);
    }

    protected void setMemoryOwner(FiroeWithBraneMind owner) {
        braneMemory.setOwningBrane(owner);
    }

    public ReadOnlyBraneMemory getBraneMemory() {
        return braneMemory;
    }

    public FIR getMemoryItem(int index) {
        return braneMemory.get(index);
    }

    // ========== PACKAGE-PRIVATE TEST ACCESSORS ==========

    int getBraneMindSize() {
        return braneMind.size();
    }

    boolean isBraneMindEmpty() {
        return braneMind.isEmpty();
    }

    FIR peekBraneMind() {
        return braneMind.isEmpty() ? null : braneMind.getFirst();
    }

    @Override
    protected FIR clone() {
        return (FiroeWithBraneMind) super.clone();
    }

    @Override
    protected FIR cloneConstanic(FIR newParent, java.util.Optional<Nyes> targetNyes) {
        if (!isConstanic()) {
            throw new IllegalStateException(formatErrorMessage(
                "cloneConstanic can only be called on CONSTANIC or CONSTANT FIRs, but this FIR is in state: " + getNyes()));
        }
        if (isConstant()) {
            return this;
        }
        FiroeWithBraneMind copy = new FiroeWithBraneMind(this, newParent) {
            @Override
            protected void initialize() {
                setInitialized();
            }
        };
        copy.nyes = targetNyes.orElse(this.nyes);
        return copy;
    }
}
