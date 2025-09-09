package org.foolish.fvm;

import com.google.common.collect.ImmutableMap;
import java.util.concurrent.atomic.AtomicReference;

/**
 * An environment that holds identifier bindings.  Environments may be nested
 * so that lookups cascade to parent environments when an identifier is not
 * found locally.
 */
public class Environment {
    private final AtomicReference<ImmutableMap<Characterizable, Object>> values =
            new AtomicReference<>(ImmutableMap.of());
    private final Environment parent;

    public Environment() {
        this(null);
    }

    public Environment(Environment parent) {
        this.parent = parent;
    }

    public void define(Characterizable id, Object value) {
        Object v = value == null ? Unknown.INSTANCE : value;
        while (true) {
            ImmutableMap<Characterizable, Object> current = values.get();
            ImmutableMap<Characterizable, Object> updated = ImmutableMap.<Characterizable, Object>builder()
                    .putAll(current)
                    .put(id, v)
                    .build();
            if (values.compareAndSet(current, updated)) {
                return;
            }
        }
    }

    public boolean contains(Characterizable id) {
        if (values.get().containsKey(id)) return true;
        return parent != null && parent.contains(id);
    }

    public Object lookup(Characterizable id) {
        if (values.get().containsKey(id)) {
            Object v = values.get().get(id);
            return v == null ? Unknown.INSTANCE : v;
        }
        if (parent != null) {
            return parent.lookup(id);
        }
        return Unknown.INSTANCE;
    }
}
