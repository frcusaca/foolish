package org.foolish.fvm.ubc;

import org.antlr.v4.runtime.*;
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

    /**
     * Custom error listener that collects parse errors for reporting.
     */
    private static class ParseErrorCollector extends BaseErrorListener {
        private final List<String> errors = new ArrayList<>();

        @Override
        public void syntaxError(Recognizer<?, ?> recognizer, Object offendingSymbol,
                              int line, int charPositionInLine,
                              String msg, RecognitionException e) {
            errors.add("line " + line + ":" + charPositionInLine + " " + msg);
        }

        public List<String> getErrors() {
            return errors;
        }

        public boolean hasErrors() {
            return !errors.isEmpty();
        }
    }

    @Override
    public String execute(String code) {
        return execute(code, "unknown.foo");
    }

    /**
     * Executes Foolish code with a specified filename for error reporting.
     *
     * @param code the Foolish code to execute
     * @param filename the source filename (e.g., "test.foo")
     * @return the formatted test output
     */
    public String execute(String code, String filename) {
        // Set up execution context for error reporting
        ExecutionContext.setCurrent(new ExecutionContext(filename));

        try {
            // Parse the code with error collection
            CharStream input = CharStreams.fromString(code);
            FoolishLexer lexer = new FoolishLexer(input);
            CommonTokenStream tokens = new CommonTokenStream(lexer);
            FoolishParser parser = new FoolishParser(tokens);

        // Add custom error listener
        ParseErrorCollector errorCollector = new ParseErrorCollector();
        parser.removeErrorListeners(); // Remove default console error listener
        parser.addErrorListener(errorCollector);

        AST.Program program = (AST.Program) new ASTBuilder().visit(parser.program());

        // Format as complete .foo file
        StringBuilder output = new StringBuilder();
        output.append("!!INPUT!!\n");
        output.append(code).append("\n\n");
        output.append("!!!\n");

        // Report parse errors if any - HALT if errors found
        if (errorCollector.hasErrors()) {
            output.append("PARSE ERRORS:\n");
            for (String error : errorCollector.getErrors()) {
                output.append("  ").append(error).append("\n");
            }
            output.append("\n");
            output.append("VM HALTED: Cannot execute due to parse errors.\n");
            output.append("!!!\n");
            return output.toString();
        }

        // Extract first brane
        AST.Brane brane = (AST.Brane) program.branes().branes().get(0);

        // Create UBC and run
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(brane);
        int stepCount = ubc.runToCompletion();
        BraneFiroe finalResult = ubc.getRootBrane();

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
        } finally {
            // Clean up execution context
            ExecutionContext.clearCurrent();
        }
    }

    @Override
    public String getName() {
        return "Java UBC";
    }
}
