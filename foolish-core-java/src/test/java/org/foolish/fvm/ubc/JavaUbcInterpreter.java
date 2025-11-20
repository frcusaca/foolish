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

        // Extract first brane
        AST.Brane brane = (AST.Brane) program.branes().branes().get(0);

        // Create UBC and run
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(brane);
        int stepCount = ubc.runToCompletion();
        BraneFiroe finalResult = ubc.getRootBrane();

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

    @Override
    public String getName() {
        return "Java UBC";
    }
}
