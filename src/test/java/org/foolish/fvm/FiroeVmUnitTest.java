package org.foolish.fvm;

import org.foolish.ast.AST;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

public class FiroeVmUnitTest {
    @Test
    void wrapHandlesNullInput() {
        Firoe result = FiroeVm.wrap(null);
        assertNotNull(result);
        assertTrue(result instanceof Firoe);
    }

    @Test
    void wrapHandlesProgram() {
        AST.Program program = new AST.Program(new AST.Branes(List.of()));
        Insoe input = new Insoe(program);
        Firoe result = FiroeVm.wrap(input);

        assertTrue(result instanceof ProgramFiroe);
    }

    @Test
    void wrapHandlesBinaryExpression() {
        AST.Identifier left = new AST.Identifier("x");
        AST.IntegerLiteral right = new AST.IntegerLiteral(42);
        AST.BinaryExpr binExpr = new AST.BinaryExpr(new AST.Identifier("integer"), "+", left, right);
        Insoe input = new Insoe(binExpr);

        Firoe result = FiroeVm.wrap(input);

        assertTrue(result instanceof BinaryFiroe);
        BinaryFiroe binary = (BinaryFiroe) result;
        assertInstanceOf(IdentifierFiroe.class, binary.left());
        assertInstanceOf(Finear.class, binary.right());
    }

    @Test
    void wrapHandlesIntegerLiteral() {
        AST.IntegerLiteral literal = new AST.IntegerLiteral(42);
        Insoe input = new Insoe(literal);

        Firoe result = FiroeVm.wrap(input);

        assertInstanceOf(Finear.class, result);
        assertEquals (42 , ((Finear) result).longValue());
    }

    @Test
    void wrapHandlesIfExpression() {
        AST.Identifier condition = new AST.Identifier("condition");
        AST.IntegerLiteral thenExpr = new AST.IntegerLiteral(1);
        AST.IntegerLiteral elseExpr = new AST.IntegerLiteral(0);
        AST.IfExpr ifExpr = new AST.IfExpr(condition, thenExpr, elseExpr, List.of());
        Insoe input = new Insoe(ifExpr);

        Firoe result = FiroeVm.wrap(input);

        assertInstanceOf(IfFiroe.class, result);
        IfFiroe ifFiroe = (IfFiroe) result;
        assertInstanceOf(IdentifierFiroe.class, ifFiroe.condition());
        assertInstanceOf(Finear.class, ifFiroe.thenExpr());
        assertInstanceOf(Finear.class, ifFiroe.elseExpr());
        assertTrue(ifFiroe.elseIfs().isEmpty());
    }

    @Test
    void wrapHandlesSimpleSearchUp() {
        AST.SearchUP searchUp = new AST.SearchUP();
        Insoe input = new Insoe(searchUp);

        Firoe result = FiroeVm.wrap(input);

        assertNotNull(result);
        assertInstanceOf(SearchUpFiroe.class, result);
        SearchUpFiroe searchUpFiroe = (SearchUpFiroe) result;
        assertNotNull(searchUpFiroe.base());
        assertInstanceOf(Insoe.class, searchUpFiroe.base());
        assertEquals(searchUp, ((Insoe) searchUpFiroe.base()).ast());
        // When wrapped at top level, parent should be null
        assertNull(searchUpFiroe.parent());
    }

    @Test
    void wrapHandlesCharacterizedSearchUp() {
        AST.SearchUP searchUp = new AST.SearchUP(new AST.Identifier("type"));
        Insoe input = new Insoe(searchUp);

        Firoe result = FiroeVm.wrap(input);

        assertNotNull(result);
        assertInstanceOf(SearchUpFiroe.class, result);
        SearchUpFiroe searchUpFiroe = (SearchUpFiroe) result;
        assertNotNull(searchUpFiroe.base());
        assertInstanceOf(Insoe.class, searchUpFiroe.base());
        assertEquals(searchUp, ((Insoe) searchUpFiroe.base()).ast());
        // When wrapped at top level, parent should be null
        assertNull(searchUpFiroe.parent());
    }

