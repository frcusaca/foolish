package org.foolish.fvm.ubc;

public sealed interface Query permits StrictlyMatchingQuery, RegexpQuery {
    boolean matches(FIR brane_line);
}
