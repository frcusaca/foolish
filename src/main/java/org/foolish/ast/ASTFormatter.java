package org.foolish.ast;

/**
 * Formatter for AST objects with proper indentation and structure.
 */
public class ASTFormatter {
    private final String indentUnit;
    
    public ASTFormatter(String indentUnit) {
        this.indentUnit = indentUnit;
    }
    
    public ASTFormatter() {
        this("  ");
    }
    
    public String format(AST ast) {
        return format(ast, 0);
    }
    
    private String format(AST ast, int indentLevel) {
        if (ast == null) {
            return "null";
        }
        
        return switch (ast) {
            case AST.Program program -> formatProgram(program, indentLevel);
            case AST.Branes branes -> formatBranes(branes, indentLevel);
            case AST.Brane brane -> formatBrane(brane, indentLevel);
            case AST.Assignment assignment -> formatAssignment(assignment, indentLevel);
            case AST.BinaryExpr binary -> formatBinaryExpr(binary, indentLevel);
            case AST.UnaryExpr unary -> formatUnaryExpr(unary, indentLevel);
            case AST.IfExpr ifExpr -> formatIfExpr(ifExpr, indentLevel);
            case AST.Identifier identifier -> identifier.toString();
            case AST.IntegerLiteral literal -> literal.toString();
            case AST.UnknownExpr unknown -> unknown.toString();
            default -> ast.toString();
        };
    }
    
    private String formatProgram(AST.Program program, int indentLevel) {
        return format(program.branes(), indentLevel);
    }
    
    private String formatBranes(AST.Branes branes, int indentLevel) {
        StringBuilder sb = new StringBuilder();
        boolean first = true;
        for (AST.Brane brane : branes.branes()) {
            if (!first) sb.append("\n");
            sb.append(format(brane, indentLevel));
            first = false;
        }
        return sb.toString();
    }
    
    private String formatBrane(AST.Brane brane, int indentLevel) {
        StringBuilder sb = new StringBuilder();
        String indent = indentUnit.repeat(indentLevel);
        String innerIndent = indentUnit.repeat(indentLevel + 1);
        
        if (brane.characterization() != null && brane.characterization().id() != null && !brane.characterization().id().isEmpty()) {
            sb.append(brane.characterization().id()).append("'");
        }
        sb.append("{\n");
        
        for (AST.Expr expr : brane.statements()) {
            sb.append(innerIndent).append(format(expr, indentLevel + 1)).append(";\n");
        }
        
        sb.append(indent).append("}");
        return sb.toString();
    }
    
    private String formatAssignment(AST.Assignment assignment, int indentLevel) {
        return assignment.id() + " = " + format(assignment.expr(), indentLevel);
    }
    
    private String formatBinaryExpr(AST.BinaryExpr binary, int indentLevel) {
        return "(" + format(binary.left(), indentLevel) + " " + binary.op() + " " + format(binary.right(), indentLevel) + ")";
    }
    
    private String formatUnaryExpr(AST.UnaryExpr unary, int indentLevel) {
        return unary.op() + format(unary.expr(), indentLevel);
    }
    
    private String formatIfExpr(AST.IfExpr ifExpr, int indentLevel) {
        StringBuilder sb = new StringBuilder();
        sb.append("if ").append(format(ifExpr.condition(), indentLevel));
        sb.append(" then ").append(format(ifExpr.thenExpr(), indentLevel));
        
        for (AST.IfExpr elseIf : ifExpr.elseIfs()) {
            sb.append(" elif ").append(format(elseIf.condition(), indentLevel));
            sb.append(" then ").append(format(elseIf.thenExpr(), indentLevel));
        }
        
        if (ifExpr.elseExpr() != null && ifExpr.elseExpr() != AST.UnknownExpr.INSTANCE) {
            sb.append(" else ").append(format(ifExpr.elseExpr(), indentLevel));
        }
        
        return sb.toString();
    }
}