    @Test
    void wrapHandlesBranesWithSearchUp() {
        AST.Brane brane1 = new AST.Brane(List.of(new AST.IntegerLiteral(42)));
        AST.SearchUP searchUp = new AST.SearchUP();
        AST.Brane brane2 = new AST.Brane(List.of(new AST.IntegerLiteral(99)));
        AST.Branes branes = new AST.Branes(List.of(brane1, searchUp, brane2));
        Insoe input = new Insoe(branes);

        Firoe result = FiroeVm.wrap(input);

        assertInstanceOf(BraneFiroe.class, result);
        BraneFiroe braneFiroe = (BraneFiroe) result;
        assertEquals(3, braneFiroe.statements().size());

        // First brane's statement
        assertInstanceOf(Finear.class, braneFiroe.statements().get(0));
        assertEquals(42L, ((Finear) braneFiroe.statements().get(0)).longValue());

        // SearchUp wraps to a SearchUpFiroe with parent pointer
        Firoe searchUpFiroe = braneFiroe.statements().get(1);
        assertInstanceOf(SearchUpFiroe.class, searchUpFiroe);
        assertInstanceOf(Insoe.class, searchUpFiroe.base());
        assertEquals(searchUp, ((Insoe) searchUpFiroe.base()).ast());
        // Verify parent pointer is set to the BraneFiroe
        assertNotNull(((SearchUpFiroe) searchUpFiroe).parent());
        assertInstanceOf(BraneFiroe.class, ((SearchUpFiroe) searchUpFiroe).parent());

        // Second brane's statement
        assertInstanceOf(Finear.class, braneFiroe.statements().get(2));
        assertEquals(99L, ((Finear) braneFiroe.statements().get(2)).longValue());
    }

    @Test
    void wrapHandlesMultipleSearchUps() {
        AST.SearchUP searchUp1 = new AST.SearchUP();
        AST.SearchUP searchUp2 = new AST.SearchUP(new AST.Identifier("n"));
        AST.SearchUP searchUp3 = new AST.SearchUP(new AST.Identifier("t"));
        AST.Branes branes = new AST.Branes(List.of(searchUp1, searchUp2, searchUp3));
        Insoe input = new Insoe(branes);

        Firoe result = FiroeVm.wrap(input);

        assertInstanceOf(BraneFiroe.class, result);
        BraneFiroe braneFiroe = (BraneFiroe) result;
        assertEquals(3, braneFiroe.statements().size());

        // Verify all SearchUPs have the BraneFiroe as parent
        assertInstanceOf(SearchUpFiroe.class, braneFiroe.statements().get(0));
        assertEquals(searchUp1, ((Insoe) braneFiroe.statements().get(0).base()).ast());
        assertNotNull(((SearchUpFiroe) braneFiroe.statements().get(0)).parent());
        assertInstanceOf(BraneFiroe.class, ((SearchUpFiroe) braneFiroe.statements().get(0)).parent());

        assertInstanceOf(SearchUpFiroe.class, braneFiroe.statements().get(1));
        assertEquals(searchUp2, ((Insoe) braneFiroe.statements().get(1).base()).ast());
        assertNotNull(((SearchUpFiroe) braneFiroe.statements().get(1)).parent());
        assertInstanceOf(BraneFiroe.class, ((SearchUpFiroe) braneFiroe.statements().get(1)).parent());

        assertInstanceOf(SearchUpFiroe.class, braneFiroe.statements().get(2));
        assertEquals(searchUp3, ((Insoe) braneFiroe.statements().get(2).base()).ast());
        assertNotNull(((SearchUpFiroe) braneFiroe.statements().get(2)).parent());
        assertInstanceOf(BraneFiroe.class, ((SearchUpFiroe) braneFiroe.statements().get(2)).parent());
    }
}