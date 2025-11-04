package org.foolish.fvm.ubc;

import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.ast.ASTFormatter;
import org.foolish.ResourcesApprovalNamer;
import org.foolish.fvm.Env;
import org.foolish.fvm.v1.Insoe;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.tree.ParseTree;
import org.approvaltests.Approvals;
import org.approvaltests.writers.ApprovalTextWriter;
import org.junit.jupiter.api.Test;

public class UbcApprovalTest {

    private void verifyUbcApprovalOf(String code) {
        // Create formatters
        Sequencer4Human humanSequencer = new Sequencer4Human();
        ASTFormatter astFormatter = new ASTFormatter();
        
        // Parse the code
        CharStream input = CharStreams.fromString(code);
        FoolishLexer lexer = new FoolishLexer(input);
        CommonTokenStream tokens = new CommonTokenStream(lexer);
        FoolishParser parser = new FoolishParser(tokens);
        ParseTree tree = parser.program();
        AST ast = new ASTBuilder().visit(tree);

        // Extract the brane from the program
        AST.Program program = (AST.Program) ast;
        AST.Branes branes = program.branes();
        AST.Characterizable firstBrane = branes.branes().get(0);
        
        if (!(firstBrane instanceof AST.Brane brane)) {
            throw new RuntimeException("Expected a brane but got: " + firstBrane.getClass());
        }

        // Create UBC and evaluate
        Insoe braneInsoe = new Insoe(brane);
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(braneInsoe, new Env());

        // Run to completion first
        int stepCount = ubc.runToCompletion();
        
        // Get final result
        BraneFiroe finalResult = ubc.getRootBrane();

        // Format output
        StringBuilder output = new StringBuilder();
        output.append("INPUT:\n");
        output.append(code).append("\n\n");
        
        output.append("PARSED AST:\n");
        output.append(astFormatter.format(ast)).append("\n\n");
        
        output.append("UBC EVALUATION:\n");
        output.append("Steps taken: ").append(stepCount).append("\n\n");
        
        output.append("FINAL RESULT:\n");
        output.append(humanSequencer.sequence(finalResult)).append("\n\n");
        
        output.append("COMPLETION STATUS:\n");
        output.append("Complete: ").append(ubc.isComplete()).append("\n");
        
        // Show environment if available
        Env finalEnv = ubc.getFinalEnvironment();
        if (finalEnv != null) {
            output.append("\nFINAL ENVIRONMENT:\n");
            output.append(finalEnv.toString()).append("\n");
        }

        // Verify with approval test
        Approvals.verify(new ApprovalTextWriter(output.toString(), "txt"), new ResourcesApprovalNamer());
    }

    // Simple arithmetic tests
    @Test
    void simpleIntegerIsApproved() {
        verifyUbcApprovalOf("{5;}");
    }

    @Test
    void simpleAdditionIsApproved() {
        verifyUbcApprovalOf("{3 + 4;}");
    }

    @Test
    void simpleSubtractionIsApproved() {
        verifyUbcApprovalOf("{10 - 3;}");
    }

    @Test
    void simpleMultiplicationIsApproved() {
        verifyUbcApprovalOf("{6 * 7;}");
    }

    @Test
    void simpleDivisionIsApproved() {
        verifyUbcApprovalOf("{15 / 3;}");
    }

    @Test
    void simpleUnaryMinusIsApproved() {
        verifyUbcApprovalOf("{-42;}");
    }


    // Complex arithmetic tests
    @Test
    void complexArithmeticIsApproved() {
        verifyUbcApprovalOf("{(5 + 3) * 2 - 1;}");
    }

    @Test
    void nestedArithmeticIsApproved() {
        verifyUbcApprovalOf("{((2 + 3) * (4 - 1)) / 5;}");
    }

    @Test
    void operatorPrecedenceIsApproved() {
        verifyUbcApprovalOf("{2 + 3 * 4 - 5;}");
    }

    // Multiple expression tests
    @Test
    void multipleExpressionsIsApproved() {
        verifyUbcApprovalOf("""
                {
                    1;
                    2;
                    3;
                }
        """);
    }

    @Test
    void multipleArithmeticExpressionsIsApproved() {
        verifyUbcApprovalOf("""
                {
                    5 + 3;
                    10 - 4;
                    2 * 6;
                }
        """);
    }

    @Test
    void mixedExpressionsIsApproved() {
        verifyUbcApprovalOf("""
                {
                    42;
                    (3 + 4) * 2;
                    -15;
                    100 / 5;
                }
        """);
    }

