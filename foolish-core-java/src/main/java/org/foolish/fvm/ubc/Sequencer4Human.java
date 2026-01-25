package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Human-friendly sequencer that formats FIR objects with configurable indentation.
 * Uses '➠' as the default tab character for visual clarity.
 */
public class Sequencer4Human extends Sequencer<String> {
    private static final String DEFAULT_TAB = "＿"; //U+FF3F
    public static final String NK_STR = "???";
    public static final String CC_STR = "⎵⎵";
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

    public String sequence(FIR fir) {
        return sequence(fir, 0);
    }

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
            case AbstractSearchFiroe search -> sequenceSearch(search, depth);
            case null, default -> indent(depth) + NK_STR;
        };
    }

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

    protected String sequenceValue(ValueFiroe value, int depth) {
        return indent(depth) + value.getValue();
    }

    protected String sequenceBinary(BinaryFiroe binary, int depth) {
        // If the binary expression has been fully evaluated
        if (!binary.isNye()) {
            // Check if the result is Constanic (unresolved)
            if (binary.atConstanic()) {
                return indent(depth) + CC_STR;
            }
            // Try to get the value - may throw if result is NK (error like division by zero)
            try {
                return indent(depth) + binary.getValue();
            } catch (IllegalStateException e) {
                // NK result (division by zero, etc.)
                return indent(depth) + NK_STR;
            }
        }
        // Otherwise show the expression structure (shouldn't normally happen in evaluated branes)
        return indent(depth) + binary;
    }

    protected String sequenceUnary(UnaryFiroe unary, int depth) {
        // If the unary expression has been fully evaluated, just show the result
        if (!unary.isNye()) {
            if (unary.atConstanic()) {
                // Constanic - show placeholder
                return indent(depth) + "⎵";
            }
            return indent(depth) + unary.getValue();
        }
        // Otherwise show the expression structure (shouldn't normally happen in evaluated branes)
        return indent(depth) + unary;
    }

    protected String sequenceIf(IfFiroe ifFiroe, int depth) {
        // If the if expression has been fully evaluated, show the result
        if (!ifFiroe.isNye()) {
            FIR result = ifFiroe.getResult();
            if (result != null) {
                return sequence(result, depth);
            }
        }
        // Otherwise show the if structure (shouldn't normally happen in evaluated branes)
        return indent(depth) + "if " + NK_STR;
    }

    protected String sequenceSearchUp(SearchUpFiroe searchUp, int depth) {
        return indent(depth) + "↑";
    }

    /**
     * Sequences an assignment FIR showing the coordinate name and its value.
     */
    protected String sequenceAssignment(AssignmentFiroe assignment, int depth) {
        String fullId = assignment.getLhs().toString();
        if (!assignment.isNye() && assignment.getResult() != null) {
            FIR result = assignment.getResult();
            // Check if the result is fully evaluated
            if (!result.isNye()) {
                // Use atConstanic() to check for exactly CONSTANIC state (unresolved)
                // not CONSTANT state (fully evaluated)
                if (result.atConstanic()) {
                    return indent(depth) + fullId + " = " + CC_STR;
                }

                // Check if result is directly an NK (error like division by zero)
                if (result instanceof NKFiroe) {
                    return indent(depth) + fullId + " = " + NK_STR;
                }

                // Unwrap identifier/assignment/oneshot to get the actual value
                FIR unwrapped = result;
                boolean constanicFound = false;

                unwrappingLoop:
                while (true) {
                    if (unwrapped == null) break;
                    if (unwrapped.atConstanic()) {
                        constanicFound = true;
                        break;
                    }

                    switch (unwrapped) {
                        case IdentifierFiroe identifierFiroe -> {
                            unwrapped = identifierFiroe.value;
                        }
                        case AssignmentFiroe assignmentFiroe -> {
                            unwrapped = assignmentFiroe.getResult();
                        }
                        case AbstractSearchFiroe searchFiroe ->
                             unwrapped = searchFiroe.getResult();
                        default -> { break unwrappingLoop; }
                    }
                }

                if (constanicFound) {
                    return indent(depth) + fullId + " = " + CC_STR;
                }

                if (unwrapped instanceof NKFiroe) {
                    return indent(depth) + fullId + " = " + NK_STR;
                }

                if (unwrapped instanceof BraneFiroe brane) {
                    // For nested branes, recursively sequence them but remove the indentation from the first line
                    // since we are already indenting 'id = '
                    String braneSeq = sequenceBrane(brane, depth);
                    // Remove the leading indentation from the brane sequence
                    String indent = indent(depth);
                    if (braneSeq.startsWith(indent)) {
                        braneSeq = braneSeq.substring(indent.length());
                    }
                    // For subsequent lines, add spaces to align with "id = "
                    String padding = " ".repeat(fullId.length() + 3);
                    braneSeq = braneSeq.replace("\n", "\n" + padding);
                    return indent(depth) + fullId + " = " + braneSeq;
                }
                return indent(depth) + fullId + " = " + unwrapped.getValue();
            }
        } else if (assignment.atConstanic()) {
             return indent(depth) + fullId + " = " + CC_STR;
        }
        // If not yet evaluated, show the structure
        return indent(depth) + fullId + " = " + NK_STR;
    }

    /**
     * Sequences an identifier FIR showing its resolved value.
     */
    protected String sequenceIdentifier(IdentifierFiroe identifier, int depth) {
        if (identifier.atConstanic()) {
            return indent(depth) + CC_STR;
        }

        // If the identifier has been resolved and is not NYE
        if (!identifier.isNye()) {
            return indent(depth) + identifier.getValue();
        }
        // If not yet evaluated
        return indent(depth) + NK_STR;
    }

    /**
     * Sequences an NK (not-known) FIR.
     */
    protected String sequenceNK(FIR nk, int depth) {
        return indent(depth) + NK_STR;
    }

    protected String sequenceSearch(AbstractSearchFiroe search, int depth) {
        if (!search.isNye()) {
            // Use atConstanic() to check for exactly CONSTANIC state (unresolved)
            if (search.atConstanic()) {
                return indent(depth) + CC_STR;
            }
            // If the result is a brane, we need to handle it gracefully
            try {
                return indent(depth) + search.getValue();
            } catch (UnsupportedOperationException e) {
                // It might be a brane or something else that doesn't support getValue()
                // Use sequence() recursively on the result if we can access it
                return sequence(search.getResult(), depth);
            }
        }
        return indent(depth) + NK_STR;
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
