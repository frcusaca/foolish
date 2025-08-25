
package foolishlang;

import java.util.*;

public class Symbols {
    public enum SymKind { VAR, TYPE, PARAM }

    public static final class Symbol {
        public final String name;
        public final SymKind kind;
        public final AST.Node node;
        public Symbol(String name, SymKind kind, AST.Node node) { this.name = name; this.kind = kind; this.node = node; }
    }

    public static class Scope {
        public final Scope parent;
        public final String name;
        private final Map<String, Symbol> symbols = new LinkedHashMap<>();
        public Scope(Scope parent, String name){ this.parent = parent; this.name = name; }

        public void define(Symbol s) { symbols.put(s.name, s); }
        public Symbol lookupLocal(String n){ return symbols.get(n); }
        public Symbol lookup(String n){
            for (Scope s=this; s!=null; s=s.parent){
                Symbol f = s.symbols.get(n);
                if (f!=null) return f;
            }
            return null;
        }
        public Collection<Symbol> entries(){ return symbols.values(); }
    }

    public static Scope createGlobal() {
        Scope g = new Scope(null, "global");
        for (String t : List.of("Int","Float","String","Brane")) {
            g.define(new Symbol(t, SymKind.TYPE, null));
        }
        return g;
    }
}
