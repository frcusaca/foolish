package org.foolish.fvm;

/**
 * The environment is responsible for findling coordinates. This means when the vm is evaluating a brane, it must know
 * what to do upon encounterance of a, possibly characterized, identifier. Since brane acts as SSA, the environment
 * lookup for the same CharacterizedIdentifier will be different from different lines. Therefor the retrieval method
 * `get` always requires a line number.
 * @param <ValueType>
 */
public interface  EnvInterface<ValueType> {
    public ValueType get(CharacterizableIdentifier id, int fromLine);
    public void put(CharacterizableIdentifier id, ValueType value, int byLine);
}
