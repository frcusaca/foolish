package org.foolish.fvm;

import org.foolish.fvm.ubc.CharacterizedIdentifier;

/**
 * The environment is responsible for finding coordinates. This means when the vm is evaluating a brane, it must know
 * what to do upon encounterance of a, possibly characterized, identifier. Since brane acts as SSA, the environment
 * lookup for the same CharacterizedIdentifier will be different from different lines. Therefore the retrieval method
 * `get` always requires a line number.
 * @param <ValueType>
 */
public interface  EnvInterface<ValueType> {
    public ValueType get(CharacterizedIdentifier id, int fromLine);
    public void put(CharacterizedIdentifier id, ValueType value, int byLine);
}
