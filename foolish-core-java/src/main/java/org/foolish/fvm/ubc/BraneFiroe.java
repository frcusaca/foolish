package org.foolish.fvm.ubc;

import org.antlr.v4.runtime.BaseErrorListener;
import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.antlr.v4.runtime.RecognitionException;
import org.antlr.v4.runtime.Recognizer;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;

import java.util.ArrayList;
import java.util.List;

/**
 * BraneFiroe represents a brane in the UBC system.
 * The BraneFiroe processes its AST to create Expression Firoes,
 * which are enqueued into the braneMind and evaluated breadth-first.
 */
public class BraneFiroe extends FiroeWithBraneMind {

    public BraneFiroe(AST ast) {
        super(ast);
        // Register this brane as the owner of its memory
        this.braneMemory.setOwningBrane(this);
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

    // Removed isNye override

    @Override
    public int step() {
        if (!isInitialized()) {
            initialize();
            return 1;
        }

        return super.step();
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

    /**
     * Evaluates an expression within the context of this brane.
     * Useful for testing or manual introspection.
     *
     * @param exprString the Foolish expression to evaluate (e.g., "?id", "1+1")
     * @return the resulting FIR
     */
    public FIR search(String exprString) {
        // Handle optional '?' prefix for backward search if it's not part of the grammar
        // This supports the syntax "?id" by treating it as "id" which performs the lookup
        String parseString = exprString;
        if (exprString.startsWith("?")) {
            parseString = exprString.substring(1);
        }

        // Parse the expression
        CharStream input = CharStreams.fromString(parseString);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);

        parser.removeErrorListeners();
        parser.addErrorListener(new BaseErrorListener() {
            @Override
            public void syntaxError(Recognizer<?, ?> recognizer, Object offendingSymbol, int line, int charPositionInLine, String msg, RecognitionException e) {
                throw new IllegalArgumentException("Syntax error at " + line + ":" + charPositionInLine + ": " + msg);
            }
        });

        AST.Expr astExpr = (AST.Expr) new ASTBuilder().visitExpr(parser.expr());
        FIR fir = FIR.createFiroeFromExpr(astExpr);

        // Link the FIR to this brane's context
        fir.setParentFir(this);
        if (fir instanceof FiroeWithBraneMind fwbm) {
            // Coordinate with the parent's memory, appending to the end
            fwbm.ordinateToParentBraneMind(this, this.braneMemory.size());
        }

        // Execute until fully evaluated
        while (fir.isNye()) {
            fir.step();
        }

        return fir;
    }
}
