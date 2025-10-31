package org.foolish.fvm;

import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.ast.ASTFormatter;
import org.foolish.ResourcesApprovalNamer;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.tree.ParseTree;
import org.approvaltests.Approvals;
import org.approvaltests.writers.ApprovalTextWriter;
import org.junit.jupiter.api.Test;

public class FinearVmApprovalTest {

    private void verifyApprovalOf(String code) {
        // Create formatters once
        TargoeFormatter humanReadableFormatter = FormatterFactory.humanReadable();
        TargoeFormatter verboseFormatter = FormatterFactory.verbose();
        ASTFormatter astFormatter = new ASTFormatter();
        
        // Parse the code
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        ParseTree tree = parser.program();
        AST ast = new ASTBuilder().visit(tree);

        // Convert to Insoe
        InsoeVm insoeVm = new InsoeVm();
        Insoe insoe = insoeVm.translate((AST.Program) ast);

        // Wrap to Firoe
        Firoe firoe = FiroeVm.wrap(insoe);

        // Evaluate
        FinearVmSimple vm = new FinearVmSimple();
        Firoe result = vm.evaluate(firoe);

        // Helper to get final brane
        Firoe finalBrane = result instanceof ProgramFiroe pm ? pm.brane() : result;

        // Format output
        StringBuilder output = new StringBuilder();
        output.append("INPUT:\n");
        output.append(code).append("\n\n");
        output.append("PARSED AST:\n");
        output.append(astFormatter.format(ast)).append("\n\n");
        
        // Human-readable format
        output.append("FINAL BRANE:\n");
        output.append(humanReadableFormatter.format(finalBrane)).append("\n\n");
        
        // Verbose format
        output.append("INITIAL FIROE:\n");
        output.append(verboseFormatter.format(firoe)).append("\n\n");
        output.append("EVALUATION RESULT:\n");
        output.append(verboseFormatter.format(result)).append("\n\n");
        output.append("FINAL BRANE STATUS:\n");
        output.append(verboseFormatter.format(finalBrane)).append("\n");

        // Verify with approval test
        Approvals.verify(new ApprovalTextWriter(output.toString(), "txt"), new ResourcesApprovalNamer());
    }

    @Test
    void simpleArithmeticIsApproved() {
        verifyApprovalOf("""
                {
                    x = 1 + 2;
                }
        """);
    }

    @Test
    void complexArithmeticIsApproved() {
        verifyApprovalOf("""
                {
                    x = 1 + 2 * 3;
                    y = x - 4;
                }
        """);
    }

    @Test
    void unaryOperationsIsApproved() {
        verifyApprovalOf("""
                {
                    x = -5;
                    y = +10;
                }
        """);
    }

    @Test
    void ifExpressionIsApproved() {
        verifyApprovalOf("""
                {
                    x = if 1 then 42 else 24;
                }
        """);
    }

    @Test
    void nestedBranesIsApproved() {
        verifyApprovalOf("""
                {
                    {
                        x = 1;
                    };
                    y = 2;
                }
        """);
    }

    // Deep nesting tests - 3+ layers of branes
    @Test
    void threeLayersOfBranesWithArithmetic() {
        verifyApprovalOf("""
                {
                    {
                        {
                            a = 1 + 2;
                        };
                        b = a * 3;
                    };
                    c = b - 4;
                }
        """);
    }

    @Test
    void fourLayersOfBranesWithComplexExpressions() {
        verifyApprovalOf("""
                {
                    {
                        {
                            {
                                x = 2 * 3;
                            };
                            y = x + 4;
                        };
                        z = y - 1;
                    };
                    w = z * 2;
                }
        """);
    }

    @Test
    void deeplyNestedBranesWithMultipleStatements() {
        verifyApprovalOf("""
                {
                    a = 1;
                    {
                        b = 2;
                        {
                            c = 3;
                            {
                                d = a + b + c;
                            };
                        };
                    };
                    result = a + b + c + d;
                }
        """);
    }

    @Test
    void threeLayersWithMixedOperations() {
        verifyApprovalOf("""
                {
                    {
                        {
                            p = -5;
                            q = +10;
                        };
                        r = p + q;
                    };
                    s = r * 2;
                }
        """);
    }

