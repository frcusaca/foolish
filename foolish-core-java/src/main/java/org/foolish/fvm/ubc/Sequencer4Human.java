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
    private boolean nyesStateInOutput = true; // Default is on

    public Sequencer4Human(String tabChar) {
        this.tabChar = tabChar;
    }

    public Sequencer4Human() {
        this(DEFAULT_TAB);
    }

    public void setNyesStateInOutput(boolean enabled) {
        this.nyesStateInOutput = enabled;
    }

    public boolean isNyesStateInOutputEnabled() {
        return nyesStateInOutput;
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
        // Rendering depends on Nyes state:
        // - If at least PRIMED: render as flat brane - flatten sub-brane contents into single brane
        // - If not PRIMED: render as {..} {..} with single-space separators showing proximity

        if (concat.getNyes().ordinal() >= Nyes.PRIMED.ordinal()) {
            // At least PRIMED - render as flat brane with flattened contents
            var sb = new StringBuilder();
            sb.append(indent(depth)).append("{\n");

            // Flatten: for each sub-brane, render its contents directly (not the brane wrapper)
            concat.stream().forEach(fir -> {
                FIR unwrapped = unwrap(fir);
                if (unwrapped instanceof FiroeWithBraneMind fwbm) {
                    // Flatten this sub-brane's contents into the parent
                    fwbm.stream().forEach(innerFir -> {
                        sb.append(sequence(innerFir, depth + 1));
                        sb.append(";\n");
                    });
                } else {
                    // Non-brane items (values, etc.) render directly
                    sb.append(sequence(fir, depth + 1));
                    sb.append(";\n");
                }
            });

            sb.append(indent(depth)).append("}");
            return sb.toString();
        }

        // Not yet PRIMED - render elements with single-space separators
        // This shows the proximity of concatenated elements
        var sb = new StringBuilder();
        sb.append(indent(depth));

        boolean first = true;
        for (var firIter = concat.stream().iterator(); firIter.hasNext(); ) {
            FIR fir = firIter.next();
            if (!first) {
                sb.append(" ");  // Single space separator for proximity
            }
            first = false;

            // Render each element inline (compact format)
            String element = sequenceInline(fir);
            sb.append(element);
        }

        return sb.toString();
    }

    /**
     * Sequences a FIR for inline display (used in concatenation proximity rendering).
     * Branes are rendered as {...} without internal expansion.
     */
    private String sequenceInline(FIR fir) {
        return switch (fir) {
            case BraneFiroe brane -> "{...}";
            case ConcatenationFiroe concat -> "{...}";
            case ValueFiroe value -> String.valueOf(value.getValue());
            case IdentifierFiroe id -> {
                if (id.isConstant()) {
                    try {
                        yield String.valueOf(id.getValue());
                    } catch (UnsupportedOperationException e) {
                        FIR resolved = id.getResolvedFir();
                        if (resolved instanceof BraneFiroe || resolved instanceof ConcatenationFiroe) {
                            yield "{...}";
                        }
                        yield CC_STR;
                    }
                } else if (id.atConstanic()) {
                    yield addNyesStateIfEnabled(CC_STR, id.getNyes());
                }
                yield NK_STR;
            }
            case null, default -> NK_STR;
        };
    }


    protected String sequenceValue(ValueFiroe value, int depth) {
        return indent(depth) + value.getValue();
    }

    protected String sequenceBinary(BinaryFiroe binary, int depth) {
        if (!binary.isNye()) {
            if (binary.atConstanic()) {
                return indent(depth) + addNyesStateIfEnabled(CC_STR, binary.getNyes());
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
                return indent(depth) + addNyesStateIfEnabled("⎵", unary.getNyes());
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
                    return indent(depth) + fullId + " = " + addNyesStateIfEnabled(CC_STR, unwrapped.getNyes());
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
             return indent(depth) + fullId + " = " + addNyesStateIfEnabled(CC_STR, assignment.getNyes());
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
            return indent(depth) + addNyesStateIfEnabled(CC_STR, identifier.getNyes());
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
            return indent(depth) + addNyesStateIfEnabled(CC_STR, search.getNyes());
        }
        return indent(depth) + NK_STR;
    }

    private String indent(int depth) {
        return tabChar.repeat(depth);
    }

    public String getTabChar() {
        return tabChar;
    }
    
    /**
     * Adds Nyes state to string if enabled
     */
    private String addNyesStateIfEnabled(String value, Nyes nyes) {
        if (nyesStateInOutput) {
            return value + " (" + nyes + ")";
        }
        return value;
    }
}