    // Nested brane tests
    @Test
    void nestedBranesIsApproved() {
        verifyUbcApprovalOf("""
                {
                    5;
                    {
                        10;
                        15;
                    };
                    20;
                }
        """);
    }

    @Test
    void deeplyNestedBranesIsApproved() {
        verifyUbcApprovalOf("""
                {
                    {
                        {
                            1;
                        };
                        2;
                    };
                    3;
                }
        """);
    }

    @Test
    void nestedBranesWithArithmeticIsApproved() {
        verifyUbcApprovalOf("""
                {
                    2 + 3;
                    {
                        4 * 5;
                        {
                            6 - 1;
                        };
                        7 + 8;
                    };
                    9 / 3;
                }
        """);
    }

    // Additional arithmetic tests
    @Test
    void chainedArithmeticIsApproved() {
        verifyUbcApprovalOf("{1 + 2 + 3 + 4;}");
    }

    @Test
    void mixedOperatorsIsApproved() {
        verifyUbcApprovalOf("{10 + 5 - 3 * 2;}");
    }

    // Edge cases
    @Test
    void emptyBraneIsApproved() {
        verifyUbcApprovalOf("{}");
    }

    @Test
    void singleExpressionIsApproved() {
        verifyUbcApprovalOf("{((((5))));}");
    }

    @Test
    void zeroDivisionIsApproved() {
        verifyUbcApprovalOf("{10 / 0;}");
    }

    @Test
    void negativeResultsIsApproved() {
        verifyUbcApprovalOf("""
                {
                    a = 5 - 10;
                    b = 3 * 2;
                    c = 15 + 7;
                }
        """);
    }

    // Complex nested tests
    @Test
    void veryDeepNestingIsApproved() {
        verifyUbcApprovalOf("""
                {
                    {
                        {
                            {
                                set_the\u202Fanswer\u2060is=42;
                            };
                        };
                    };
                }
        """);
    }

    @Test
    void fourLevelNestedBranesWithNamesIsApproved() {
        verifyUbcApprovalOf("""
                {
                    a = -10;
                    second'{
                        b = 20;
                        third'{
                            c = 30;
                            inner'{
                                d = 40;
                            };
                        };
                    };
                }
        """);
    }

    // If-then-else tests
    // TODO: Fix IfFiroe infinite loop issues before enabling these tests
    /*
    @Test
    void simpleIfThenElseIsApproved() {
        verifyUbcApprovalOf("""
                {
                    if 1 then 42 else 99;
                }
        """);
    }

    @Test
    void simpleIfThenElseFalseIsApproved() {
        verifyUbcApprovalOf("""
                {
                    if 0 then 42 else 99;
                }
        """);
    }

    @Test
    void ifThenNoElseImplicitNKIsApproved() {
        verifyUbcApprovalOf("""
                {
                    if 0 then 42;
                }
        """);
    }

    @Test
    void ifElifElseChainIsApproved() {
        verifyUbcApprovalOf("""
                {
                    if 0 then 10 elif 0 then 20 elif 1 then 30 else 40;
                }
        """);
    }

    @Test
    void ifWithComplexConditionIsApproved() {
        verifyUbcApprovalOf("""
                {
                    if 5 + 3 then 100 else 200;
                }
        """);
    }

    @Test
    void ifWithComplexThenValueIsApproved() {
        verifyUbcApprovalOf("""
                {
                    if 1 then 10 * 5 + 3 else 0;
                }
        """);
    }

    @Test
    void nestedIfThenElseIsApproved() {
        verifyUbcApprovalOf("""
                {
                    if 1 then if 0 then 10 else 20 else 30;
                }
        """);
    }

    @Test
    void ifWithFiMarkerIsApproved() {
        verifyUbcApprovalOf("""
                {
                    if 1 then 42 else 99 fi;
                }
        """);
    }

    @Test
    void deeplyNestedIfWithFiIsApproved() {
        verifyUbcApprovalOf("""
                {
                    if 1 then
                        if 0 then 10
                        elif 0 then 20
                        else if 1 then 30 else 40 fi
                    fi
                    else 50;
                }
        """);
    }

    @Test
    void multipleNestedIfCheckingFiAssociationIsApproved() {
        verifyUbcApprovalOf("""
                {
                    if 1 then
                        if 1 then
                            if 0 then 100 else 200 fi
                        else 300 fi
                    else 400 fi;
                }
        """);
    }
    */
}