package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.fvm.AlarmSystem;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * Creates the standard configuration/library for the Foolish environment.
 */
public class StandardConfiguration {

    /**
     * Creates the standard library as a FiroeWithBraneMind.
     * The returned FIR contains predefined values like ALARM_LEVELS.
     */
    public static FiroeWithBraneMind createStandardLibrary() {
        // Create a FiroeWithBraneMind to hold the standard library items
        FiroeWithBraneMind stdLib = new FiroeWithBraneMind((AST) null) {
            @Override
            protected void initialize() { setInitialized(); }
        };

        // Define ALARM_LEVELS brane
        // ALARM_LEVELS = [ NOT=0; BARELY=1; MILD=3; HAIR_RAISING=5; PANIC=10; ]

        List<AST.Expr> alarmStmts = new ArrayList<>();
        alarmStmts.add(createConstAssignment("NOT", AlarmSystem.NOT));
        alarmStmts.add(createConstAssignment("BARELY", AlarmSystem.BARELY));
        alarmStmts.add(createConstAssignment("MILD", AlarmSystem.MILD));
        alarmStmts.add(createConstAssignment("HAIR_RAISING", AlarmSystem.HAIR_RAISING));
        alarmStmts.add(createConstAssignment("PANIC", AlarmSystem.PANIC));

        AST.Brane alarmBraneAst = new AST.Brane(alarmStmts);

        // Run a BraneFiroe manually to evaluate the ALARM_LEVELS brane without recursion
        BraneFiroe val = new BraneFiroe(alarmBraneAst);
        while (val.isNye()) {
            val.step();
        }

        AST.Identifier id = new AST.Identifier("ALARM_LEVELS");
        AST.Assignment dummyAst = new AST.Assignment(id, new AST.IntegerLiteral(0));

        AssignmentFiroe assignment = new AssignmentFiroe(dummyAst) {
             @Override
             public FIR getResult() {
                 return val;
             }

             @Override
             public boolean isNye() {
                 return false;
             }
        };

        stdLib.storeFirs(assignment);

        return stdLib;
    }

    private static AST.Assignment createConstAssignment(String name, long value) {
        return new AST.Assignment(new AST.Identifier(name), new AST.IntegerLiteral(value));
    }
}
