package org.foolish.fvm.ubc;

public interface Cursor {

    /**
     * A cursor points to a specific brane.
     *
     * @return the brane the cursor is pointing to.
     */
    BraneFiroe brane();

    /**
     * A cursor points to a specific statement. Since Statements are
     * numbered, they have associated with them an index. This should
     * correspond to the seek `#NNN` where `NNN` is a non-negative number.
     *
     * @return the statement index witin the brane
     */
    int statementIndex();

    /**
     * Having located the brane and the line we're on, the cursor
     * can furter be specified as before or after statement of this line.
     * <p>
     * By default this sould be true if unspecfied.
     *
     * @return wheter the cursor is infront of the start of the assignment
     */
    boolean beforeLine();

    boolean braneBound();

    enum SearchAttributes {
        BEFORE_LINE_BRANE_BOUND,
        BEFORE_LINE_NOT_BRANE_BOUND,
        AFTER_LINE_BRANE_BOUND,
        AFTER_LINE_NOT_BRANE_BOUND,

    }

    /**
     * Implementation of Cursor interface.
     * Attributes for search are by default before line and NOT brane bound.
     */
    class CursorImpl implements Cursor {
        final BraneFiroe brane;
        final int statementIndex;
        final boolean beforeLine;
        final boolean braneBound;

        public CursorImpl(BraneFiroe brane, int statementIndex, SearchAttributes attributes) {
            this.brane = brane;
            this.statementIndex = statementIndex;
            this.beforeLine = switch (attributes) {
                case null -> true;
                case BEFORE_LINE_BRANE_BOUND, BEFORE_LINE_NOT_BRANE_BOUND -> true;
                case AFTER_LINE_BRANE_BOUND, AFTER_LINE_NOT_BRANE_BOUND -> false;
            };
            this.braneBound = switch (attributes) {
                case null -> false;
                case BEFORE_LINE_BRANE_BOUND, AFTER_LINE_BRANE_BOUND -> true;
                case BEFORE_LINE_NOT_BRANE_BOUND, AFTER_LINE_NOT_BRANE_BOUND -> false;
            };
        }

        @Override
        public BraneFiroe brane() {
            return this.brane;
        }

        @Override
        public int statementIndex() {
            return this.statementIndex;
        }

        @Override
        public boolean beforeLine() {
            return beforeLine;
        }

        @Override
        public boolean braneBound() {
            return braneBound;
        }
    }
}
