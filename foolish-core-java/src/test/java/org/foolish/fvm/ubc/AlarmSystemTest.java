package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import org.foolish.fvm.AlarmSystem;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.List;

public class AlarmSystemTest {

    @Test
    public void testStandardLibraryInjection() {
        AST.Brane emptyBrane = new AST.Brane(Collections.emptyList());
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(emptyBrane);

        // Verify that the UBC initializes without error, implying standard library is linked
        ubc.runToCompletion();
    }

    @Test
    public void testAlarmConfiguration() {
        // Verify that setting alarming_level executes without error
        AST.Identifier id = new AST.Identifier("alarming_level");
        AST.Assignment assign = new AST.Assignment(id, new AST.IntegerLiteral(5));

        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(new AST.Brane(List.of(assign)));
        ubc.runToCompletion();
    }

    @Test
    public void testStandardLibraryAccess() {
        UnicelluarBraneComputer ubc = new UnicelluarBraneComputer(new AST.Brane(Collections.emptyList()));
        ubc.runToCompletion();
    }

    @Test
    public void testAlarmRaisingDoesNotCrash() {
        AlarmSystem.raise(null, "Test alarm without context", AlarmSystem.PANIC);

        BraneMemory mem = new BraneMemory(null);
        AlarmSystem.raise(mem, "Test alarm with empty context", AlarmSystem.PANIC);
    }
}
