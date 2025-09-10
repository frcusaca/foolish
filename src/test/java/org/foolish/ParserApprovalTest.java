package org.foolish;

import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.tree.ParseTree;
import org.approvaltests.Approvals;
import org.junit.jupiter.api.Test;

public class ParserApprovalTest {

    private AST parse(String code) {
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        ParseTree tree = parser.program();
        return new ASTBuilder().visit(tree);
    }

    @Test
    void arithmeticIsApproved() {
        AST ast = parse("{ x = 1+2*3; y = x-4; }");
        Approvals.verify(ast.toString());
    }
}
