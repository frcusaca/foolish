package org.foolish.ast;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

public sealed interface AST permits AST.Program, AST.Expr, AST.DetachmentStatement {
    static <T> T setCharacterization(String id, T chrbl) {
        Identifier identifier = new Identifier(id == null ? "" : id);
        if (chrbl instanceof IntegerLiteral intLit) {
            return (T) (new IntegerLiteral(identifier, intLit.value()));
        } else if (chrbl instanceof Identifier ident) {
            return (T) (new Identifier(identifier, ident.id()));
        } else if (chrbl instanceof Brane brn) {
            return (T) (new Brane(identifier, brn.statements()));
        } else if (chrbl instanceof DetachmentBrane detachment) {
            return (T) (new DetachmentBrane(identifier, detachment.statements()));
        } else if (chrbl instanceof SearchUP searchUp) {
            return (T) (new SearchUP(identifier));
        } else {
            throw new IllegalArgumentException("Cannot set characterization on type: " + chrbl.getClass());
        }
    }

    sealed interface Expr extends AST permits Characterizable, BinaryExpr, UnaryExpr, Branes, IfExpr, UnknownExpr, Stmt {

    }

    sealed interface Characterizable extends Expr permits Literal, Identifier, Brane, DetachmentBrane, SearchUP {
        Identifier characterization();  // null means no characterization

        default String canonicalCharacterization() {
            return this.characterization() == null ? "" : this.characterization().cannonicalId();
        }

    }

    sealed interface Literal extends Characterizable permits IntegerLiteral {
    }

    sealed interface Stmt extends Expr permits Assignment {
    }

    record Program(Branes branes) implements AST {
        public String toString() {
            return branes.toString();
        }
    }

    record IntegerLiteral(Identifier characterization, long value) implements Literal {
        public IntegerLiteral(long value) {
            this("", value);
        }

        public IntegerLiteral(String chara, long value) {
            this(new Identifier(chara == null ? "" : chara), value);
        }

        public String toString() {
            return (((characterization != null && characterization.id.length() > 0) ? characterization.id + "'" : "")
                    + value);
        }

        public boolean equals(Object obj) {
            return (obj != null &&
                    obj instanceof IntegerLiteral other && this.value == other.value
                    && this.canonicalCharacterization().equals(other.canonicalCharacterization())
            );
        }
    }

    record Identifier(Identifier characterization, String id) implements Characterizable {
        public Identifier(String id) {
            this(null, id);
        }

        public String toString() {
            return (((characterization != null && characterization.id.length() > 0) ? characterization.id + "'" : "")
                    + id);
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

    record Brane(Identifier characterization, List<Expr> statements) implements Characterizable {
        public Brane(List<Expr> statements) {
            this(null, statements);
        }

        public String toString() {
            StringBuilder sb = new StringBuilder();
            if (canonicalCharacterization() != "") {
                sb.append(characterization.id).append("'");
            }
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

    record DetachmentBrane(Identifier characterization,
                           List<DetachmentStatement> statements) implements Characterizable {
        public DetachmentBrane(List<DetachmentStatement> statements) {
            this(null, statements);
        }

        public String toString() {
            StringBuilder sb = new StringBuilder();
            if (!canonicalCharacterization().isEmpty()) {
                sb.append(canonicalCharacterization()).append("'");
            }
            sb.append("[\n");
            for (DetachmentStatement stmt : statements) {
                sb.append("  ").append(stmt).append(";\n");
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

    record DetachmentStatement(Identifier identifier, Expr expr) implements AST {
        public String toString() {
            return identifier + " = " + expr;
        }
    }

    record SearchUP(Identifier characterization) implements Characterizable {
        public SearchUP() {
            this(null);
        }

        public String toString() {
            return (((characterization != null && characterization.id.length() > 0) ? characterization.id + "'" : "")
                    + "↑");
        }

        @Override
        public boolean equals(Object obj) {
            return (obj != null &&
                    obj instanceof SearchUP other &&
                    this.canonicalCharacterization().equals(other.canonicalCharacterization())
            );
        }
    }

    record Branes(List<Characterizable> branes) implements Expr {
        public String toString() {
            StringBuilder sb = new StringBuilder();
            for (Characterizable brane : branes) {
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
            return identifier + " = " + expr;
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
}
