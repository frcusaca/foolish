package org.foolish.fvm.ubc;

import org.foolish.ast.AST;

/**
 * Human-friendly sequencer that formats FIR objects with configurable indentation.
 */
public class Sequencer4Human extends Sequencer<String> {
    private static final String DEFAULT_TAB = "＿"; //U+FF3F
    public static final String NK_STR = "???";
    public static final String CC_STR = "⎵⎵";
    private final String tabChar;

    public Sequencer4Human(String tabChar) {
        this.tabChar = tabChar;
    }

    public Sequencer4Human() {
        this(DEFAULT_TAB);
    }

    public String sequence(FIR fir) {
        return sequence(fir, 0);
    }

    public String sequence(FIR fir, int depth) {
        return switch (fir) {
            case BraneFiroe brane -> sequenceBrane(brane, depth);
            case ConcatenationFiroe concat -> sequenceConcatenation(concat, depth);
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

    protected String sequenceConcatenation(ConcatenationFiroe concat, int depth) {
        // Concatenation is displayed as merged brane content
        var sb = new StringBuilder();
        sb.append(indent(depth)).append("{\n");

        concat.stream().forEach(fir -> {
            sb.append(sequence(fir, depth + 1));
            sb.append(";\n");
        });

        sb.append(indent(depth)).append("}");
        return sb.toString();
    }

    protected String sequenceValue(ValueFiroe value, int depth) {
        return indent(depth) + value.getValue();
    }

    protected String sequenceBinary(BinaryFiroe binary, int depth) {
        if (!binary.isNye()) {
            if (binary.atConstanic()) {
                return indent(depth) + CC_STR;
            }
            try {
                return indent(depth) + binary.getValue();
            } catch (IllegalStateException e) {
                return indent(depth) + NK_STR;
            }
        }
        return indent(depth) + binary;
    }

    protected String sequenceUnary(UnaryFiroe unary, int depth) {
        if (!unary.isNye()) {
            if (unary.atConstanic()) {
                return indent(depth) + "⎵";
            }
            return indent(depth) + unary.getValue();
        }
        return indent(depth) + unary;
    }

    protected String sequenceIf(IfFiroe ifFiroe, int depth) {
        if (!ifFiroe.isNye()) {
            FIR result = ifFiroe.getResult();
            if (result != null) {
                return sequence(result, depth);
            }
        }
        return indent(depth) + "if " + NK_STR;
    }

    protected String sequenceSearchUp(SearchUpFiroe searchUp, int depth) {
        return indent(depth) + "↑";
    }

    protected String sequenceAssignment(AssignmentFiroe assignment, int depth) {
        String fullId = assignment.getLhs().toString();
        if (!assignment.isNye() && assignment.getResult() != null) {
            FIR result = assignment.getResult();
            if (!result.isNye()) {
                if (result instanceof NKFiroe) {
                    return indent(depth) + fullId + " = " + NK_STR;
                }

                FIR unwrapped = unwrap(result);

                if (unwrapped != null && unwrapped.atConstanic() && !(unwrapped instanceof BraneFiroe)) {
                    return indent(depth) + fullId + " = " + CC_STR;
                }

                if (unwrapped instanceof NKFiroe) {
                    return indent(depth) + fullId + " = " + NK_STR;
                }

                if (unwrapped instanceof BraneFiroe brane) {
                    String braneSeq = sequenceBrane(brane, depth);
                    String indent = indent(depth);
                    if (braneSeq.startsWith(indent)) {
                        braneSeq = braneSeq.substring(indent.length());
                    }
                    String padding = " ".repeat(fullId.length() + 3);
                    String nestedIndent = indent(depth + 1);
                    String parentIndent = indent(depth);
                    braneSeq = braneSeq.replace("\n" + nestedIndent, "\n" + parentIndent + padding + tabChar);
                    return indent(depth) + fullId + " = " + braneSeq;
                }
                if (unwrapped instanceof ConcatenationFiroe concat) {
                    String concatSeq = sequenceConcatenation(concat, depth);
                    String indent = indent(depth);
                    if (concatSeq.startsWith(indent)) {
                        concatSeq = concatSeq.substring(indent.length());
                    }
                    String padding = " ".repeat(fullId.length() + 3);
                    String nestedIndent = indent(depth + 1);
                    String parentIndent = indent(depth);
                    concatSeq = concatSeq.replace("\n" + nestedIndent, "\n" + parentIndent + padding + tabChar);
                    return indent(depth) + fullId + " = " + concatSeq;
                }
                if (unwrapped != null) {
                    try {
                        return indent(depth) + fullId + " = " + unwrapped.getValue();
                    } catch (IllegalStateException e) {
                        // Fallback if value cannot be retrieved (e.g., still NYE or NK inside CMFir)
                        return indent(depth) + fullId + " = " + NK_STR;
                    }
                }
            }
        } else if (assignment.atConstanic()) {
             return indent(depth) + fullId + " = " + CC_STR;
        }
        return indent(depth) + fullId + " = " + NK_STR;
    }

    /**
     * Unwraps wrapper FIRs to get the underlying value.
     * Always expands branes even if constanic.
     */
    private FIR unwrap(FIR fir) {
        FIR current = fir;
        while (current != null) {
            if (current instanceof BraneFiroe || current instanceof ConcatenationFiroe) {
                return current;
            }

            switch (current) {
                case IdentifierFiroe id -> {
                    if (id.value == null) return id;
                    current = id.value;
                }
                case AssignmentFiroe assign -> {
                    if (assign.getResult() == null) return assign;
                    current = assign.getResult();
                }
                case AbstractSearchFiroe search -> {
                    if (search.getResult() == null) return search;
                    current = search.getResult();
                }
                case UnanchoredSeekFiroe seek -> {
                    if (seek.getResult() == null) return seek;
                    current = seek.getResult();
                }
                default -> {
                    return current;
                }
            }
        }
        return current;
    }

    protected String sequenceIdentifier(IdentifierFiroe identifier, int depth) {
        if (identifier.atConstanic()) {
            return indent(depth) + CC_STR;
        }
        if (!identifier.isNye()) {
            // If the identifier resolved to a brane or concatenation, sequence that
            FIR resolved = identifier.getResolvedFir();
            if (resolved instanceof BraneFiroe brane) {
                return sequenceBrane(brane, depth);
            }
            if (resolved instanceof ConcatenationFiroe concat) {
                return sequenceConcatenation(concat, depth);
            }
            // Unwrap through assignments to find the actual value
            FIR unwrapped = unwrap(resolved);
            if (unwrapped instanceof BraneFiroe brane) {
                return sequenceBrane(brane, depth);
            }
            if (unwrapped instanceof ConcatenationFiroe concat) {
                return sequenceConcatenation(concat, depth);
            }
            try {
                return indent(depth) + identifier.getValue();
            } catch (UnsupportedOperationException e) {
                // Fall through to NK if getValue fails
            }
        }
        return indent(depth) + NK_STR;
    }

    protected String sequenceNK(FIR nk, int depth) {
        return indent(depth) + NK_STR;
    }

    protected String sequenceSearch(AbstractSearchFiroe search, int depth) {
        if (!search.isNye()) {
            if (!search.isFound()) {
                return indent(depth) + CC_STR;
            }
            if (search.atConstant()) {
                try {
                    return indent(depth) + search.getValue();
                } catch (UnsupportedOperationException e) {
                    return sequence(search.getResult(), depth);
                }
            }
            return indent(depth) + CC_STR;
        }
        return indent(depth) + NK_STR;
    }

    private String indent(int depth) {
        return tabChar.repeat(depth);
    }

    public String getTabChar() {
        return tabChar;
    }
}
