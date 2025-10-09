package org.foolish.fvm;

/**
 * Configurable formatter for Targoe objects with support for both verbose and human-readable output.
 */
public class TargoeFormatter {
    private final boolean verbose;
    private final String indentUnit;
    private final boolean showProgressHeap;
    
    private TargoeFormatter(Builder builder) {
        this.verbose = builder.verbose;
        this.indentUnit = builder.indentUnit;
        this.showProgressHeap = builder.showProgressHeap;
    }
    
    public String format(Targoe targoe) {
        return format(targoe, 0);
    }
    
    private String format(Targoe targoe, int indentLevel) {
        if (targoe == null) {
            return "null";
        }
        
        String indent = indentUnit.repeat(indentLevel);
        
        if (targoe instanceof Finear finear) {
            return formatFinear(finear);
        } else if (targoe instanceof BraneMidoe brane) {
            return formatBraneMidoe(brane, indentLevel);
        } else if (targoe instanceof AssignmentMidoe assignment) {
            return formatAssignmentMidoe(assignment, indentLevel);
        } else if (targoe instanceof IfMidoe ifMidoe) {
            return formatIfMidoe(ifMidoe, indentLevel);
        } else if (targoe instanceof BinaryMidoe binary) {
            return formatBinaryMidoe(binary, indentLevel);
        } else if (targoe instanceof UnaryMidoe unary) {
            return formatUnaryMidoe(unary, indentLevel);
        } else if (targoe instanceof IdentifierMidoe identifier) {
            return formatIdentifierMidoe(identifier);
        } else if (targoe instanceof ProgramMidoe program) {
            return formatProgramMidoe(program, indentLevel);
        } else if (targoe instanceof Midoe midoe) {
            return formatGenericMidoe(midoe, indentLevel);
        } else {
            return formatGenericTargoe(targoe, indentLevel);
        }
    }
    
    private String formatFinear(Finear finear) {
        if (verbose) {
            return "Finear(" + (finear.isUnknown() ? "UNKNOWN" : finear.value()) + ")";
        } else {
            return finear.isUnknown() ? "UNKNOWN" : String.valueOf(finear.value());
        }
    }
    
    private String formatBraneMidoe(BraneMidoe brane, int indentLevel) {
        if (verbose) {
            if (brane.statements().isEmpty()) {
                return "MidoeBrane()";
            }
            StringBuilder sb = new StringBuilder("MidoeBrane(\n");
            String indent = indentUnit.repeat(indentLevel + 1);
            boolean first = true;
            for (Midoe stmt : brane.statements()) {
                if (!first) sb.append(",\n");
                sb.append(indent).append(format(stmt, indentLevel + 1));
                first = false;
            }
            sb.append("\n").append(indentUnit.repeat(indentLevel)).append(")");
            return sb.toString();
        } else {
            if (brane.statements().isEmpty()) {
                return "{}";
            }
            StringBuilder sb = new StringBuilder("{\n");
            String indent = indentUnit.repeat(indentLevel + 1);
            for (Midoe stmt : brane.statements()) {
                sb.append(indent).append(format(stmt, indentLevel + 1)).append(";\n");
            }
            sb.append(indentUnit.repeat(indentLevel)).append("}");
            return sb.toString();
        }
    }
    
    private String formatAssignmentMidoe(AssignmentMidoe assignment, int indentLevel) {
        if (verbose) {
            return "MidoeAssignment(" + assignment.id() + ", " + format(assignment.expr(), indentLevel) + ")";
        } else {
            return assignment.id() + " = " + format(assignment.expr(), indentLevel);
        }
    }
    
