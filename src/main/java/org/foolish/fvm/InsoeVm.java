package org.foolish.fvm;

import org.foolish.ast.AST;

/**
 * Converts the high level AST into a single {@link Insoe} wrapper.
 */
public class InsoeVm {
    public Insoe translate(AST.Program program) {
        return new Insoe(program);
    }
}
