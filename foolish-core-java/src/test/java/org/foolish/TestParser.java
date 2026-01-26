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
        System.out.println("Branes count: " + branes.primary().size());
        
        FoolishParser.PrimaryContext primary = branes.primary(0);
        System.out.println("Primary class: " + primary.getClass().getName());
        if (primary.characterizable() != null && primary.characterizable().brane() != null) {
            FoolishParser.BraneContext brane = primary.characterizable().brane();
            System.out.println("Brane found: " + brane.getText());
        }
    }
}
