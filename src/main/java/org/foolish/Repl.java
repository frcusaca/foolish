package org.foolish;

import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.fvm.ASTToFVM;
import org.foolish.fvm.Environment;
import org.foolish.fvm.Program;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;

/** Simple read-eval-print loop for the Foolish language. */
public class Repl {
    public static void main(String[] args) throws IOException {
        BufferedReader reader = new BufferedReader(new InputStreamReader(System.in));
        Environment env = new Environment();
        ASTToFVM translator = new ASTToFVM();
        String line;
        while ((line = reader.readLine()) != null) {
            if (line.isBlank()) continue;
            CharStream input = CharStreams.fromString(line);
            FoolishLexer lexer = new FoolishLexer(input);
            CommonTokenStream tokens = new CommonTokenStream(lexer);
            FoolishParser parser = new FoolishParser(tokens);
            AST.Program ast = (AST.Program) new ASTBuilder().visitProgram(parser.program());
            Program program = translator.translate(ast);
            Object result = program.execute(env);
            if (result != null) {
                System.out.println(result);
            }
        }
    }
}