    // Deep nesting with unbound variables
    @Test
    void nestedBranesWithUnboundVariable() {
        verifyApprovalOf("""
                {
                    {
                        {
                            a = unknownVar;
                        };
                        b = a + 1;
                    };
                    c = b * 2;
                }
        """);
    }

    @Test
    void fourLayersWithUnboundInMiddle() {
        verifyApprovalOf("""
                {
                    {
                        x = 5;
                        {
                            y = missing + x;
                            {
                                z = y - 1;
                            };
                        };
                    };
                    w = z;
                }
        """);
    }

    @Test
    void mixedBoundAndUnbound() {
        verifyApprovalOf("""
                {
                    a = 10;
                    {
                        b = notDefined;
                        {
                            c = a + b;
                        };
                    };
                    d = c - a;
                }
        """);
    }

    // Deeply nested if-then-else (3+ levels)
    @Test
    void threeNestedIfThenElse() {
        verifyApprovalOf("""
                {
                    x = if 1 then if 2 then if 3 then 100 else 200 else 300 else 400;
                }
        """);
    }

    @Test
    void fourNestedIfWithElseIf() {
        verifyApprovalOf("""
                {
                    result = if 1 then if 0 then 10 else if 1 then if 1 then 42 else 24 else 12 else 5;
                }
        """);
    }

    @Test
    void deepIfNestingWithArithmetic() {
        verifyApprovalOf("""
                {
                    a = 5;
                    b = if a then if a - 3 then if a - 5 then 1 else 2 else 3 else 4;
                }
        """);
    }

    @Test
    void nestedIfWithBranes() {
        verifyApprovalOf("""
                {
                    {
                        {
                            x = if 1 then if 2 then 10 else 20 else if 3 then 30 else 40;
                        };
                    };
                    y = x + 5;
                }
        """);
    }

    // Deeply nested if-then-else with unbound variables
    @Test
    void nestedIfWithUnboundCondition() {
        verifyApprovalOf("""
                {
                    x = if unknown then if 1 then 42 else 24 else 0;
                }
        """);
    }

    @Test
    void deepIfWithUnboundInBranches() {
        verifyApprovalOf("""
                {
                    result = if 1 then if 0 then notFound else if 1 then stillMissing else 10 else 5;
                }
        """);
    }

    @Test
    void threeLayerIfWithMixedUnbound() {
        verifyApprovalOf("""
                {
                    a = 10;
                    b = if a then if missing then 1 else if a - 5 then undefined else 3 else 4;
                }
        """);
    }

    // Complex combinations
    @Test
    void deepBranesAndDeepIfCombined() {
        verifyApprovalOf("""
                {
                    {
                        {
                            a = 2;
                        };
                        b = if a then if a - 1 then 100 else 200 else 300;
                    };
                    c = b + a;
                }
        """);
    }

    @Test
    void complexNestingWithPartialUnbound() {
        verifyApprovalOf("""
                {
                    x = 5;
                    {
                        {
                            y = if x then if notHere then 1 else 2 else 3;
                        };
                        z = y + x;
                    };
                    w = z - unknown;
                }
        """);
    }

    @Test
    void maximalNestingComplexity() {
        verifyApprovalOf("""
                {
                    {
                        a = 1;
                        {
                            {
                                b = if a then if a + 1 then 10 else 20 else 30;
                            };
                            c = b * 2;
                        };
                    };
                    {
                        d = if c then c + missing else 0;
                    };
                    result = d - a;
                }
        """);
    }

    // SearchUp tests
    @Test
    void simpleSearchUp() {
        verifyApprovalOf("↑");
    }

    @Test
    void multipleSearchUps() {
        verifyApprovalOf("↑ ↑");
    }

    @Test
    void searchUpMixedWithBranes() {
        verifyApprovalOf("""
                { x = 1; }
                ↑
                { y = 2; }
        """);
    }

    @Test
    void searchUpWithComplexBranes() {
        verifyApprovalOf("""
                {
                    a = 5;
                    {
                        b = 10;
                    };
                }
                ↑
                {
                    c = a + b;
                }
        """);
    }
}
