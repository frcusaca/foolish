package org.foolish.fvm;

import java.util.HashMap;
import java.util.Map;

/**
 * An environment that holds identifier bindings.  Environments may be nested
 * so that lookups cascade to parent environments when an identifier is not
 * found locally.
 */
public class Environment {
    private final Map<Characterizable, Finear> values = new HashMap<>();
    private final Environment parent;

    public Environment() {
        this(null);
    }

    public Environment(Environment parent) {
        this.parent = parent;
    }

    public void define(Characterizable id, Finear value) {
        values.put(id, value == null ? Finear.UNKNOWN : value);
    }

    public boolean contains(Characterizable id) {
        if (values.containsKey(id)) return true;
        return parent != null && parent.contains(id);
    }

    public Finear lookup(Characterizable id) {
        if (values.containsKey(id)) {
            Finear v = values.get(id);
            return v == null ? Finear.UNKNOWN : v;
        }
        if (parent != null) {
            return parent.lookup(id);
        }
        return Finear.UNKNOWN;
    }
}
