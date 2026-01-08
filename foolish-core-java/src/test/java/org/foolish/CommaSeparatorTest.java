package org.foolish;

import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.tree.ParseTree;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class CommaSeparatorTest {

    private AST parse(String code) {
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        ParseTree tree = parser.program();
        return new ASTBuilder().visit(tree);
    }

    @Test
    public void testCommaSeparator() {
        AST ast = parse("{ a=1, b=2, c=3; }");
        assertTrue(ast instanceof AST.Program);
        AST.Branes branes = ((AST.Program) ast).branes();
        assertEquals(1, branes.branes().size());
        AST.Brane brane = (AST.Brane) branes.branes().get(0);
        assertEquals(3, brane.statements().size());

        assertEquals("""
                {
                  a = 1;
                  b = 2;
                  c = 3;
                }
                """, ast.toString());
    }
}
