package org.foolish.fvm;

import com.google.common.collect.Maps;

import java.util.Map;

/**
 * An immutable chain of environments supporting copy-on-write semantics.  Each
 * environment holds a local mapping of identifiers to {@link Resoe} values and
 * may reference a parent environment for lookups.  Updates create new local
 * bindings without mutating ancestors.
 */
public class Environment {
    private final Map<Characterizable, Resoe> values;
    private final Environment parent;

    /** Root environment with no parent. */
    public Environment() {
        this(null);
    }

    /** Child environment referencing the given parent. */
    public Environment(Environment parent) {
        this.parent = parent;
        this.values = Maps.newHashMap();
    }

    private Environment(Environment parent, Map<Characterizable, Resoe> values) {
        this.parent = parent;
        this.values = values;
    }

    /**
     * Create a child environment that refers to this environment for lookups.
     */
    public Environment child() {
        return new Environment(this);
    }

    /**
     * Define or update a value in this environment returning the modified
     * environment.  The original environment is not altered; instead a shallow
     * copy of the value map is produced.
     */
    public Environment define(Characterizable id, Resoe value) {
        Map<Characterizable, Resoe> copy = Maps.newHashMap(values);
        copy.put(id, value == null ? Resoe.UNKNOWN : value);
        return new Environment(parent, copy);
    }

    /**
     * Lookup a value by identifier searching this environment and, if
     * necessary, cascading to parent environments.
     */
    public Resoe lookup(Characterizable id) {
        Resoe val = values.get(id);
        if (val != null) {
            return val;
        }
        return parent != null ? parent.lookup(id) : Resoe.UNKNOWN;
    }
}

