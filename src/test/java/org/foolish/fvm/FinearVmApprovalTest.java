package org.foolish.fvm;

import org.foolish.grammar.FoolishLexer;
import org.foolish.grammar.FoolishParser;
import org.foolish.ast.AST;
import org.foolish.ast.ASTBuilder;
import org.foolish.ResourcesApprovalNamer;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.tree.ParseTree;
import org.approvaltests.Approvals;
import org.approvaltests.writers.ApprovalTextWriter;
import org.junit.jupiter.api.Test;

public class FinearVmApprovalTest {

    private void verifyApprovalOf(String code) {
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

        // Wrap to Midoe
        Midoe midoe = MidoeVm.wrap(insoe);

        // Evaluate
        FinearVmSimple vm = new FinearVmSimple();
        Midoe result = vm.evaluate(midoe);

        // Format output
        StringBuilder output = new StringBuilder();
        output.append("INPUT:\n");
        output.append(code).append("\n\n");
        output.append("PARSED AST:\n");
        output.append(ast.toString()).append("\n\n");
        output.append("INITIAL MIDOE:\n");
        output.append(midoe.toString()).append("\n\n");
        output.append("EVALUATION RESULT:\n");
        output.append(result.toString()).append("\n\n");
        output.append("FINAL BRANE STATUS:\n");
        if (result instanceof ProgramMidoe pm) {
            output.append(pm.brane().toString()).append("\n");
        } else if (result instanceof BraneMidoe bm) {
            output.append(bm.toString()).append("\n");
        } else {
            output.append(result.toString()).append("\n");
        }

        // Verify with approval test
        Approvals.verify(new ApprovalTextWriter(output.toString(), "txt"), new ResourcesApprovalNamer());
    }

    @Test
    void simpleArithmeticIsApproved() {
        verifyApprovalOf("{ x = 1 + 2; }");
    }

    @Test
    void complexArithmeticIsApproved() {
        verifyApprovalOf("{ x = 1 + 2 * 3; y = x - 4; }");
    }

    @Test
    void unaryOperationsIsApproved() {
        verifyApprovalOf("{ x = -5; y = +10; }");
    }

    @Test
    void ifExpressionIsApproved() {
        verifyApprovalOf("{ x = if 1 then 42 else 24; }");
    }

    @Test
    void nestedBranesIsApproved() {
        verifyApprovalOf("{ { x = 1; }; y = 2; }");
    }
}
