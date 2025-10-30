package org.foolish.fvm;

import org.foolish.ast.AST;

public interface FinearVmAbstract {
    /**
     * Evaluates the given firoe within the provided environment.
     */
    abstract public Firoe evaluate(Firoe firoe, Env env);
    abstract public Firoe evaluate(Firoe firoe);

}
