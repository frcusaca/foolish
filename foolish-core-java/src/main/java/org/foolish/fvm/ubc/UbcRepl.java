package org.foolish.fvm.ubc;

import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;

/**
 * Simple read-eval-print loop for the Foolish language using the UBC.
 * This REPL uses the Unicellular Brane Computer for evaluation.
 *
 * <h2>Usage</h2>
 * <p>Run the REPL interactively:
 * <pre>
 * mvn exec:java -Dexec.mainClass="org.foolish.fvm.ubc.UbcRepl"
 * </pre>
 *
 * <p>Or compile and run:
 * <pre>
 * mvn verify -DskipTests
 * java -cp target/classes:target/dependency/* org.foolish.fvm.ubc.UbcRepl
 * </pre>
 *
 * <h2>Examples</h2>
 * <pre>
 * {@code
 * => {5;}
 * => {
 *   5;
 * }
 *
 * => {10 + 20;}
 * => {
 *   30;
 * }
 *
 * => {(5 + 3) * 2;}
 * => {
 *   16;
 * }
 *
 * => {1; 2; 3 + 4;}
 * => {
 *   1;
 *   2;
 *   7;
 * }
 * }
 * </pre>
 *
 * <h2>Differences from FVM REPL</h2>
 * <ul>
 *   <li>Uses {@link UnicelluarBraneComputer} instead of {@link org.foolish.fvm.v1.FiroeVm}</li>
 *   <li>Implements step-by-step breadth-first evaluation</li>
 *   <li>Returns the whole evaluated brane (not just last value)</li>
 *   <li>Provides detailed error messages with optional --debug flag</li>
 * </ul>
 */
public class UbcRepl {

    /**
     * Parse the provided source into an AST program.
     */
    public static AST.Program parse(String source) {
        CharStream input = CharStreams.fromString(source);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        parser.removeErrorListeners();
        parser.addErrorListener(new org.antlr.v4.runtime.ConsoleErrorListener());
        return (AST.Program) new ASTBuilder().visitProgram(parser.program());
    }

    /**
     * Evaluate the given source using UBC, returning the result.
     */
    public static Object eval(String source) {
        AST.Program ast = parse(source);

        // Extract the brane from the program
        AST.Branes branes = ast.branes();
        if (branes == null || branes.branes().isEmpty()) {
            return null;
        }

        // Get the first brane to evaluate
        AST.Characterizable firstBrane = branes.branes().get(0);
        if (!(firstBrane instanceof AST.Brane brane)) {
            return null;
        }

        // Create UBC and evaluate
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(brane);

        // Run to completion
        ubc.runToCompletion();

        // Return the whole BraneFiroe (the evaluated brane)
        return ubc.getRootBrane();
    }

    public static void main(String[] args) throws IOException {
        System.out.println("Foolish UBC REPL");
        System.out.println("Using Unicellular Brane Computer");
        System.out.println("Type Foolish expressions (Ctrl+D to exit)");
        System.out.println();

        BufferedReader reader = new BufferedReader(new InputStreamReader(System.in));
        String line;

        while ((line = reader.readLine()) != null) {
            if (line.isBlank()) continue;

            try {
                Object result = eval(line);
                if (result != null) {
                    System.out.println("=> " + result);
                }
            } catch (Exception e) {
                org.foolish.fvm.AlarmSystem.raise(null, "REPL Error: " + e.getMessage(), org.foolish.fvm.AlarmSystem.PANIC);
                if (args.length > 0 && args[0].equals("--debug")) {
                    e.printStackTrace();
                }
            }
        }
    }
}
