package org.foolish.fvm.ubc;

import java.util.function.Function;

/**
 * Either type utility.
 */
public sealed interface Either<L, R> permits Either.One, Either.TheOther {
    record One<L, R>(L value) implements Either<L, R> {}
    record TheOther<L, R>(R value) implements Either<L, R> {}

    /**
     * Applies a function if this is a One value.
     */
    default <T> Either<T, R> applyOne(Function<? super L, ? extends T> mapper) {
        if (this instanceof One<L, R> l) {
            return new One<>(mapper.apply(l.value()));
        } else if (this instanceof TheOther<L, R> r) {
            return new TheOther<>(r.value());
        }
        throw new IllegalStateException("Unknown Either subtype: " + this.getClass());
    }

    /**
     * Applies a function if this is a TheOther value.
     */
    default <T> Either<L, T> applyTheOther(Function<? super R, ? extends T> mapper) {
        if (this instanceof One<L, R> l) {
            return new One<>(l.value());
        } else if (this instanceof TheOther<L, R> r) {
            return new TheOther<>(mapper.apply(r.value()));
        }
        throw new IllegalStateException("Unknown Either subtype: " + this.getClass());
    }

    /**
     * Transforms both sides simultaneously or reduces to a single value.
     */
    default <T> T fold(Function<? super L, ? extends T> oneMapper,
                       Function<? super R, ? extends T> otherMapper) {
        if (this instanceof One<L, R> l) {
            return oneMapper.apply(l.value());
        } else if (this instanceof TheOther<L, R> r) {
            return otherMapper.apply(r.value());
        }
        throw new IllegalStateException("Unknown Either subtype: " + this.getClass());
    }
}
