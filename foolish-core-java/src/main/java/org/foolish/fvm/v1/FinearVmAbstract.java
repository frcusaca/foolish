package org.foolish.fvm.v1;

import org.foolish.fvm.Env;

public interface FinearVmAbstract {
    /**
     * Evaluates the given firoe within the provided environment.
     */
    Firoe evaluate(Firoe firoe, Env env);

    Firoe evaluate(Firoe firoe);

}
