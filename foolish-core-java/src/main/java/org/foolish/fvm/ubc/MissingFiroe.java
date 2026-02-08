package org.foolish.fvm.ubc;

import org.foolish.ast.AST;
import java.util.Optional;

/**
 * Represents a missing identifier or unresolved value.
 * Always in CONSTANIC state, returns Empty valuableSelf.
 */
public class MissingFiroe extends FIR implements Constanicable {

    public MissingFiroe() {
         super("Missing Identifier");
         // Ensure it's constanic
         setNyes(Nyes.CONSTANIC);
    }
    


    @Override
    public int step() {
        return 0;
    }

    @Override
    public Optional<FIR> valuableSelf() {
        return Optional.empty();
    }
    
    @Override
    public String toString() {
        return "MISSING";
    }

    @Override
    public long getValue() {
        throw new IllegalStateException("Cannot get value of Missing/Unresolved Identifier");
    }

    @Override
    public FIR getResult() {
        return null; // Not resolved to anything meaningful
    }

    @Override
    public boolean isConstanic() {
        return true;
    }
}
