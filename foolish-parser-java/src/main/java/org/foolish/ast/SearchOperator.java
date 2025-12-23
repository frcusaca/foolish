package org.foolish.ast;

public enum SearchOperator {
    HEAD("^"),
    TAIL("$"),
    REGEXP_LOCAL("?"),
    REGEXP_GLOBAL("??");

    public final String symbol;

    SearchOperator(String symbol) {
        this.symbol = symbol;
    }

    @Override
    public String toString() {
        return symbol;
    }
}
