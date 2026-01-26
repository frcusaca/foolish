package org.foolish.fvm.scubc

/**
 * NYE STATES (say "NICE") - represents the evaluation stages of a Firoe.
 *
 * This enum tracks the progression of a FIR from initial construction through
 * complete evaluation. Each state represents a distinct phase in the evaluation
 * lifecycle, with transitions managed through the FIR.setNyes() method.
 *
 * The evaluation flow typically progresses as follows:
 * UNINITIALIZED → INITIALIZED → CHECKED → EVALUATING → CONSTANIC → CONSTANT
 *
 * CONSTANIC (say "CON-STAN-NICK") represents a state where evaluation has paused due to missing information (e.g. unbound identifiers),
 * but could resume in a different context. CONSTANt IN Context - "Stay Foolish" state.
 * CONSTANT represents a fully evaluated, immutable state (Result or Error).
 */
enum Nyes:
  /**
   * Just an AST - this is where the constructor leaves the object.
   * No initialization has occurred yet.
   */
  case UNINITIALIZED

  /**
   * Various misc items initialized including cache for final value.
   * Transition taken care of by step().
   */
  case INITIALIZED

  /**
   * Type/reference checking completed.
   * Reserved for future type checking and reference validation.
   * Currently used as a transitional state between INITIALIZED and EVALUATING.
   * AB (Abstract Brane), IB (Implementation Brane) established firmly.
   * All variables for an expression are collected and validated.
   * Transition taken care of by step().
   */
  case CHECKED

  /**
   * Take a step() as we do currently, including branes.
   * Active evaluation is in progress.
   * Transition taken care of by step().
   */
  case EVALUATING

  /**
   * Like CONSTANT, it is a terminal state as far as step() is concerned.
   * Evaluation halted due to missing information (unbound identifiers).
   * It is CONSTANt IN Context (say "CON-STAN-NICK"). It is constant if context does not change.
   * It is not required that this FIR do change for some context. But
   * for computational efficiency, it would be best if Constanic state
   * only happens for a FIR that is expected to change if context changes.
   */
  case CONSTANIC

  /**
   * No more changes will happen with call to step() unless the environment changes.
   * This is the only !isNye() state - a FIR in CONSTANT state is fully evaluated.
   */
  case CONSTANT
