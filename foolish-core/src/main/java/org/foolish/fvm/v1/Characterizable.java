package org.foolish.fvm.v1;

import org.foolish.ast.AST;

import java.util.Objects;

/**
 * Represents an identifier that may be characterized.  Characterizations are
 * themselves {@code Characterizable} allowing chains such as {@code a'b'c}.
 * <p>
 * The {@link Env} uses this class as the key for identifier
 * resolution so equality and hashing are based on the canonical
 * representation of the entire characterization chain.
 */
public record Characterizable(Characterizable characterization, String id) {
    public Characterizable(String id) {
        this(null, id);
    }

    public Characterizable(Characterizable characterization, String id) {
        this.characterization = characterization;
        this.id = id == null ? "" : id;
    }

    /**
     * Creates a {@code Characterizable} from an AST identifier chain.
     */
    public static Characterizable fromAst(AST.Identifier id) {
        if (id == null) return null;
        return new Characterizable(fromAst(id.characterization()), id.id());
    }

    /**
     * Canonical string representation used for equality and hashing.  The
     * representation flattens the characterization chain using apostrophes as
     * separators.
     */
    public String canonical() {
        if (characterization == null || characterization.id.isEmpty()) {
            return id;
        }
        String prefix = characterization.canonical();
        return prefix.isEmpty() ? id : prefix + "'" + id;
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null || getClass() != obj.getClass()) return false;
        Characterizable other = (Characterizable) obj;
        return Objects.equals(this.canonical(), other.canonical());
    }

    @Override
    public int hashCode() {
        return Objects.hash(canonical());
    }

    @Override
    public String toString() {
        return canonical();
    }
}

