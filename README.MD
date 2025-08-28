# Foolish language — Java + ANTLR4 + Maven

Build & test:
  - `mvn clean generate-sources compile test`  : 

Layout:
  - `src/main/antlr4/Foolish.g4` — grammar (Java target, visitor on, package `com.foolishlang.grammar`)
  - `src/main/java/org/foolish` — AST, AstBuilder, Symbols, SymbolBuilder, ParserFacade, Main
  - `src/test/java/org/foolish` — JUnit tests
  - `samples/hello.foo` — sample program

Notes:
  - Comments are lexed to the hidden channel and therefore ignored (denotational semantics preserved).
  - Concatenation is adjacency of brane-valued expressions (RPN), left-to-right; association is immaterial for denotation.
  - `^` and `$` are shorthand path derefs equivalent to `#1` and `#-1` when index omitted.

The Foolish language is a computer programming language environment. The language is specified to grow as we find the market fit for it in this age of rapidly expanding AI usage.

Foolish language begins as an exercise in vibe programming. We use AI to generate much of the code for a simplistic functional programming language. This language has simple concept of a "brane" that largely resembles code blocks, lambda's, struct, class, enum, records and other similar constructs in modern day programming languages. To that end, it is curly bracket enclosed block of statements. Statements in Foolish are simply assignments of an expression to a name. Statements can also just be an express, the name is anonymized in this case. Brane, once understood, can be inspected for it's values at a certain key, or a certain line/statement number. It is permitted to assign to the same name repeatedly. The language facilitates retrieval of all of its values convenienced by syntactical sugaring.

Branes having unbound symbols are permitted. They are essentially brane functions. Function invocation, along with many other programing concepts such as concatenation, derivation, extending classes and traits, are all accomplished by brane concatenation. So to call a function `F` using parameters listed in brane `P`, the reverse polish expression creates the exact condition for invocation `P F` concatenates `P` infront of `F` setting values to names that `F` will use. Derivation simply concatenate updates to the end of the brane.

Note, this foolishness is unrelated with other previous efforts that may have used the name Foolish for name of a language. However, I'm sure we're similarly inspired by the same foolhearty impulses to this level of lunacy.

There's some kind of very permissive sofware license on the code and ideas expressed here. Please use foolishly, but do give back to the originators and contributors of this project so as to sustain either their work and lives, or otherwise keep alive the spirit to develope despite our fatuity.
