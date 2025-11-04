package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Human-friendly sequencer that formats FIR objects with configurable indentation.
 * Uses '➠' as the default tab character for visual clarity.
 */
public class Sequencer4Human extends Sequencer<String> {
    private static final String DEFAULT_TAB = "＿"; //U+FF3F
    private final String tabChar;

    /**
     * Creates a Sequencer4Human with custom tab character.
     *
     * @param tabChar The character(s) to use for indentation
     */
    public Sequencer4Human(String tabChar) {
        this.tabChar = tabChar;
    }

    /**
     * Creates a Sequencer4Human with default tab character '➠'.
     */
    public Sequencer4Human() {
        this(DEFAULT_TAB);
    }

    @Override
    public String sequence(FIR fir) {
        return sequence(fir, 0);
    }

    @Override
    public String sequence(FIR fir, int depth) {
        if (fir instanceof BraneFiroe brane) {
            return sequenceBrane(brane, depth);
        } else if (fir instanceof ValueFiroe value) {
            return sequenceValue(value, depth);
        } else if (fir instanceof BinaryFiroe binary) {
            return sequenceBinary(binary, depth);
        } else if (fir instanceof UnaryFiroe unary) {
            return sequenceUnary(unary, depth);
        } else if (fir instanceof IfFiroe ifFiroe) {
            return sequenceIf(ifFiroe, depth);
        } else if (fir instanceof SearchUpFiroe searchUp) {
            return sequenceSearchUp(searchUp, depth);
        } else if (fir instanceof AssignmentFiroe assignment) {
            return sequenceAssignment(assignment, depth);
        } else {
            return indent(depth) + "???";
        }
    }

    @Override
    protected String sequenceBrane(BraneFiroe brane, int depth) {
        StringBuilder sb = new StringBuilder();

        // Add characterization (name) if present
        if (brane.ast() instanceof AST.Brane braneAst &&
            braneAst.characterization() != null &&
            !braneAst.canonicalCharacterization().isEmpty()) {
            sb.append(indent(depth)).append(braneAst.canonicalCharacterization()).append("'");
        } else {
            sb.append(indent(depth));
        }

        sb.append("{\n");

        for (FIR expr : brane.getExpressionFiroes()) {
            sb.append(sequence(expr, depth + 1));
            sb.append(";\n");
        }

        sb.append(indent(depth)).append("}");
        return sb.toString();
    }

    @Override
    protected String sequenceValue(ValueFiroe value, int depth) {
        return indent(depth) + value.getValue();
    }

    @Override
    protected String sequenceBinary(BinaryFiroe binary, int depth) {
        // If the binary expression has been fully evaluated, just show the result
        if (!binary.isNye()) {
            return indent(depth) + binary.getValue();
        }
        // Otherwise show the expression structure (shouldn't normally happen in evaluated branes)
        return indent(depth) + binary;
    }

    @Override
    protected String sequenceUnary(UnaryFiroe unary, int depth) {
        // If the unary expression has been fully evaluated, just show the result
        if (!unary.isNye()) {
            return indent(depth) + unary.getValue();
        }
        // Otherwise show the expression structure (shouldn't normally happen in evaluated branes)
        return indent(depth) + unary;
    }

    @Override
    protected String sequenceIf(IfFiroe ifFiroe, int depth) {
        // If the if expression has been fully evaluated, show the result
        if (!ifFiroe.isNye()) {
            FIR result = ifFiroe.getResult();
            if (result != null) {
                return sequence(result, depth);
            }
        }
        // Otherwise show the if structure (shouldn't normally happen in evaluated branes)
        return indent(depth) + "if ???";
    }

    @Override
    protected String sequenceSearchUp(SearchUpFiroe searchUp, int depth) {
        return indent(depth) + "↑";
    }

    /**
     * Sequences an assignment FIR showing the coordinate name and its value.
     */
    protected String sequenceAssignment(AssignmentFiroe assignment, int depth) {
        if (!assignment.isNye() && assignment.getResult() != null) {
            FIR result = assignment.getResult();
            // Check if the result is fully evaluated before getting its value
            if (!result.isNye()) {
                return indent(depth) + assignment.getId() + " = " + result.getValue();
            }
        }
        // If not yet evaluated, show the structure
        return indent(depth) + assignment.getId() + " = ???";
    }

    /**
     * Creates indentation string based on depth.
     *
     * @param depth The depth level
     * @return Indentation string
     */
    private String indent(int depth) {
        return tabChar.repeat(depth);
    }

    /**
     * Gets the tab character used for indentation.
     *
     * @return The tab character
     */
    public String getTabChar() {
        return tabChar;
    }
}