    private String formatIfMidoe(IfMidoe ifMidoe, int indentLevel) {
        if (verbose) {
            StringBuilder sb = new StringBuilder("MidoeIf(\n");
            String indent = indentUnit.repeat(indentLevel + 1);
            sb.append(indent).append(format(ifMidoe.condition(), indentLevel + 1));
            sb.append(",\n").append(indent).append(format(ifMidoe.thenExpr(), indentLevel + 1));
            
            for (IfMidoe elif : ifMidoe.elseIfs()) {
                sb.append(",\n").append(indent).append(format(elif, indentLevel + 1));
            }
            
            if (ifMidoe.elseExpr() != null) {
                sb.append(",\n").append(indent).append(format(ifMidoe.elseExpr(), indentLevel + 1));
            }
            sb.append("\n").append(indentUnit.repeat(indentLevel)).append(")");
            return sb.toString();
        } else {
            StringBuilder sb = new StringBuilder("if ");
            sb.append(format(ifMidoe.condition(), indentLevel));
            sb.append(" then ").append(format(ifMidoe.thenExpr(), indentLevel));
            
            for (IfMidoe elif : ifMidoe.elseIfs()) {
                sb.append(" else ").append(format(elif, indentLevel));
            }
            
            if (ifMidoe.elseExpr() != null) {
                sb.append(" else ").append(format(ifMidoe.elseExpr(), indentLevel));
            }
            return sb.toString();
        }
    }
    
    private String formatBinaryMidoe(BinaryMidoe binary, int indentLevel) {
        if (verbose) {
            return "MidoeBinary(" + format(binary.left(), indentLevel) + ", " + binary.op() + ", " + format(binary.right(), indentLevel) + ")";
        } else {
            return format(binary.left(), indentLevel) + " " + binary.op() + " " + format(binary.right(), indentLevel);
        }
    }
    
    private String formatUnaryMidoe(UnaryMidoe unary, int indentLevel) {
        if (verbose) {
            return "MidoeUnary(" + unary.op() + ", " + format(unary.expr(), indentLevel) + ")";
        } else {
            return unary.op() + format(unary.expr(), indentLevel);
        }
    }
    
    private String formatIdentifierMidoe(IdentifierMidoe identifier) {
        if (verbose) {
            return "MidoeId(" + identifier.id() + ")";
        } else {
            return identifier.id().toString();
        }
    }
    
    private String formatProgramMidoe(ProgramMidoe program, int indentLevel) {
        if (verbose) {
            return "MidoeProgram(" + format(program.brane(), indentLevel) + ")";
        } else {
            return format(program.brane(), indentLevel);
        }
    }
    
    private String formatGenericMidoe(Midoe midoe, int indentLevel) {
        if (verbose) {
            StringBuilder sb = new StringBuilder(midoe.getClass().getSimpleName() + "(");
            if (showProgressHeap && !midoe.progress_heap().isEmpty()) {
                sb.append("heap: [");
                boolean first = true;
                for (Targoe item : midoe.progress_heap()) {
                    if (!first) sb.append(", ");
                    sb.append(format(item, indentLevel + 1));
                    first = false;
                }
                sb.append("]");
            }
            sb.append(")");
            return sb.toString();
        } else {
            return midoe.getClass().getSimpleName();
        }
    }
    
    private String formatGenericTargoe(Targoe targoe, int indentLevel) {
        if (verbose) {
            return targoe.getClass().getSimpleName() + "()";
        } else {
            return targoe.getClass().getSimpleName();
        }
    }
    
    public static class Builder {
        private boolean verbose = false;
        private String indentUnit = "  ";
        private boolean showProgressHeap = false;
        
        public Builder verbose(boolean verbose) {
            this.verbose = verbose;
            return this;
        }
        
        public Builder indentUnit(String indentUnit) {
            this.indentUnit = indentUnit;
            return this;
        }
        
        public Builder showProgressHeap(boolean showProgressHeap) {
            this.showProgressHeap = showProgressHeap;
            return this;
        }
        
        public TargoeFormatter build() {
            return new TargoeFormatter(this);
        }
    }
    
    public static Builder builder() {
        return new Builder();
    }
    
    public static TargoeFormatter verbose() {
        return builder().verbose(true).build();
    }
    
    public static TargoeFormatter humanReadable() {
        return builder().verbose(false).build();
    }
}