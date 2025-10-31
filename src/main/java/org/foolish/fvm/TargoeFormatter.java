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
        } else if (targoe instanceof BraneFiroe brane) {
            return formatBraneFiroe(brane, indentLevel);
        } else if (targoe instanceof AssignmentFiroe assignment) {
            return formatAssignmentFiroe(assignment, indentLevel);
        } else if (targoe instanceof IfFiroe ifFiroe) {
            return formatIfFiroe(ifFiroe, indentLevel);
        } else if (targoe instanceof BinaryFiroe binary) {
            return formatBinaryFiroe(binary, indentLevel);
        } else if (targoe instanceof UnaryFiroe unary) {
            return formatUnaryFiroe(unary, indentLevel);
        } else if (targoe instanceof IdentifierFiroe identifier) {
            return formatIdentifierFiroe(identifier);
        } else if (targoe instanceof SearchUpFiroe searchUp) {
            return formatSearchUpFiroe(searchUp, indentLevel);
        } else if (targoe instanceof ProgramFiroe program) {
            return formatProgramFiroe(program, indentLevel);
        } else if (targoe instanceof Firoe firoe) {
            return formatGenericFiroe(firoe, indentLevel);
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
    
    private String formatBraneFiroe(BraneFiroe brane, int indentLevel) {
        if (verbose) {
            if (brane.statements().isEmpty()) {
                return "FiroeBrane()";
            }
            StringBuilder sb = new StringBuilder("FiroeBrane(\n");
            String indent = indentUnit.repeat(indentLevel + 1);
            boolean first = true;
            for (Firoe stmt : brane.statements()) {
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
            for (Firoe stmt : brane.statements()) {
                sb.append(indent).append(format(stmt, indentLevel + 1)).append(";\n");
            }
            sb.append(indentUnit.repeat(indentLevel)).append("}");
            return sb.toString();
        }
    }
    
    private String formatAssignmentFiroe(AssignmentFiroe assignment, int indentLevel) {
        if (verbose) {
            return "FiroeAssignment(" + assignment.id() + ", " + format(assignment.expr(), indentLevel) + ")";
        } else {
            return assignment.id() + " = " + format(assignment.expr(), indentLevel);
        }
    }
    
    private String formatIfFiroe(IfFiroe ifFiroe, int indentLevel) {
        if (verbose) {
            StringBuilder sb = new StringBuilder("FiroeIf(\n");
            String indent = indentUnit.repeat(indentLevel + 1);
            sb.append(indent).append(format(ifFiroe.condition(), indentLevel + 1));
            sb.append(",\n").append(indent).append(format(ifFiroe.thenExpr(), indentLevel + 1));
            
            for (IfFiroe elif : ifFiroe.elseIfs()) {
                sb.append(",\n").append(indent).append(format(elif, indentLevel + 1));
            }
            
            if (ifFiroe.elseExpr() != null) {
                sb.append(",\n").append(indent).append(format(ifFiroe.elseExpr(), indentLevel + 1));
            }
            sb.append("\n").append(indentUnit.repeat(indentLevel)).append(")");
            return sb.toString();
        } else {
            StringBuilder sb = new StringBuilder("if ");
            sb.append(format(ifFiroe.condition(), indentLevel));
            sb.append(" then ").append(format(ifFiroe.thenExpr(), indentLevel));
            
            for (IfFiroe elif : ifFiroe.elseIfs()) {
                sb.append(" else ").append(format(elif, indentLevel));
            }
            
            if (ifFiroe.elseExpr() != null) {
                sb.append(" else ").append(format(ifFiroe.elseExpr(), indentLevel));
            }
            return sb.toString();
        }
    }
    
    private String formatBinaryFiroe(BinaryFiroe binary, int indentLevel) {
        if (verbose) {
            return "FiroeBinary(" + format(binary.left(), indentLevel) + ", " + binary.op() + ", " + format(binary.right(), indentLevel) + ")";
        } else {
            return format(binary.left(), indentLevel) + " " + binary.op() + " " + format(binary.right(), indentLevel);
        }
    }
    
    private String formatUnaryFiroe(UnaryFiroe unary, int indentLevel) {
        if (verbose) {
            return "FiroeUnary(" + unary.op() + ", " + format(unary.expr(), indentLevel) + ")";
        } else {
            return unary.op() + format(unary.expr(), indentLevel);
        }
    }
    
    private String formatIdentifierFiroe(IdentifierFiroe identifier) {
        if (verbose) {
            return "FiroeId(" + identifier.id() + ")";
        } else {
            return identifier.id().toString();
        }
    }

    private String formatSearchUpFiroe(SearchUpFiroe searchUp, int indentLevel) {
        if (verbose) {
            String parentInfo = searchUp.parent() != null ? " parent=" + searchUp.parent().getClass().getSimpleName() : " parent=null";
            return "FiroeSearchUp(" + parentInfo + ")";
        } else {
            return "↑";
        }
    }

    private String formatProgramFiroe(ProgramFiroe program, int indentLevel) {
        if (verbose) {
            return "FiroeProgram(" + format(program.brane(), indentLevel) + ")";
        } else {
            return format(program.brane(), indentLevel);
        }
    }
    
    private String formatGenericFiroe(Firoe firoe, int indentLevel) {
        if (verbose) {
            StringBuilder sb = new StringBuilder(firoe.getClass().getSimpleName() + "(");
            if (showProgressHeap && !firoe.progress_heap().isEmpty()) {
                sb.append("heap: [");
                boolean first = true;
                for (Targoe item : firoe.progress_heap()) {
                    if (!first) sb.append(", ");
                    sb.append(format(item, indentLevel + 1));
                    first = false;
                }
                sb.append("]");
            }
            sb.append(")");
            return sb.toString();
        } else {
            return firoe.getClass().getSimpleName();
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