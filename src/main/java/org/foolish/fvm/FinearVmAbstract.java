package org.foolish.fvm;

import org.foolish.ast.AST;

public interface FinearVmAbstract {
    /**
     * Evaluates the given midoe within the provided environment.
     */
    abstract public Midoe evaluate(Midoe midoe, Env env);
    abstract public Midoe evaluate(Midoe midoe);

}
