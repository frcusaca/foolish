package org.foolish.ast;

public enum SearchOperator {
    HEAD("^"),
    TAIL("$"),
    REGEXP_LOCAL("?"),              // Backward search (from end to start)
    REGEXP_FORWARD_LOCAL("~"),      // Forward search (from start to end)
    REGEXP_GLOBAL("??"),            // Find-all backward (not yet implemented)
    REGEXP_FORWARD_GLOBAL("~~"),    // Find-all forward (not yet implemented)
    SEEK("#");

    public final String symbol;

    SearchOperator(String symbol) {
        this.symbol = symbol;
    }

    @Override
    public String toString() {
        return symbol;
    }
}
