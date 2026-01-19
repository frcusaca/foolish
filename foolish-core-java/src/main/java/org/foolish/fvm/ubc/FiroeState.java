package org.foolish.fvm.ubc;

public sealed interface FiroeState permits FiroeState.Unknown, FiroeState.Value, FiroeState.Constantic {
    record Unknown() implements FiroeState {}
    record Value(FIR fir) implements FiroeState {}
    record Constantic() implements FiroeState {}
}
