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
        return switch (fir) {
            case BraneFiroe brane -> sequenceBrane(brane, depth);
            case NKFiroe nk -> sequenceNK(nk, depth);
            case ValueFiroe value -> sequenceValue(value, depth);
            case BinaryFiroe binary -> sequenceBinary(binary, depth);
            case UnaryFiroe unary -> sequenceUnary(unary, depth);
            case IfFiroe ifFiroe -> sequenceIf(ifFiroe, depth);
            case SearchUpFiroe searchUp -> sequenceSearchUp(searchUp, depth);
            case AssignmentFiroe assignment -> sequenceAssignment(assignment, depth);
            case IdentifierFiroe identifier -> sequenceIdentifier(identifier, depth);
            case null, default -> indent(depth) + "???";
        };
    }

    @Override
    protected String sequenceBrane(BraneFiroe brane, int depth) {
        var sb = new StringBuilder();

        // Add characterization (name) if present
        if (brane.ast() instanceof AST.Brane braneAst
            && !braneAst.canonicalCharacterization().isEmpty()) {
            sb.append(indent(depth)).append(braneAst.canonicalCharacterization());
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
        // If the binary expression has been fully evaluated
        if (!binary.isNye()) {
            // Check if the result is NK (not-known)
            if (binary.isAbstract()) {
                return indent(depth) + "???";
            }
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
            // Check if the result is fully evaluated
            if (!result.isNye()) {
                // Check if the result is NK (not-known)
                if (result.isAbstract()) {
                    return indent(depth) + assignment.getId() + " = ???";
                }
                if (result instanceof BraneFiroe brane) {
                    // For nested branes, recursively sequence them but remove the indentation from the first line
                    // since we are already indenting 'id = '
                    String braneSeq = sequenceBrane(brane, depth);
                    // Remove the leading indentation from the brane sequence
                    String indent = indent(depth);
                    if (braneSeq.startsWith(indent)) {
                        braneSeq = braneSeq.substring(indent.length());
                    }
                    return indent(depth) + assignment.getId() + " = " + braneSeq;
                }
                return indent(depth) + assignment.getId() + " = " + result.getValue();
            }
        }
        // If not yet evaluated, show the structure
        return indent(depth) + assignment.getId() + " = ???";
    }

    /**
     * Sequences an identifier FIR showing its resolved value.
     */
    protected String sequenceIdentifier(IdentifierFiroe identifier, int depth) {
        // If the identifier has been resolved and is not NYE
        if (!identifier.isNye()) {
            // Check if it resolved to an abstract value
            if (identifier.isAbstract()) {
                return indent(depth) + "???";
            }
            return indent(depth) + identifier.getValue();
        }
        // If not yet evaluated
        return indent(depth) + "???";
    }

    /**
     * Sequences an NK (not-known) FIR.
     */
    protected String sequenceNK(FIR nk, int depth) {
        return indent(depth) + "???";
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
