package org.foolish;

import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.tree.ParseTree;
import org.approvaltests.Approvals;
import org.approvaltests.writers.ApprovalTextWriter;
import org.junit.jupiter.api.Test;

public class ParserApprovalTest {

    private void verifyApprovalOf(String code) {
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        ParseTree tree = parser.program();
        AST ast = new ASTBuilder().visit(tree);

        Approvals.verify(new ApprovalTextWriter(ast.toString(), "txt"), new ResourcesApprovalNamer());
    }

    @Test
    void arithmeticIsApproved() {
        verifyApprovalOf("""
                {
                    x = 1+2*3;
                    y = x-4;
                }
        """);
    }

    @Test
    void operatorPrecedenceIsApproved() {
        verifyApprovalOf("""
                {
                    x = -1 + +2 * 3 / *4 - +5;
                }
        """);
    }

    @Test
    void nestedBranesAreApproved() {
        verifyApprovalOf("""
                {
                    {
                        {z = 3;};
                        y = 2;
                        { w = 4; };
                    };
                    x = 1;
                    {
                        p = 5;
                        { q = 6; };
                    };
                }
        """);
    }

    @Test
    void detachmentBraneAssignmentsAreApproved() {
        verifyApprovalOf("""
                [
                    x = ???;
                    y;
                ]
                {
                    result = x;
                }
        """);
    }

    @Test
    void characterizedDetachmentBraneIsApproved() {
        verifyApprovalOf("""
                [
                    det'x = 1;
                    det'y;
                ]
        """);
    }

    @Test
    void otherSpacesAreApproved() {
        verifyApprovalOf("""
                [
                    variable\u202Fx = ???;
                    coordinate\u2060y;
                ]
                {
                    my_result = x;
                    my\u202Fresult\u2060coordinate=-42;
                    here\u202Ftoo'a\u202Fb\u2060c = 5;
                    simple\u2060name'd\u202Fe\u2060f;
                }
        """);
    }

}
