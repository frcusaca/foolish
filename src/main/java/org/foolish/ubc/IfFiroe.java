package org.foolish.ubc;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;
import org.apache.commons.lang3.tuple.ImmutablePair;

/**
 * IfFiroe represents an if-else expression in the UBC system.
 * Contains a series of Firoes representing conditions and values.
 */

public class IfFiroe extends FiroeWithBraneMind {

    // This class is private to ensure that nothign else can insert
    // this class into the "else branch"
    private class ConditionalFiroe extends FiroeWithBraneMind{

        Boolean condition_value=null;
        protected ConditionalFiroe(AST.IfExpr ifExpr) {
            super(ifExpr);
            enqueueFirOf(createFiroeFromExpr(ifExpr.condition()), createFiroeFromExpr(ifExpr.thenExpr()));
        }

        @Override
        public void  step(){
            if(braneMemory.isEmpty()){
                // first must evaluate condition
                super.step();
                if(!braneMemory.isEmpty()){
                    condition_value = braneMemory.peek().getValue()!=0;
                }
            }else if (condition_value == true){
                // when condition is known and it is decided this needs to be executed
                super.step();
            }
        }

        @Override
        public boolean underevaluated(){
            return hasUnknownCondition() || (hasTrueCondition() && super.underevaluated());
        }

         public boolean hasUnknownCondition(){
            return condition_value==null;
        }
        public boolean hasTrueCondition(){
            return condition_value==true;
        }
        public boolean hasFalseCondition(){
            return condition_value==false;
        }
    }

    private boolean initialized;
    private FIR result;

    public IfFiroe(AST.IfExpr ifExpr) {
        super(ifExpr);
        this.initialized = false;
        this.result = null;
    }

    private void initialize() {
        if (initialized) return;
        initialized = true;

        AST.IfExpr ifExpr = (AST.IfExpr) ast;
        enqueueFirOf(new ConditionalFiroe(ifExpr));

        // Create Firoes for condition, then, and else branches
        enqueueFirOf(createFiroeFromExpr(ifExpr.condition()), createFiroeFromExpr(ifExpr.thenExpr()));
        for (AST.IfExpr elseIf : ifExpr.elseIfs()) {
            enqueueFir(new ConditionalFiroe(elseIf));
        }

        // Enqueue condition for evaluation first
        enqueueFir(createFiroeFromExpr(ifExpr.elseExpr()));
    }

    @Override
    public void step() {
        if (!initialized){
            initialize();
            return;
        }
        switch(braneMemory.peekLast()){
            case null -> step();
            case ConditionalFiroe cfiroe -> {
                if (cfiroe.hasTrueCondition()) {
                    step();
                }
            }
            case FIR firoe -> {/*else was ealuated fully already*/}
        }
    }

    @Override
    public boolean underevaluated() {
        return switch (braneMemory.peekLast()) {
            case null -> true;
            case ConditionalFiroe cfiroe when (cfiroe.condition_value==true) ->
                    false;
            default ->
                    // This queue, when it contains something other than a conditionalFiroe means else.
                    true;
        };
    }

    /**
     * Get the result of the if expression.
     */
    public FIR getResult() {
        return switch (braneMemory.peekLast()) {
            case null ->
                throw new IllegalStateException("IfFiroe result requested before evaluation.");
            case ConditionalFiroe cfiroe when (cfiroe.condition_value==true) ->
                    cfiroe.braneMemory.getLast();
            case FIR elseResult ->
                elseResult;
        };
    }
    /**
     * Get the value if the result is a ValueFiroe.
     */
    @Override
    public long getValue() {
            return getResult().getValue();
    }

}
