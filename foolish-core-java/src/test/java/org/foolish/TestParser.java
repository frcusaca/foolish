package org.foolish;

import org.antlr.v4.runtime.*;
import org.foolish.grammar.*;

public class TestParser {
    public static void main(String[] args) {
        String code = "{ a = 5; }";
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        
        FoolishParser.ProgramContext program = parser.program();
        System.out.println("Program tree: " + program.toStringTree(parser));
        
        FoolishParser.BranesContext branes = program.branes();
        System.out.println("Branes count: " + branes.brane().size());
        
        FoolishParser.BraneContext brane = branes.brane(0);
        System.out.println("Brane class: " + brane.getClass().getName());
        System.out.println("standard_brane: " + brane.standard_brane());
        System.out.println("detach_brane: " + brane.detach_brane());
        System.out.println("brane_search: " + brane.brane_search());
        
        if (brane.standard_brane() != null) {
            System.out.println("Statements: " + brane.standard_brane().stmt().size());
            for (int i = 0; i < brane.standard_brane().stmt().size(); i++) {
                FoolishParser.StmtContext stmt = brane.standard_brane().stmt(i);
                System.out.println("  Statement " + i + ": " + stmt.getText());
            }
        }
    }
}
