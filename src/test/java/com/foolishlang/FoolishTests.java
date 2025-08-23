
package com.foolishlang;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class FoolishTests {

    private static final String SAMPLE = "{\n" +
        "  myProduct = (a: t'Int, b: t'Int) -> {\n" +
        "    product = a * b\n" +
        "  }\n" +
        "  call_myProduct_result = {1,2} myProduct\n" +
        "  call_myProduct_result =$ {1,2} myProduct\n" +
        "  firstLine = call_myProduct_result^\n" +
        "  lastLine  = call_myProduct_result$\n" +
        "  nthLine   = call_myProduct_result#3\n" +
        "}";

    @Test
    public void parseProgramNoErrors() {
        var pr = ParserFacade.parseProgram(SAMPLE);
        assertTrue(pr.errors().isEmpty(), "Parse errors: " + pr.errors());
    }

    @Test
    public void buildAst() {
        var pr = ParserFacade.parseProgram(SAMPLE);
        var ast = (AST.Program) new AstBuilder().visit(pr.tree());
        assertNotNull(ast);
        assertNotNull(ast.body);
        assertTrue(ast.body.statements.size() >= 5);
    }

    @Test
    public void incrementalEdit() {
        ParserFacade.Incremental inc = new ParserFacade.Incremental(SAMPLE);
        assertFalse(inc.segments().isEmpty());
        // edit first char inside product line (non-destructive)
        String needle = "product = a * b";
        int start = SAMPLE.indexOf(needle);
        inc.edit(start, start+1, "p");
        assertFalse(inc.segments().isEmpty());
    }
}
