package org.foolish.fvm;

/**
 * Container holding both the result of evaluating a {@link Targoe} and the
 * resulting environment. The environment is provided so that copy-on-write
 * semantics can propagate updated environments to subsequent evaluations.
 */
public record EvalResult(Resoe value, Environment env) {}
