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

        // Format output with INPUT and OUTPUT sections (like UBC tests)
        StringBuilder output = new StringBuilder();
        output.append("INPUT:\n");
        output.append(code.trim()).append("\n\n");
        output.append("OUTPUT:\n");
        output.append(ast.toString());

        Approvals.verify(new ApprovalTextWriter(output.toString(), "txt"), new ResourcesApprovalNamer());
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

    @Test
    void simpleIfThenElseIsApproved() {
        verifyApprovalOf("""
                {
                    x = if a then 1 else 2;
                }
        """);
    }

    @Test
    void simpleIfThenElseWithFiIsApproved() {
        verifyApprovalOf("""
                {
                    x = if a then 1 else 2 fi;
                }
        """);
    }

    @Test
    void ifWithoutElseIsApproved() {
        verifyApprovalOf("""
                {
                    x = if a then 1;
                }
        """);
    }

    @Test
    void ifWithoutElseButWithFiIsApproved() {
        verifyApprovalOf("""
                {
                    x = if a then 1 fi;
                }
        """);
    }

    @Test
    void ifWithElifChainIsApproved() {
        verifyApprovalOf("""
                {
                    x = if a then 1 elif b then 2 elif c then 3 else 4;
                }
        """);
    }

    @Test
    void ifWithElifChainAndFiIsApproved() {
        verifyApprovalOf("""
                {
                    x = if a then 1 elif b then 2 elif c then 3 else 4 fi;
                }
        """);
    }

    @Test
    void nestedIfWithFiMarkersIsApproved() {
        verifyApprovalOf("""
                {
                    x = if a then
                        if b then 10 else 20 fi
                    else 30 fi;
                }
        """);
    }

    @Test
    void deeplyNestedIfWithFiIsApproved() {
        verifyApprovalOf("""
                {
                    result = if x then
                        if y then 10
                        elif z then 20
                        else if w then 30 else 40 fi
                    fi
                    else 50;
                }
        """);
    }

    @Test
    void multipleNestedIfCheckingFiAssociationIsApproved() {
        verifyApprovalOf("""
                {
                    result = if a then
                        if b then
                            if c then 100 else 200 fi
                        else 300 fi
                    else 400 fi;
                }
        """);
    }

    @Test
    void complexNestedIfElifWithMixedFiMarkersIsApproved() {
        verifyApprovalOf("""
                {
                    result = if a then
                        if z then 10 else 2 fi
                    elif b then
                        if x then 30
                        else if y then 20 else 3 fi fi
                    elif d then
                        if p then
                            if q then 300 else 200 fi
                        elif r then 100
                        elif s then
                            if t then 50 elif u then 40 else 0 fi
                        fi
                    else 4 fi;
                }
        """);
    }

    @Test
    void ifWithFiInBraneSequenceIsApproved() {
        verifyApprovalOf("""
                {
                    first = if a then 1 else 2 fi;
                    second = if b then 3 fi;
                    third = if c then if d then 4 fi else 5 fi;
                }
        """);
    }

    @Test
    void simpleDereferenceIsApproved() {
        verifyApprovalOf("""
                {
                    x = a.b;
                }
        """);
    }

    @Test
    void chainedDereferenceIsApproved() {
        verifyApprovalOf("""
                {
                    result = a.b.c.d;
                }
        """);
    }

    @Test
    void dereferenceWithCharacterizationIsApproved() {
        verifyApprovalOf("""
                {
                    x = my_brane'a.coord;
                    y = 'anonymous.value;
                }
        """);
    }

    @Test
    void dereferenceOnExpressionIsApproved() {
        verifyApprovalOf("""
                {
                    x = (a + b).result;
                    y = (x * 2).value;
                }
        """);
    }

    @Test
    void dereferenceOnBraneIsApproved() {
        verifyApprovalOf("""
                {
                    x = {y = 10;}.y;
                    result = {a = 1; b = 2;}.a;
                }
        """);
    }

    @Test
    void dereferenceInComplexExpressionsIsApproved() {
        verifyApprovalOf("""
                {
                    x = a.b + c.d;
                    y = data.x * data.y;
                    z = (a.b + c.d) * e.f;
                }
        """);
    }

    @Test
    void dereferenceWithIfExprIsApproved() {
        verifyApprovalOf("""
                {
                    result = if condition then brane1.x else brane2.y;
                    value = data.field.nested;
                }
        """);
    }

    @Test
    void dereferenceWithThinSpacesIsApproved() {
        verifyApprovalOf("""
                {
                    x = my\u202Fbrane.my\u2060coordinate;
                }
        """);
    }

    @Test
    void dereferenceWithCharacterizedCoordinatesIsApproved() {
        verifyApprovalOf("""
                {
                    x = br.integer'x;
                    result = br.integer'x + br.float'y;
                    value = a.b.type'value;
                }
        """);
    }

    @Test
    void dereferenceOperatorPrecedenceWithUnaryIsApproved() {
        verifyApprovalOf("""
                {
                    x = -a.b;
                    y = +obj.value;
                    z = *ptr.data;
                    w = -a.b.c.d;
                }
        """);
    }

    @Test
    void dereferenceOperatorPrecedenceWithBinaryIsApproved() {
        verifyApprovalOf("""
                {
                    sum = a.b + c.d;
                    product = a.b * c.d;
                    complex = -a.x + b.y * c.z;
                    nested = (a.b + c.d) * (e.f - g.h);
                }
        """);
    }

    @Test
    void dereferenceOnBraneListIsApproved() {
        verifyApprovalOf("""
                {
                    a = ({z=1;}{x=1;}).c;
                    result = ({a=1;}{b=2;}{c=3;}).value;
                    complex = ({data=10;}{more=20;}).field.nested;
                }
        """);
    }

    // Regexp search tests with different operators
    @Test
    void regexpSearchWithDotOperatorIsApproved() {
        verifyApprovalOf("""
                {
                    result = myBrane.coordinate;
                    value = data.field;
                }
        """);
    }

    @Test
    void regexpSearchWithDotDotOperatorIsApproved() {
        verifyApprovalOf("""
                {
                    result = myBrane..pattern;
                    deep = data..nested;
                }
        """);
    }

    @Test
    void regexpSearchWithQuestionOperatorIsApproved() {
        verifyApprovalOf("""
                {
                    maybe = myBrane?optional;
                    check = data?exists;
                }
        """);
    }

    @Test
    void regexpSearchWithQuestionQuestionOperatorIsApproved() {
        verifyApprovalOf("""
                {
                    unknown = myBrane??search;
                    find = data??anything;
                }
        """);
    }

    // Tests for stacked regexp suffixes
    @Test
    void stackedRegexpSearchSameOperatorIsApproved() {
        verifyApprovalOf("""
                {
                    result = a.b.c.d;
                    chain = x..y..z;
                }
        """);
    }

    @Test
    void stackedRegexpSearchMixedOperatorsIsApproved() {
        verifyApprovalOf("""
                {
                    mixed = a.b..c?d??e;
                    complex = data.field..nested?maybe??unknown;
                }
        """);
    }

    @Test
    void stackedRegexpSearchWithCharacterizationIsApproved() {
        verifyApprovalOf("""
                {
                    result = brane'a.coord'x..deep'y?opt'z;
                }
        """);
    }

    // Tests for regexp ending conditions
    @Test
    void regexpSearchEndsAtWhitespaceIsApproved() {
        verifyApprovalOf("""
                {
                    x = a.b + c.d;
                    y = data.field * value.coord;
                }
        """);
    }

    @Test
    void regexpSearchEndsAtSemicolonIsApproved() {
        verifyApprovalOf("""
                {
                    result = myBrane.pattern;
                    other = data.field;
                }
        """);
    }

    @Test
    void regexpSearchEndsAtOperatorIsApproved() {
        verifyApprovalOf("""
                {
                    sum = a.x + b.y;
                    product = data.val * count.num;
                    diff = first.coord - second.coord;
                }
        """);
    }

    @Test
    void regexpSearchEndsAtParenIsApproved() {
        verifyApprovalOf("""
                {
                    result = (a.b);
                    nested = (data.field).value;
                }
        """);
    }

    // Tests for regexp on different element types
    @Test
    void regexpSearchOnIdentifierIsApproved() {
        verifyApprovalOf("""
                {
                    result = identifier.pattern;
                    value = name.coord;
                }
        """);
    }

    @Test
    void regexpSearchOnParenthesizedExprIsApproved() {
        verifyApprovalOf("""
                {
                    result = (a + b).field;
                    value = (x * 2).coord;
                }
        """);
    }

    @Test
    void regexpSearchOnBraneIsApproved() {
        verifyApprovalOf("""
                {
                    result = {x = 10;}.field;
                    value = {a = 1; b = 2;}.coord;
                }
        """);
    }

    @Test
    void regexpSearchOnBraneListIsApproved() {
        verifyApprovalOf("""
                {
                    result = ({a=1;}{b=2;}).pattern;
                    deep = ({x=1;}{y=2;}).field..nested;
                }
        """);
    }

    @Test
    void regexpSearchOnCharacterizedBraneIsApproved() {
        verifyApprovalOf("""
                {
                    result = type'{x = 10;}.field;
                    value = name'data'{a = 1;}.coord;
                }
        """);
    }

    @Test
    void regexpSearchWithIfExprIsApproved() {
        verifyApprovalOf("""
                {
                    result = if condition then brane1.x else brane2.y;
                    chain = data.field..nested;
                }
        """);
    }

    @Test
    void regexpSearchInComplexExpressionsIsApproved() {
        verifyApprovalOf("""
                {
                    result = a.b + c..d * e?f - g??h;
                    nested = (x.y + z.w) * data..field;
                }
        """);
    }

    // Tests for balanced parentheses in regexp patterns
    @Test
    void regexpWithBalancedParenthesesIsApproved() {
        verifyApprovalOf("""
                {
                    result = brane.(pattern);
                    value = data..(nested);
                }
        """);
    }

    @Test
    void regexpWithBalancedBracesIsApproved() {
        verifyApprovalOf("""
                {
                    result = brane.{pattern};
                    value = data..{nested};
                }
        """);
    }

    @Test
    void regexpWithBalancedBracketsIsApproved() {
        verifyApprovalOf("""
                {
                    result = brane.[pattern];
                    value = data..[nested];
                }
        """);
    }

    @Test
    void regexpWithNestedBalancedParenthesesIsApproved() {
        verifyApprovalOf("""
                {
                    result = brane.((nested));
                    deep = data..{outer[inner(deep)]};
                    mixed = x?(a{b[c]});
                }
        """);
    }

    @Test
    void regexpWithMultipleBalancedGroupsIsApproved() {
        verifyApprovalOf("""
                {
                    result = brane.(first)(second)[third]{fourth};
                    chain = data..(a)(b)..(c)[d]??(e){f};
                }
        """);
    }

    @Test
    void regexpWithBalancedParensAndIdentifiersIsApproved() {
        verifyApprovalOf("""
                {
                    result = brane.(foo)bar(baz);
                    complex = data..prefix(inner)suffix[more]end;
                }
        """);
    }

}
