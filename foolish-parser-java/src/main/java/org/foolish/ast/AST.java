package org.foolish.ast;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

public sealed interface AST permits AST.Program, AST.Expr, AST.DetachmentStatement, AST.BraneRegexpSearch {
    static <T> T setCharacterization(List<String> characterizations, T chrbl) {
        return (T) switch (chrbl) {
            case IntegerLiteral intLit -> new IntegerLiteral(characterizations, intLit.value());
            case Identifier ident -> new Identifier(characterizations, ident.id());
            case Brane brn -> new Brane(characterizations, brn.statements());
            case DetachmentBrane detachment -> new DetachmentBrane(characterizations, detachment.statements());
            case SearchUP searchUp -> new SearchUP(characterizations);
            case null, default -> throw new IllegalArgumentException("Cannot set characterization on type: " + (chrbl == null ? "null" : chrbl.getClass()));
        };
    }

    sealed interface Expr extends AST permits Characterizable, BinaryExpr, UnaryExpr, Branes, IfExpr, UnknownExpr, Stmt, DereferenceExpr, RegexpSearchExpr, SeekExpr, OneShotSearchExpr, ConstanticExpr, SFFExpr {

    }

    sealed interface Characterizable extends Expr permits Literal, Identifier, Brane, DetachmentBrane, SearchUP {
        List<String> characterizations();  // empty list means no characterization

        default String canonicalCharacterization() {
            if (this.characterizations() == null || this.characterizations().isEmpty()) {
                return "";
            }
            return String.join("'", this.characterizations()) + "'";
        }

    }

    sealed interface Literal extends Characterizable permits IntegerLiteral {
    }

    sealed interface Stmt extends Expr permits Assignment, ConfirmStmt {
    }

    record Program(Branes branes) implements AST {
        public String toString() {
            return branes.toString();
        }
    }

    record IntegerLiteral(List<String> characterizations, long value) implements Literal {
        public IntegerLiteral(long value) {
            this(List.of(), value);
        }

        public IntegerLiteral(String chara, long value) {
            this(chara == null || chara.isEmpty() ? List.of() : List.of(chara), value);
        }

        public String toString() {
            return canonicalCharacterization() + value;
        }

        public boolean equals(Object obj) {
            return (obj != null &&
                    obj instanceof IntegerLiteral other && this.value == other.value
                    && this.canonicalCharacterization().equals(other.canonicalCharacterization())
            );
        }
    }

    record Identifier(List<String> characterizations, String id) implements Characterizable {
        public Identifier(String id) {
            this(List.of(), id);
        }

        public String toString() {
            return canonicalCharacterization() + id;
        }

        public String cannonicalId() {
            return this.id == null ? "" : this.id;
        }


        @Override
        public boolean equals(Object obj) {
            return (obj != null &&
                    obj instanceof Identifier other &&
                    this.cannonicalId().equals(other.cannonicalId()) &&
                    this.canonicalCharacterization().equals(other.canonicalCharacterization())
            );
        }
    }

    record Brane(List<String> characterizations, List<Expr> statements) implements Characterizable {
        public Brane(List<Expr> statements) {
            this(List.of(), statements);
        }

        public String toString() {
            var sb = new StringBuilder();
            sb.append(canonicalCharacterization());
            sb.append("{\n");
            for (Expr expr : statements) {
                sb.append("  ").append(expr).append(";\n");
            }
            sb.append("}");
            return sb.toString();
        }

        @Override
        public boolean equals(Object obj) {
            return (obj != null &&
                    obj instanceof Brane other &&
                    this.statements.equals(other.statements) &&
                    this.canonicalCharacterization().equals(other.canonicalCharacterization())
            );
        }
    }

    record DetachmentBrane(List<String> characterizations,
                           List<DetachmentStatement> statements) implements Characterizable {
        public DetachmentBrane(List<DetachmentStatement> statements) {
            this(List.of(), statements);
        }

        public String toString() {
            StringBuilder sb = new StringBuilder();
            sb.append(canonicalCharacterization());
            sb.append("[");
            for (int i = 0; i < statements.size(); i++) {
                sb.append(statements.get(i));
                if (i < statements.size() - 1) {
                    sb.append(", ");
                }
            }
            sb.append("]");
            return sb.toString();
        }

        @Override
        public boolean equals(Object obj) {
            return (obj != null &&
                    obj instanceof DetachmentBrane other &&
                    this.statements.equals(other.statements) &&
                    this.canonicalCharacterization().equals(other.canonicalCharacterization())
            );
        }
    }

    record DetachmentStatement(Identifier identifier, Expr expr, boolean isPBrane, ForwardSearchType forwardSearchType, int seekIndex) implements AST {
        public DetachmentStatement(Identifier identifier, Expr expr) {
            this(identifier, expr, false, ForwardSearchType.NONE, 0);
        }

        public enum ForwardSearchType { NONE, LOCAL, GLOBAL }

