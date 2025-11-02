package org.foolish.ubc;

import org.foolish.ast.AST;

import java.util.ArrayList;
import java.util.List;

/**
 * IfFiroe represents an if-else expression in the UBC system.
 * Contains a series of Firoes representing conditions and values.
 */
public class IfFiroe extends FiroeWithBraneMind {
    private FIR conditionFiroe;
    private FIR thenFiroe;
    private FIR elseFiroe;
    private List<IfFiroe> elseIfFiroes;
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

        // Create Firoes for condition, then, and else branches
        conditionFiroe = createFiroeFromExpr(ifExpr.condition());
        thenFiroe = createFiroeFromExpr(ifExpr.thenExpr());
        elseFiroe = createFiroeFromExpr(ifExpr.elseExpr());

        // Create else-if Firoes
        elseIfFiroes = new ArrayList<>();
        for (AST.IfExpr elseIf : ifExpr.elseIfs()) {
            elseIfFiroes.add(new IfFiroe(elseIf));
        }

        // Enqueue condition for evaluation first
        if (conditionFiroe.underevaluated()) {
            braneMind.offer(conditionFiroe);
        }
    }

    @Override
    public void step() {
        if (result != null) {
            return;
        }

        if (!initialized) {
            initialize();
            return;
        }

        // Step 1: Evaluate the condition
        if (conditionFiroe.underevaluated()) {
            if (conditionFiroe instanceof FiroeWithBraneMind firoeWithMind) {
                firoeWithMind.step();
                if (!conditionFiroe.underevaluated()) {
                    braneMind.poll();
                }
            }
            return;
        }

        // Step 2: Based on condition, evaluate the appropriate branch
        if (conditionFiroe instanceof ValueFiroe conditionValue) {
            boolean conditionIsTrue = conditionValue.getValue() != 0;

            if (conditionIsTrue) {
                // Evaluate then branch
                if (thenFiroe.underevaluated()) {
                    if (thenFiroe instanceof FiroeWithBraneMind firoeWithMind) {
                        firoeWithMind.step();
                    }
                } else {
                    result = thenFiroe;
                }
            } else {
                // Check else-if branches
                boolean elseIfMatched = false;
                for (IfFiroe elseIfFiroe : elseIfFiroes) {
                    if (elseIfFiroe.underevaluated()) {
                        elseIfFiroe.step();
                        if (!elseIfFiroe.underevaluated()) {
                            result = elseIfFiroe.getResult();
                            elseIfMatched = true;
                            break;
                        }
                    }
                }

                if (!elseIfMatched && result == null) {
                    // Evaluate else branch
                    if (elseFiroe.underevaluated()) {
                        if (elseFiroe instanceof FiroeWithBraneMind firoeWithMind) {
                            firoeWithMind.step();
                        }
                    } else {
                        result = elseFiroe;
                    }
                }
            }
        }
    }

    @Override
    public boolean underevaluated() {
        return result == null;
    }

    /**
     * Get the result of the if expression.
     */
    public FIR getResult() {
        return result;
    }

    /**
     * Get the value if the result is a ValueFiroe.
     */
    @Override
    public long getValue() {
        if (result instanceof ValueFiroe valueFiroe) {
            return valueFiroe.getValue();
        }
        throw new IllegalStateException("IfFiroe result is not a ValueFiroe");
    }

    @Override
    public boolean isAbstract() {
        return (conditionFiroe != null && conditionFiroe.isAbstract()) ||
               (thenFiroe != null && thenFiroe.isAbstract()) ||
               (elseFiroe != null && elseFiroe.isAbstract());
    }

    /**
     * Creates a FIR from an AST expression.
     */
    private FIR createFiroeFromExpr(AST.Expr expr) {
        if (expr instanceof AST.IntegerLiteral literal) {
            return new ValueFiroe(expr, literal.value());
        } else if (expr instanceof AST.BinaryExpr binary) {
            return new BinaryFiroe(binary);
        } else if (expr instanceof AST.UnaryExpr unary) {
            return new UnaryFiroe(unary);
        } else if (expr instanceof AST.IfExpr ifExpr) {
            return new IfFiroe(ifExpr);
        } else if (expr instanceof AST.Brane brane) {
            return new BraneFiroe(brane);
        } else {
            // Placeholder for unsupported types
            return new ValueFiroe(0L);
        }
    }
}
