package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * IfFiroe represents an if-else expression in the UBC system.
 * Contains a series of Firoes representing conditions and values.
 */

public class IfFiroe extends FiroeWithBraneMind implements Constanicable {

    private FIR result;

    public IfFiroe(AST.IfExpr ifExpr) {
        super(ifExpr);
        this.result = null;
    }

    protected int nextPossibleIdx;

    protected void initialize() {
        if (isInitialized()) return;

        AST.IfExpr ifExpr = (AST.IfExpr) ast;
        storeSubfirOfExprs(ifExpr);

        // Create Firoes for condition, then, and else branches
        storeSubfirOfExprs(ifExpr.condition(), ifExpr.thenExpr());
        for (AST.IfExpr elseIf : ifExpr.elseIfs()) {
            storeFirs(new ConditionalFiroe(elseIf));
        }

        // Enqueue else branch - if not present (UnknownExpr), use NK (???)
        if (ifExpr.elseExpr() == AST.UnknownExpr.INSTANCE || ifExpr.elseExpr() == null) {
            // No explicit else branch - add implicit else ???
            storeFirs(new NKFiroe("No matching condition in if-elif chain"));
        } else {
            storeFirs(createFiroeFromExpr(ifExpr.elseExpr()));
        }

        nextPossibleIdx=0;

        setInitialized();
    }

    public int step() {
        if (!isInitialized()) {
            initialize();
            return 1;
        }

        if (result != null) {
            // Already done
            return 0;
        }

        switch (memoryGet(nextPossibleIdx)) {
            case ConditionalFiroe cfiroe -> {
                if (cfiroe.hasTrueCondition()) {
                    step();
                    FIR thenFir=cfiroe.getThenFir();
                    if(!thenFir.isNye()){
                        result = cfiroe.getThenFir();
                    }
                    return 1;
                }else if(cfiroe.hasFalseCondition()){
                    nextPossibleIdx+=1;
                    return 1;
                }else{
                    // condition is nye, need to step further
                    cfiroe.step();
                    return 1;
                }
            }
            case FIR firoe -> {
                /* Else branch (explicitly provided or implicit ???) has been fully evaluated */
                nextPossibleIdx = memorySize()-1; // Choose the else branch
                result = firoe;
                super.step();
                return 1;
            }
        }
    }


    /**
     * Get the result of the if expression.
     */
    public FIR getResult() {
        return result;
    }


    // This class is private to ensure that nothing else can insert
    // this class into the "else branch"
    private class ConditionalFiroe extends FiroeWithBraneMind {

        Boolean condition_value = null;

        protected ConditionalFiroe(AST.IfExpr ifExpr) {
            super(ifExpr);
            storeSubfirOfExprs(ifExpr.condition(), ifExpr.thenExpr());
        }

        protected void initialize() {
            setInitialized();
            // ConditionalFiroe handles initialization in constructor
        }

        public int step() {
            int work = super.step();
            // After stepping, check if condition has been evaluated
            if (condition_value == null && !isMemoryEmpty() && !memoryGet(0).isNye()) {
                // Condition is evaluated, get its value
                FIR conditionFir = memoryGet(0);
                if (!conditionFir.isConstanic()) {
                    long condValue = conditionFir.getValue();
                    condition_value = (condValue != 0);
                } else {
                    // Condition is NK, treat as false
                    condition_value = false;
                }
            }
            return work;
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
            return memoryGetLast();
        }
    }

}
