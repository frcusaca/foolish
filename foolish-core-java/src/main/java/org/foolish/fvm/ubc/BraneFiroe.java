package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;

/**
 * BraneFiroe represents a brane in the UBC system.
 * The BraneFiroe processes its AST to create Expression Firoes,
 * which are enqueued into the braneMind and evaluated breadth-first.
 */
public class BraneFiroe extends FiroeWithBraneMind {
    private final List<FIR> initialFirs;
    private final List<BraneMemory.QueryModification> initialQueryMods;
    private BraneFiroe predecessor;

    public BraneFiroe(AST ast) {
        this(ast, new ArrayList<>(), new ArrayList<>());
    }

    public BraneFiroe(AST ast, List<FIR> initialFirs, List<BraneMemory.QueryModification> initialQueryMods) {
        super(ast);
        this.initialFirs = initialFirs;
        this.initialQueryMods = initialQueryMods;
        this.predecessor = null;
    }

    /**
     * Adds query modifications (rules) to this brane.
     * New rules are appended to the list to maintain correct priority during initialization.
     * Since processing happens Right-to-Left, we append the new (Left-most) rules so they are processed
     * last during initialize(), and thus prepended to the front of BraneMemory list (highest priority).
     */
    public void addQueryModifications(List<BraneMemory.QueryModification> mods) {
        this.initialQueryMods.addAll(mods);
    }

    public void prepend(BraneFiroe prev) {
        if (this.predecessor != null) {
            this.predecessor.prepend(prev);
        } else {
            this.predecessor = prev;
        }
    }

    @Override
    public void ordinateToParentBraneMind(FiroeWithBraneMind parent, int myPos) {
        if (predecessor != null) {
            // Ordinate predecessor to the actual parent
            predecessor.ordinateToParentBraneMind(parent, myPos);
            // This brane's parent memory becomes the predecessor's memory
            this.braneMemory.setParent(predecessor.braneMemory);
            this.braneMemory.setMyPos(myPos); // We still use myPos? Or predecessor's pos?
            // Since we share the position effectively (concatenated), it's fine.
            this.ordinated = true;
        } else {
            super.ordinateToParentBraneMind(parent, myPos);
        }
    }

    /**
     * Initialize the BraneFiroe by converting AST statements to Expression Firoes.
     */
    @Override
    protected void initialize() {
        if (isInitialized()) return;
        setInitialized();

        // Add initial query modifications
        for (BraneMemory.QueryModification mod : initialQueryMods) {
            braneMemory.addQueryModification(mod.query(), mod.modification());
        }

        // Enqueue initial FIRs
        if (initialFirs != null) {
            for (FIR fir : initialFirs) {
                enqueueFirs(fir);
            }
        }

        if (ast instanceof AST.Brane brane) {
            for (AST.Expr expr : brane.statements()) {
                FIR firoe = createFiroeFromExpr(expr);
                enqueueFirs(firoe);
            }
        } else if (ast == null) {
            // No AST, purely synthetic brane (initialized via initialFirs)
        } else {
            throw new IllegalArgumentException("AST must be of type AST.Brane or null for synthetic branes");
        }
    }

    @Override
    public boolean isNye() {
        if (!isInitialized()) return true;
        if (predecessor != null && predecessor.isNye()) return true;
        return super.isNye();
    }

    @Override
    public void step() {
        if (!isInitialized()) {
            initialize();
        }

        if (predecessor != null && predecessor.isNye()) {
            predecessor.step();
        }

        super.step();
    }

    /**
     * Returns the list of expression Firoes in this brane.
     * Includes both completed (in braneMemory) and pending (in braneMind) FIRs.
     */
    public List<FIR> getExpressionFiroes() {
        List<FIR> allFiroes = new ArrayList<>();
        braneMemory.forEach(allFiroes::add);
        return allFiroes;
    }

    @Override
    public String toString() {
        return new Sequencer4Human().sequence(this);
    }
}
