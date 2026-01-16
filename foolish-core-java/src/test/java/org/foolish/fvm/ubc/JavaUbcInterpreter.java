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

        // Handle detachment brane concatenation (including chains)
        UnicelluarBraneComputer ubc;
        BraneFiroe finalResult;
        int stepCount;

        // Collect all leading detachment branes (they combine left-to-right until hitting a regular brane)
        List<AST.DetachmentBrane> detachmentChain = new ArrayList<>();
        int firstBraneIndex = 0;

        for (int i = 0; i < branes.size(); i++) {
            if (branes.get(i) instanceof AST.DetachmentBrane detachment) {
                detachmentChain.add(detachment);
            } else {
                firstBraneIndex = i;
                break;
            }
        }

        // Check if we have detachment branes followed by a regular brane
        if (!detachmentChain.isEmpty() && firstBraneIndex < branes.size() &&
            branes.get(firstBraneIndex) instanceof AST.Brane targetBrane) {
            // Chain of detachment branes: [a][b][c]{...}
            // Combine them left-to-right, then apply to the brane
            ubc = createDetachedBraneUbc(detachmentChain, targetBrane);
            stepCount = ubc.runToCompletion();
            finalResult = ubc.getRootBrane();
        } else if (branes.size() == 1 && branes.get(0) instanceof AST.Brane brane) {
            // Single regular brane (no detachment)
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
     * Creates a UBC for a brane with detachment chain applied.
     * <p>
     * Handles multiple detachment branes that combine left-to-right before
     * applying to the target brane. Implements left-override semantics where
     * the leftmost detachment wins for conflicts.
     * <p>
     * Example: {@code [a=1][a=2][b=3]{...}} → combines to {@code [a=1, b=3]{...}}
     * where the left {@code a=1} overrides the right {@code a=2}.
     *
     * @param detachmentChain List of detachment branes to combine (left-to-right order)
     * @param targetBrane The regular brane to apply detachment to
     * @return UBC with combined detachment applied
     */
    private UnicelluarBraneComputer createDetachedBraneUbc(List<AST.DetachmentBrane> detachmentChain, AST.Brane targetBrane) {
        // Create UBC for the target brane
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(targetBrane);

        // Get the root brane FIR
        BraneFiroe rootBrane = ubc.getRootBrane();

        // Combine all detachment branes left-to-right
        DetachmentFiroe combined = null;
        for (AST.DetachmentBrane detachment : detachmentChain) {
            DetachmentFiroe current = new DetachmentFiroe(detachment);
            current.step(); // Initialize

            if (combined == null) {
                combined = current;
            } else {
                // Left (combined) overrides right (current)
                combined = combined.combineWith(current);
            }
        }

        // Apply the combined detachment to the root brane
        if (combined != null) {
            combined.applyDetachmentTo(rootBrane);
        }

        return ubc;
    }

    @Override
    public String getName() {
        return "Java UBC";
    }
}