        public String toString() {
            if (seekIndex != 0) { // Assuming 0 is default/unused for now, unless seek #0 is valid (it is). But seekIndex is only used if identifier is null?
                // Let's use logic.
            }
            // Logic for formatting:
            // if forwardSearch: ~id or ~~id
            // if seek: #N
            // else: +? id (= expr)?

            if (forwardSearchType == ForwardSearchType.LOCAL) return "~" + identifier;
            if (forwardSearchType == ForwardSearchType.GLOBAL) return "~~" + identifier;
            // Seek case (AST structure might need refinement if identifier is null)
            // If identifier is null, it might be seek?

            String prefix = isPBrane ? "+" : "";
            String idStr = identifier != null ? identifier.toString() : "";
            String exprStr = (expr != null && expr != UnknownExpr.INSTANCE) ? " = " + expr : "";
            if (idStr.isEmpty() && exprStr.isEmpty()) return "";
            return prefix + idStr + exprStr;
        }
    }

    record SearchUP(List<String> characterizations) implements Characterizable {
        public SearchUP() {
            this(List.of());
        }

        public String toString() {
            return canonicalCharacterization() + "↑";
        }

        @Override
        public boolean equals(Object obj) {
            return (obj != null &&
                    obj instanceof SearchUP other &&
                    this.canonicalCharacterization().equals(other.canonicalCharacterization())
            );
        }
    }

    record Branes(List<Expr> branes) implements Expr {
        public String toString() {
            StringBuilder sb = new StringBuilder();
            for (Expr brane : branes) {
                sb.append(brane).append("\n");
            }
            return sb.toString();
        }

        public int size() {
            return branes.size();
        }
    }

    record BinaryExpr(String op, Expr left, Expr right) implements Expr {
        public String toString() {
            return "(" + left + " " + op + " " + right + ")";
        }
    }

    record UnaryExpr(String op, Expr expr) implements Expr {
        public String toString() {
            return op + expr;
        }
    }

    record DereferenceExpr(Expr anchor, Identifier coordinate) implements Expr {
        public String toString() {
            return anchor + "." + coordinate;
        }
    }

    record RegexpSearchExpr(Expr anchor, SearchOperator operator, String pattern) implements Expr {
        public String toString() {
            return anchor + " " + operator + " " + pattern;
        }
    }

    record SeekExpr(Expr anchor, int offset) implements Expr {
        public String toString() {
            return anchor + "#" + offset;
        }
    }

    record OneShotSearchExpr(Expr anchor, SearchOperator operator) implements Expr {
        public String toString() {
            return operator + anchor.toString();
        }
    }

    record ConstanticExpr(Expr expr) implements Expr {
        public String toString() {
            return "<" + expr + ">";
        }
    }

    record SFFExpr(Expr expr) implements Expr {
        public String toString() {
            return "<<" + expr + ">>";
        }
    }

    record Assignment(Identifier identifier, Expr expr) implements Stmt {
        /**
         * Creates an Assignment with a simple uncharacterized identifier.
         * @deprecated Use Assignment(Identifier, Expr) instead
         */
        @Deprecated
        public Assignment(String id, Expr expr) {
            this(new Identifier(id), expr);
        }

        /**
         * Gets the identifier name (without characterization).
         * For compatibility with existing code.
         */
        public String id() {
            return identifier.id();
        }

        public String toString() {
            if (expr instanceof OneShotSearchExpr) {
                return identifier + " =" + expr;
            }
            return identifier + " = " + expr;
        }
    }

    record ConfirmStmt(Expr left, Expr right) implements Stmt {
        public String toString() {
            return "confirm " + left + " == " + right;
        }
    }

    record UnknownExpr() implements Expr {
        public static final UnknownExpr INSTANCE = new UnknownExpr();

        public String toString() {
            return "???";
        }
    }

    record IfExpr(Expr condition, Expr thenExpr, Expr elseExpr, List<IfExpr> elseIfs) implements Expr {
        public IfExpr(Expr condition, Expr thenExpr, Expr elseExpr, List<IfExpr> elseIfs) {
            this.condition = condition;
            this.thenExpr = Objects.requireNonNullElse(thenExpr, UnknownExpr.INSTANCE);
            this.elseExpr = Objects.requireNonNullElse(elseExpr, UnknownExpr.INSTANCE);
            this.elseIfs = Objects.requireNonNullElse(elseIfs, new ArrayList<>());
            for (IfExpr elseIf : this.elseIfs) {
                assert (elseIf.elseExpr == null ||
                        elseIf.elseExpr == UnknownExpr.INSTANCE) &&
                        elseIf.elseIfs.isEmpty() : "elif if cannot have else or elif";
            }
        }

        public String toString() {
            StringBuilder sb = new StringBuilder();
            sb.append("if ").append(condition).append(" then ").append(thenExpr);
            for (IfExpr elseIf : elseIfs) {
                sb.append(" elif ").append(elseIf.condition()).append(" then ").append(elseIf.thenExpr());
            }
            if (elseExpr != null && elseExpr != UnknownExpr.INSTANCE) {
                sb.append(" else ").append(elseExpr);
            }
            return sb.toString();
        }
    }

    record BraneRegexpSearch(Characterizable brane, String operator, String pattern) implements AST {
        public String toString() {
            return brane + " " + operator + " " + pattern;
        }

        @Override
        public boolean equals(Object obj) {
            return (obj != null &&
                    obj instanceof BraneRegexpSearch other &&
                    this.brane.equals(other.brane) &&
                    this.operator.equals(other.operator) &&
                    this.pattern.equals(other.pattern)
            );
        }
    }
}
