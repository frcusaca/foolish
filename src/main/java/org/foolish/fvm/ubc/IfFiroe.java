package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * IfFiroe represents an if-else expression in the UBC system.
 * Contains a series of Firoes representing conditions and values.
 */

public class IfFiroe extends FiroeWithBraneMind {

    private FIR result;

    public IfFiroe(AST.IfExpr ifExpr) {
        super(ifExpr);
        this.result = null;
    }

    protected int nextPossibleIdx;

    @Override
    protected void initialize() {
        if (isInitialized()) return;

        AST.IfExpr ifExpr = (AST.IfExpr) ast;
        enqueueSubfirOfExprs(ifExpr);

        // Create Firoes for condition, then, and else branches
        enqueueSubfirOfExprs(ifExpr.condition(), ifExpr.thenExpr());
        for (AST.IfExpr elseIf : ifExpr.elseIfs()) {
            enqueueFirs(new ConditionalFiroe(elseIf));
        }

        // Enqueue condition for evaluation first
        enqueueFirs(createFiroeFromExpr(ifExpr.elseExpr()));

        nextPossibleIdx=0;

        setInitialized();
    }

    @Override
    public void step() {
        if (!isInitialized()) {
            initialize();
            return;
        }

        if (result != null) {
            // Already done
            return;
        }

        switch (braneMemory.get(nextPossibleIdx)) {
            case ConditionalFiroe cfiroe -> {
                if (cfiroe.hasTrueCondition()) {
                    step();
                    FIR thenFir=cfiroe.getThenFir();
                    if(!thenFir.isNye()){
                        result = cfiroe.getThenFir();
                    }
                }else if(cfiroe.hasFalseCondition()){
                    nextPossibleIdx+=1;
                }else{
                    // condition is nye, need to step further
                    cfiroe.step();
                }
            }
            case FIR firoe -> {
                /*else was ealuated fully already*/
                // TODO: Make sure there is always an else branch
                nextPossibleIdx = braneMemory.size()-1; // Choose the else branch
                result = firoe;
                super.step();
            }
        }
    }

    @Override
    public boolean isNye() {
        return nextPossibleIdx < braneMemory.size() || braneMemory.getLast().isNye();
    }

    /**
     * Get the result of the if expression.
     */
    public FIR getResult() {
        return result;
    }


    // This class is private to ensure that nothign else can insert
    // this class into the "else branch"
    private class ConditionalFiroe extends FiroeWithBraneMind {

        Boolean condition_value = null;

        protected ConditionalFiroe(AST.IfExpr ifExpr) {
            super(ifExpr);
            enqueueSubfirOfExprs(ifExpr.condition(), ifExpr.thenExpr());
        }

        @Override
        protected void initialize() {
            setInitialized();
            // ConditionalFiroe handles initialization in constructor
        }

        @Override
        public boolean isNye() {
            return hasUnknownCondition() || (hasTrueCondition() && super.isNye());
        }

        public boolean hasUnknownCondition() {
            return condition_value == null;
        }

        public boolean hasTrueCondition() {
            return condition_value == true;
        }

        public boolean hasFalseCondition() {
            return condition_value == false;
        }
        public FIR getThenFir(){
            return braneMemory.getLast();
        }
    }

}
