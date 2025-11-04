package org.foolish.fvm.ubc;

/**
 * Abstract base class for formatting FIR objects into various output formats.
 * A Sequencer can produce arbitrary output types by traversing and formatting
 * the FIR tree structure.
 *
 * @param <T> The output type this sequencer produces (e.g., String, StringBuilder, etc.)
 */
public abstract class Sequencer<T> {

    /**
     * Sequence a FIR into the output format.
     *
     * @param fir The FIR to sequence
     * @return The formatted output
     */
    public abstract T sequence(FIR fir);

    /**
     * Sequence a FIR with a specific depth level for indentation.
     *
     * @param fir   The FIR to sequence
     * @param depth The current depth level for indentation
     * @return The formatted output
     */
    public abstract T sequence(FIR fir, int depth);

    /**
     * Sequence a BraneFiroe into the output format.
     *
     * @param brane The BraneFiroe to sequence
     * @param depth The current depth level for indentation
     * @return The formatted output
     */
    protected abstract T sequenceBrane(BraneFiroe brane, int depth);

    /**
     * Sequence a ValueFiroe into the output format.
     *
     * @param value The ValueFiroe to sequence
     * @param depth The current depth level for indentation
     * @return The formatted output
     */
    protected abstract T sequenceValue(ValueFiroe value, int depth);

    /**
     * Sequence a BinaryFiroe into the output format.
     *
     * @param binary The BinaryFiroe to sequence
     * @param depth  The current depth level for indentation
     * @return The formatted output
     */
    protected abstract T sequenceBinary(BinaryFiroe binary, int depth);

    /**
     * Sequence a UnaryFiroe into the output format.
     *
     * @param unary The UnaryFiroe to sequence
     * @param depth The current depth level for indentation
     * @return The formatted output
     */
    protected abstract T sequenceUnary(UnaryFiroe unary, int depth);

    /**
     * Sequence an IfFiroe into the output format.
     *
     * @param ifFiroe The IfFiroe to sequence
     * @param depth   The current depth level for indentation
     * @return The formatted output
     */
    protected abstract T sequenceIf(IfFiroe ifFiroe, int depth);

    /**
     * Sequence a SearchUpFiroe into the output format.
     *
     * @param searchUp The SearchUpFiroe to sequence
     * @param depth    The current depth level for indentation
     * @return The formatted output
     */
    protected abstract T sequenceSearchUp(SearchUpFiroe searchUp, int depth);

    /**
     * Sequence an AssignmentFiroe into the output format.
     *
     * @param assignment The AssignmentFiroe to sequence
     * @param depth      The current depth level for indentation
     * @return The formatted output
     */
    protected abstract T sequenceAssignment(AssignmentFiroe assignment, int depth);

    /**
     * Sequence an NKFiroe (not-known value) into the output format.
     *
     * @param nk    The NKFiroe to sequence
     * @param depth The current depth level for indentation
     * @return The formatted output
     */
    protected abstract T sequenceNK(FIR nk, int depth);
}
