package org.foolish.repl;

import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.antlr.v4.runtime.tree.ParseTree;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.foolish.interpreter.Interpreter;
import org.foolish.interpreter.Value;

import java.io.BufferedReader;
import java.io.InputStreamReader;

/**
 * Simple REPL for the Foolish language.
 */
public class Repl {
    public static void main(String[] args) throws Exception {
        Interpreter interpreter = new Interpreter();
        BufferedReader reader = new BufferedReader(new InputStreamReader(System.in));
        String line;
        System.out.print("> ");
        while ((line = reader.readLine()) != null) {
            line = line.trim();
            if (line.equals("exit") || line.equals("quit")) {
                break;
            }
            if (line.isEmpty()) {
                System.out.print("> ");
                continue;
            }
            try {
                CharStream input = CharStreams.fromString(line);
                FoolishLexer lexer = new FoolishLexer(input);
                CommonTokenStream tokens = new CommonTokenStream(lexer);
                FoolishParser parser = new FoolishParser(tokens);
                ParseTree tree = parser.stmt();
                AST ast = new ASTBuilder().visit(tree);
                Value result = interpreter.evaluate(ast);
                System.out.println(result);
            } catch (Exception e) {
                System.out.println("Error: " + e.getMessage());
            }
            System.out.print("> ");
        }
    }
}
