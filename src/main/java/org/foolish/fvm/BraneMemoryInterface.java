package org.foolish.fvm;

import org.foolish.fvm.ubc.CharacterizedIdentifier;

/**
 * The environment is responsible for finding coordinates by name. When evaluating a brane, the VM must know
 * what to do upon encountering of an, possibly characterized, identifier. Since brane acts as SSA, the environment
 * lookup for the same CharacterizedIdentifier will be different from different lines. Therefore, the retrieval method
 * `get` always requires a line number.
 *
 * @param <KeyType>
 * @param <ValueType>
 */
public interface BraneMemoryInterface<KeyType, ValueType> {

    /**
     * Retrieves the value associated with the given identifier inseted at the specified line.
     *
     * @param id The characterized identifier to look up.
     * @param fromLine The line number from which to perform the lookup.
     * @return The value associated with the identifier at the specified line.
     */
    public ValueType get(KeyType id, int fromLine);

    /**
     * Stores the value associated with the given identifier queried at the specified line.
     *
     * @param id The characterized identifier to store.
     * @param value The value to associate with the identifier.
     * @param byLine The line number at which to store the value.
     */
    public void put(KeyType id, ValueType value, int byLine);
}
