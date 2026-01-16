package org.foolish.fvm.ubc;

import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.foolish.UbcTester;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.ast.ASTFormatter;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;

import java.util.ArrayList;
import java.util.List;

/**
 * Java implementation of the UBC tester.
 * Produces complete .approved.foo files with input code and test results.
 */
public class JavaUbcInterpreter implements UbcTester {

    @Override
    public String execute(String code) {
        // Parse the code
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        AST.Program program = (AST.Program) new ASTBuilder().visit(parser.program());

        // Extract branes and handle concatenation
        List<AST.Characterizable> branes = program.branes().branes();

        // Handle detachment brane concatenation
        UnicelluarBraneComputer ubc;
        BraneFiroe finalResult;
        int stepCount;

        if (branes.size() == 2 &&
            branes.get(0) instanceof AST.DetachmentBrane detachment &&
            branes.get(1) instanceof AST.Brane targetBrane) {
            // This is detachment brane concatenation: [x, y]{...}
            // We need to apply the detachment to the target brane
            ubc = createDetachedBraneUbc(detachment, targetBrane);
            stepCount = ubc.runToCompletion();
            finalResult = ubc.getRootBrane();
        } else if (branes.size() == 1 && branes.get(0) instanceof AST.Brane brane) {
            // Single regular brane
            ubc = new UnicelluarBraneComputer(brane);
            stepCount = ubc.runToCompletion();
            finalResult = ubc.getRootBrane();
        } else {
            // For other cases, just use the first brane
            // TODO: Handle other concatenation cases
            AST.Brane brane = (AST.Brane) branes.get(0);
            ubc = new UnicelluarBraneComputer(brane);
            stepCount = ubc.runToCompletion();
            finalResult = ubc.getRootBrane();
        }

        // Format as complete .foo file
        StringBuilder output = new StringBuilder();
        output.append("!!INPUT!!\n");
        output.append(code).append("\n\n");
        output.append("!!!\n");

        output.append("PARSED AST:\n");
        output.append(new ASTFormatter().format(program)).append("\n\n");

        output.append("UBC EVALUATION:\n");
        output.append("Steps taken: ").append(stepCount).append("\n\n");

        output.append("FINAL RESULT:\n");
        output.append(new Sequencer4Human().sequence(finalResult)).append("\n\n");

        output.append("COMPLETION STATUS:\n");
        output.append("Complete: ").append(ubc.isComplete());

        output.append("\n!!!\n");

        return output.toString();
    }

    /**
     * Creates a UBC for a detached brane.
     * This applies the detachment brane to the target brane, blocking identifiers from parent resolution.
     */
    private UnicelluarBraneComputer createDetachedBraneUbc(AST.DetachmentBrane detachment, AST.Brane targetBrane) {
        // Create a wrapper brane that includes the detachment logic
        // The detachment should block the specified identifiers from parent resolution

        // Create UBC for the target brane
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(targetBrane);

        // Get the root brane FIR
        BraneFiroe rootBrane = ubc.getRootBrane();

        // Create a detachment FIR and apply it to the root brane
        DetachmentFiroe detachmentFir = new DetachmentFiroe(detachment);

        // Initialize the detachment FIR
        detachmentFir.step(); // Initialize

        // Apply the detachment to the root brane
        detachmentFir.applyDetachmentTo(rootBrane);

        return ubc;
    }

    @Override
    public String getName() {
        return "Java UBC";
    }
}
