package org.foolish.fvm;

/**
 * Enumeration of the various kinds of evaluation targets within the FVM.
 * Each entry is assigned a stable numeric identifier so that instructions can
 * be referenced both by name and id.
 */
public enum TargoeType {
    ASSIGNMENT(1),
    BINARY_EXPR(2),
    BRANE(3),
    BRANES(4),
    SINGLE_BRANE(5),
    IDENTIFIER_EXPR(6),
    INTEGER_LITERAL(7),
    IF_EXPR(8),
    PROGRAM(9),
    UNARY_EXPR(10),
    RESOE(11),
    MIDOE(12);

    private final int id;

    TargoeType(int id) {
        this.id = id;
    }

    /**
     * @return stable numeric id for the targoe type
     */
    public int id() {
        return id;
    }
}

