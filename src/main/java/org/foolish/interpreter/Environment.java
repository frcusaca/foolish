package org.foolish.interpreter;

import java.util.HashMap;
import java.util.Map;

/**
 * Lexically scoped environment for variable bindings.
 */
public class Environment {
    private final Map<String, Value> values = new HashMap<>();
    private final Environment parent;

    public Environment() {
        this(null);
    }

    private Environment(Environment parent) {
        this.parent = parent;
    }

    /** Create a child environment. */
    public Environment createChild() {
        return new Environment(this);
    }

    /** Lookup a value in this environment chain. */
    public Value get(String name) {
        if (values.containsKey(name)) {
            return values.get(name);
        }
        if (parent != null) {
            return parent.get(name);
        }
        return Value.UNKNOWN;
    }

    /** Bind a value in this environment. */
    public void set(String name, Value value) {
        values.put(name, value);
    }

    /** Snapshot of current bindings for testing. */
    public Map<String, Value> bindings() {
        return Map.copyOf(values);
    }
}

