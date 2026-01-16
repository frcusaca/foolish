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
    private final List<BraneMemory.Rule> initialRules;
    private BraneFiroe predecessor;

    public BraneFiroe(AST ast) {
        this(ast, new ArrayList<>(), new ArrayList<>());
    }

    public BraneFiroe(AST ast, List<FIR> initialFirs, List<BraneMemory.Rule> initialRules) {
        super(ast);
        this.initialFirs = initialFirs;
        this.initialRules = initialRules;
        this.predecessor = null;
    }

    // Constructor for legacy compatibility (if needed)
    // Note: Java generics erasure might cause clash if we keep signature same.
    // So we avoid it or cast carefully.

    public void addRules(List<BraneMemory.Rule> rules) {
        // Append new rules to the initial rules list to ensure correct priority when added to memory
        // (Since memory.addRule prepends, adding L-to-R items to memory results in R-to-L lookup priority.
        // Wait, memory list is [LastAdded, ... FirstAdded].
        // Lookup checks 0..N (LastAdded first).
        // R-to-L iteration in createFiroeFromBranes:
        // 1. [b]. rules=[b]. Memory: [b].
        // 2. [a]. rules=[b, a]. Memory: [b, a, b]? No.
        // initialize() iterates rules.
        // If initialRules is [b, a].
        // initialize:
        //   addRule(b) -> Memory [b].
        //   addRule(a) -> Memory [a, b].
        // Lookup checks `a` then `b`.
        // Left (a) overrides Right (b). Correct.
        // So we want `initialRules` to be `[b, a]` (Right then Left).
        // createFiroeFromBranes iterates R-to-L.
        // 1. [b]. `acc` has `initialRules` = `[b]`.
        // 2. [a]. We add `a`.
        // If we Append: `initialRules` = `[b, a]`.
        // Correct.
        this.initialRules.addAll(rules);
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

        // Add initial rules
        for (BraneMemory.Rule rule : initialRules) {
            braneMemory.addRule(rule.query(), rule.action());
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
            // Fall through to allow stepping predecessor if needed?
            // Usually initialize() is enough for one step.
            // But we should check predecessor.
        }

        if (predecessor != null && predecessor.isNye()) {
            // Step predecessor
            if (!predecessor.isInitialized()) {
                // Predecessor needs to be initialized.
                // It should have been ordinated via ordinateToParentBraneMind call on 'this'.
                // So it's safe to step.
            }
            predecessor.step();
            // We can continue to step 'this' (pipeline) or wait?
            // "Sequential execution": {x} then {y}.
            // While {x} is running, {y} can run?
            // {y} might depend on {x}. {y} searches {x}.
            // If {x} hasn't allocated vars, {y} search might fail or return NK.
            // But searches are dynamic.
            // Let's assume we can step both.
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
