package org.foolish;

import org.antlr.v4.runtime.CharStream;
import org.antlr.v4.runtime.CharStreams;
import org.antlr.v4.runtime.CommonTokenStream;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.fvm.*;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;

/**
 * Simple read-eval-print loop for the Foolish language.
 */
public class Repl {

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
     * Translate and eval the given source, returning the result.
     */
    public static Object eval(String source, Env env) {
        AST.Program ast = parse(source);
        Insoe program = new InsoeVm().translate(ast);
        Firoe target = FiroeVm.wrap(program);
        FinearVmAbstract fvm = new FinearVmSimple();
        Firoe eval_result = fvm.evaluate(target);
        return eval_result;
    }

    public static void main(String[] args) throws IOException {
        BufferedReader reader = new BufferedReader(new InputStreamReader(System.in));
        Env env = new Env();
        String line;
        while ((line = reader.readLine()) != null) {
            if (line.isBlank()) continue;
            Object result = eval(line, env);
            if (result != null) {
                System.out.println(result);
            }
        }
    }
}
