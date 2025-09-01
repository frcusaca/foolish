package org.foolish.reference.interpreter;

import org.foolish.ast.AST;
import org.foolish.reference.interpreter.ir.RuntimeNode;

import java.util.Collections;
import java.util.HashMap;
import java.util.Map;

/** Simple mutable environment mapping Identifier -> RuntimeNode. */
public class Environment {
    private final Map<AST.Identifier, RuntimeNode> bindings = new HashMap<>();

    public RuntimeNode get(AST.Identifier id) {
        return bindings.get(id);
    }

    public void put(AST.Identifier id, RuntimeNode value) {
        bindings.put(id, value);
    }

    public Map<AST.Identifier, RuntimeNode> snapshot() {
        return Collections.unmodifiableMap(new HashMap<>(bindings));
    }

    public Environment copy() { // new child environment seeded with current bindings
        Environment e = new Environment();
        e.bindings.putAll(this.bindings);
        return e;
    }
}

