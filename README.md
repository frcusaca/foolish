
# Foolish language — Java + ANTLR4 + Ant + Ivy

Build & test:
  - `ant test`  : bootstraps Ivy, resolves deps (ANTLR 4.13.1, JUnit Console 1.10.2), generates parser, compiles, runs tests
  - `ant run`   : runs the demo on `samples/hello.foo`

Layout:
  - `src/main/antlr4/Foolish.g4` — grammar (Java target, visitor on, package `com.foolishlang.grammar`)
  - `src/main/java/com/foolishlang` — AST, AstBuilder, Symbols, SymbolBuilder, ParserFacade, Main
  - `src/test/java/com/foolishlang` — JUnit tests
  - `samples/hello.foo` — sample program

Notes:
  - Comments are lexed to the hidden channel and therefore ignored (denotational semantics preserved).
  - Concatenation is adjacency of brane-valued expressions (RPN), left-to-right; association is immaterial for denotation.
  - `^` and `$` are shorthand path derefs equivalent to `#1` and `#-1` when index omitted.
