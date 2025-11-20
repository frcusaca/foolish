package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.fvm.Env;

import java.util.ArrayList;
import java.util.List;

/**
 * BraneFiroe represents a brane in the UBC system.
 * The BraneFiroe processes its AST to create Expression Firoes,
 * which are enqueued into the braneMind and evaluated breadth-first.
 */
public class BraneFiroe extends FiroeWithBraneMind {
    private final Env environment;

    /**
     * Constructs a BraneFiroe with the given AST and Ancestral Brane environment.
     * The ABEnv is used for resolving identifiers from ancestral branes.
     */
    public BraneFiroe(AST ast, Env aBEnv) {
        super(ast);
        this.environment = aBEnv;
    }

    public BraneFiroe(AST ast) {
        this(ast, null);
    }

    /**
     * Initialize the BraneFiroe by converting AST statements to Expression Firoes.
     */
    @Override
    protected void initialize() {
        if (isInitialized()) return;
        setInitialized();

        if (ast instanceof AST.Brane brane) {
            for (AST.Expr expr : brane.statements()) {
                FIR firoe = createFiroeFromExpr(expr);
                enqueueFirs(firoe);
            }
        }else{
            throw new IllegalArgumentException("AST must be of type AST.Brane");
        }
    }

    @Override
    public boolean isNye() {
        return !isInitialized() || super.isNye();
    }

    @Override
    public void step() {
        if (!isInitialized()) {
            initialize();
            return;
        }

        super.step();
    }

    /**
     * Returns the frozen environment after full evaluation.
     * This is the value of a fully evaluated BraneFiroe.
     */
    @Override
    public Env getEnvironment() {
        if (isNye()) {
            throw new IllegalStateException("BraneFiroe not fully evaluated");
        }
        return environment;
    }


    /**
     * Returns the list of expression Firoes in this brane.
     * Includes both completed (in braneMemory) and pending (in braneMind) FIRs.
     */
    public List<FIR> getExpressionFiroes() {
        List<FIR> allFiroes = new ArrayList<>();
        allFiroes.addAll(braneMemory);
        return allFiroes;
    }


    @Override
    public String toString() {
        return new Sequencer4Human().sequence(this);
    }


    public BraneFiroe cloneWithABEnv(Env newABEnv) {
        return new BraneFiroe(this.ast, newABEnv==null?this.environment:newABEnv);
    }

    public BraneFiroe cloneAbstract() {
        return cloneWithABEnv(null);
    }
}
