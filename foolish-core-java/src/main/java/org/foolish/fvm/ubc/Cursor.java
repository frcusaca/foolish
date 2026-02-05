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
     * Implementation of Cursor interface.
     */
    class CursorImpl implements Cursor {
        final BraneFiroe brane;
        final int statementIndex;

        public CursorImpl(BraneFiroe brane, int statementIndex) {
            this.brane = brane;
            this.statementIndex = statementIndex;
        }

        @Override
        public BraneFiroe brane() {
            return this.brane;
        }

        @Override
        public int statementIndex() {
            return this.statementIndex;
        }
    }
}
