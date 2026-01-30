package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

public class DebugParse {
    public static void main(String[] args) {
        String input = "{??? ?x}";
        System.out.println("Parsing: " + input);
        AST.Program program = UbcRepl.parse(input);
        System.out.println("Program: " + program);

        AST.Branes branes = program.branes();
        AST.Characterizable first = branes.branes().get(0);
        AST.Brane brane = (AST.Brane) first;
        AST.Expr stmt = brane.statements().get(0);

        System.out.println("Statement class: " + stmt.getClass().getName());
        System.out.println("Statement: " + stmt);

        FIR firoe = FIR.createFiroeFromExpr(stmt);
        System.out.println("FIR class: " + firoe.getClass().getName());
        System.out.println("Is AbstractSearchFiroe? " + (firoe instanceof AbstractSearchFiroe));
    }
}
