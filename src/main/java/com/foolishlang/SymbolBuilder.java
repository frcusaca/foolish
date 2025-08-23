
package com.foolishlang;

import java.util.*;

public class SymbolBuilder {
    public record Result(Symbols.Scope globals, Map<AST.Node, Symbols.Scope> scopes, List<String> errors) {}

    public static Result build(AST.Program program) {
        Symbols.Scope globals = Symbols.createGlobal();
        Map<AST.Node, Symbols.Scope> scopes = new IdentityHashMap<>();
        List<String> errors = new ArrayList<>();
        scopes.put(program, globals);
        // Extend here: walk AST, define vars, params, types per brane, etc.
        return new Result(globals, scopes, errors);
    }
}
