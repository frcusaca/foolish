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
import java.util.stream.Collectors;

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
     * <b>Filter Chain Semantics (Right-to-Left Evaluation):</b>
     * <p>
     * For {@code [d1][d2][d3]{B}} searching for identifier {@code v}:
     * <ol>
     * <li>Search finds match in parent scope</li>
     * <li>Apply d3 filter (rightmost): does d3 block v?</li>
     * <li>Apply d2 filter: does d2 override d3's decision?</li>
     * <li>Apply d1 filter (leftmost): d1 has final say</li>
     * <li>If undetached after all filters → use match</li>
     * <li>If detached → identifier is blocked</li>
     * </ol>
     * <p>
     * IMPORTANT: Cannot merge filters (except identical exact matches).
     * The sequence must be preserved and applied as a filter chain.
     * <p>
     * Each detachment brane becomes one stage in the filter chain.
     * Default values are determined by the leftmost filter that blocks the identifier.
     *
     * @param detachmentChain List of detachment branes (left-to-right order as in code)
     * @param targetBrane The regular brane to apply detachment chain to
     * @return UBC with detachment filter chain applied
     */
    private UnicelluarBraneComputer createDetachedBraneUbc(List<AST.DetachmentBrane> detachmentChain, AST.Brane targetBrane) {
        // Create UBC for the target brane
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(targetBrane);

        // Get the root brane FIR
        BraneFiroe rootBrane = ubc.getRootBrane();

        // Build the filter chain: each detachment brane becomes one filter stage
        List<List<Query>> filterChain = new ArrayList<>();

        for (AST.DetachmentBrane detachment : detachmentChain) {
            DetachmentFiroe detachmentFir = new DetachmentFiroe(detachment);
            detachmentFir.step(); // Initialize

            // Extract queries for this filter stage
            List<Query> filterStage = detachmentFir.getBlockedIdentifiers().stream()
                    .map(id -> new Query.StrictlyMatchingQuery(id.getId(), id.getCharacterization()))
                    .collect(Collectors.toList());

            filterChain.add(filterStage);
        }

        // Apply the filter chain to the root brane
        rootBrane.braneMemory.setDetachmentFilterChain(filterChain);

        // Apply default values from the leftmost detachment that provides them
        // (leftmost wins for defaults)
        for (AST.DetachmentBrane detachment : detachmentChain) {
            DetachmentFiroe detachmentFir = new DetachmentFiroe(detachment);
            detachmentFir.step(); // Initialize
            detachmentFir.applyDetachmentTo(rootBrane);
            break; // Only apply defaults from first (leftmost) detachment for now
            // TODO: Properly handle default override semantics
        }

        return ubc;
    }

    @Override
    public String getName() {
        return "Java UBC";
    }
}
