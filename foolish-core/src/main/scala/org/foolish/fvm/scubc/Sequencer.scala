package org.foolish.fvm.scubc

/**
 * Abstract base trait for formatting FIR objects into various output formats.
 * A Sequencer can produce arbitrary output types by traversing and formatting
 * the FIR tree structure.
 */
trait Sequencer[T]:

  /** Sequence a FIR into the output format */
  def sequence(fir: FIR): T = sequence(fir, 0)

  /** Sequence a FIR with a specific depth level for indentation */
  def sequence(fir: FIR, depth: Int): T

  /** Sequence a BraneFiroe into the output format */
  protected def sequenceBrane(brane: BraneFiroe, depth: Int): T

  /** Sequence a ValueFiroe into the output format */
  protected def sequenceValue(value: ValueFiroe, depth: Int): T

  /** Sequence a BinaryFiroe into the output format */
  protected def sequenceBinary(binary: BinaryFiroe, depth: Int): T

  /** Sequence a UnaryFiroe into the output format */
  protected def sequenceUnary(unary: UnaryFiroe, depth: Int): T

  /** Sequence an IfFiroe into the output format */
  protected def sequenceIf(ifFiroe: IfFiroe, depth: Int): T

  /** Sequence a SearchUpFiroe into the output format */
  protected def sequenceSearchUp(searchUp: SearchUpFiroe, depth: Int): T

  /** Sequence an AssignmentFiroe into the output format */
  protected def sequenceAssignment(assignment: AssignmentFiroe, depth: Int): T

  /** Sequence an NKFiroe (not-known value) into the output format */
  protected def sequenceNK(nk: FIR, depth: Int): T
