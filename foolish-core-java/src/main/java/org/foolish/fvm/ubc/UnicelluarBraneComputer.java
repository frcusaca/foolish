package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Unicellular Brane Computer (UBC).
 * <p>
 * The UBC is a simple computational machine that has just enough capacity to hold
 * the AST of a brane and the ability to interpret and understand a single expression
 * at a time. It proceeds from the beginning of the brane to the end, evaluating and
 * creating new values.
 */
public class UnicelluarBraneComputer {
    private final BraneFiroe rootBrane;

    /**
     * Creates a UBC with a Brane AST.
     *
     * @param braneAst The brane AST
     */
    public UnicelluarBraneComputer(AST braneAst) {
        if (braneAst == null) {
            throw new IllegalArgumentException("Brane AST cannot be null");
        }

        if (!(braneAst instanceof AST.Brane)) {
            throw new IllegalArgumentException("AST must be a Brane");
        }

        BraneMemory standardLib = StandardConfiguration.createStandardLibrary();
        this.rootBrane = new BraneFiroe(braneAst);
        this.rootBrane.ordinateToParentBraneMind(FiroeWithBraneMind.of(), -1);
        // Manually set the parent of the rootBrane's memory to the standardLib
        // Since ordinateToParentBraneMind expects a Firoe parent, we might need a dummy parent wrapping the stdLib
        // or just expose setParent.
        // FiroeWithBraneMind.ordinateToParentBraneMind sets this.braneMemory.setParent(parent.braneMemory).
        // So we need a Firoe that holds standardLib.

        // Let's create a dummy Firoe for the Standard Library.
        FiroeWithBraneMind stdLibOwner = new FiroeWithBraneMind(null) {
            {
                this.braneMemory.setParent(standardLib);
                // Wait, standardLib IS the memory. We want rootBrane.memory.parent = standardLib.
                // But FiroeWithBraneMind.braneMemory is protected.
                // We can't access rootBrane.braneMemory directly from here unless we are in the same package (we are).
            }
            @Override
            protected void initialize() { setInitialized(); }
        };
        // Actually, since we are in the same package, we can access protected fields?
        // Yes, UnicelluarBraneComputer is in org.foolish.fvm.ubc.
        // BraneFiroe extends FiroeWithBraneMind.
        // FiroeWithBraneMind has `protected final BraneMemory braneMemory`.

        this.rootBrane.braneMemory.setParent(standardLib);
    }

    /**
     * Takes a single evaluation step.
     * Steps forward from the braneMind until it's empty, at which time it returns false.
     *
     * @return true if more steps are needed, false if evaluation is complete
     */
    public boolean step() {
        if (!rootBrane.isNye()) {
            return false;
        }

        rootBrane.step();
        return rootBrane.isNye();
    }

    /**
     * Runs the UBC until evaluation is complete.
     *
     * @return the number of steps taken
     */
    public int runToCompletion() {
        int steps = 0;
        while (step()) {
            steps++;

            // Safety check to prevent infinite loops
            if (steps > 100000) {
                throw new RuntimeException("Evaluation exceeded maximum step count (possible infinite loop)");
            }
        }
        return steps;
    }

    /**
     * Returns true if the UBC has completed evaluation.
     */
    public boolean isComplete() {
        return !rootBrane.isNye();
    }

    /**
     * Returns the root BraneFiroe being evaluated.
     */
    public BraneFiroe getRootBrane() {
        return rootBrane;
    }
}
