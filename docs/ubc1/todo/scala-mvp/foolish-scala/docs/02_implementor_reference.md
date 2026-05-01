# Foolish Scala MVP — Implementor Reference

> Everything a new implementor needs to understand the existing build infrastructure,
> parser module, and Java AST before writing a single line of evaluation code.
>
> Read this before touching any source file.

---

## Repository Layout

```
foolish/                          ← root (Maven multi-module)
├── pom.xml                       ← parent POM — versions, plugin management
├── foolish-parser-java/          ← ANTLR grammar + Java AST records  ← READ THIS
├── foolish-core-java/            ← Java UBC1 evaluator  ← reference only, do not port
├── foolish-core-scala/           ← Scala UBC1 evaluator  ← clean-slate target module
├── foolish-lsp-java/             ← Language Server Protocol (ignore)
├── foolish-crossvalidation/      ← byte-identical Java/Scala checks  ← removed from scope
└── test-resources/               ← shared .foo approval test inputs
    └── org/foolish/fvm/inputs/   ← 60 active .foo + 7 disabled + 5 .tbd
```

**The implementation target is `foolish-core-scala/`.** It starts fresh — the existing
Scala UBC1 code there is reference material, not a base to build on. Treat it as readable
archaeology, not something to inherit.

---

## Build System

### Key versions (from root `pom.xml`)

| Thing | Version |
|-------|---------|
| Java | 25 |
| Scala | 3.8.2 |
| ANTLR4 | 4.13.2 |
| ScalaTest | 3.2.19 |
| JUnit Jupiter | 6.0.2 |
| ApprovalTests | 29.0.0 |
| scala-maven-plugin | 4.9.9 |
| maven-surefire-plugin | 3.2.5 |

All versions are managed in the root `pom.xml` `<dependencyManagement>` block — child
modules should NOT re-specify versions.

### Essential build commands

```bash
# Full clean build with all tests
mvn clean test

# Build only, skip tests (fast iteration on compilation errors)
mvn clean compile -DskipTests

# Regenerate parser from grammar (required after any .g4 change)
mvn clean generate-sources

# Run only Scala approval tests in foolish-core-scala
mvn test -pl foolish-core-scala -Dtest=ScUbcApprovalTest

# Run only a specific approval test, filtered by input file name
mvn test -pl foolish-core-scala -Dtest=ScUbcApprovalTest -Dfoolish.test.filter=Shadow

# Parallel build (use on multi-core machines)
mvn clean compile -T $(($(nproc) * 2))
```

### JVM flags required (Java 25 + unsafe memory access)

The root POM and both plugin configurations include these JVM args — the child module
must propagate them too. See the `foolish-core-scala/pom.xml` `<argLine>` and
`<jvmArgs>` sections:

```xml
--sun-misc-unsafe-memory-access=allow
--enable-native-access=ALL-UNNAMED
```

Without these, Scala compiler and Surefire both fail at runtime on Java 25.

### Test resource layout

The `foolish-core-scala/pom.xml` points test resources at `../test-resources` (the shared
directory). This means `foolish-core-scala` tests see the same `.foo` input files as
`foolish-core-java` without copying them. Do not change this — it is how the approval
test harness discovers inputs:

```xml
<testResources>
  <testResource>
    <directory>../test-resources</directory>
  </testResource>
</testResources>
```

### Approval test output directories

- Java approved files: `foolish-core-java/src/test/resources/org/foolish/fvm/ubc/`
- **Scala approved files: `foolish-core-scala/src/test/resources/org/foolish/fvm/scubc/`**

When the Scala harness runs a test and finds a mismatch, it writes a `.received.foo`
next to the `.approved.foo`. You must review the diff and manually rename (never
bulk-approve):

```bash
diff -y --color path/to/test.received.foo path/to/test.approved.foo
mv path/to/test.received.foo path/to/test.approved.foo
```

---

## Parser Module (`foolish-parser-java`)

### What it does

1. ANTLR4 generates a lexer and parser from `Foolish.g4`
2. `ASTBuilder` (a `FoolishBaseVisitor`) walks the parse tree and produces Java records
3. The resulting `AST.Program` is the input to all evaluators

The parser is **shared infrastructure** — do not modify it unless a grammar change is
explicitly planned. Grammar changes require `mvn clean generate-sources` and affect
both Java and Scala modules.

### Grammar file: `foolish-parser-java/src/main/antlr4/Foolish.g4`

The grammar is 328 lines. Key rules for MVP1:

```
program         → SHEBANG? branes EOF
branes          → brane+
brane           → standard_brane | detach_brane | brane_search
standard_brane  → '{' stmt* stmt_body? '}'
stmt            → stmt_body (';'|','|comment)+ | comment
stmt_body       → assignment | expr
assignment      → id '=' expr           (normal)
                | id '=' '$' expr        (tail sugar → OneShotSearchExpr TAIL)
                | id '=' '^' expr        (head sugar → OneShotSearchExpr HEAD)
                | id '<=>' expr          (SF, deferred MVP3)
                | id '<<=>>: expr        (SFF, deferred MVP3)
expr            → addExpr | ifExpr | concatenation
concatenation   → element (element)+     (elements on same line only)
postfixExpr     → primary (postfix_op)*
postfix_op      → '.' id                 → DereferenceExpr
               | ('?'|'~') regexp        → RegexpSearchExpr
               | '#' [-]N                → SeekExpr
               | '^'                     → OneShotSearchExpr HEAD
               | '$'                     → OneShotSearchExpr TAIL
primary         → characterizable
               | '(' expr ')'
               | '<<' expr '>>'          → StayFullyFoolishExpr (MVP3)
               | '<' expr '>'            → StayFoolishExpr (MVP3)
               | '???'                   → UnknownExpr
               | '#' '-' N              → UnanchoredSeekExpr
```

**Rules deferred to MVP3 (present in grammar, produce Java AST nodes to ignore):**
- `ifExpr` / `ifExprHelperIf` / `elif` / `else` / `fi` → `AST.IfExpr`
- `detach_brane` / `detach_stmt` → `AST.DetachmentBrane`, `AST.DetachmentStatement`
- `brane_search` (`↑`) → `AST.SearchUP`
- `'<<' expr '>>'` / `'<' expr '>'` in primary → `AST.StayFullyFoolishExpr`, `AST.StayFoolishExpr`
- SF/SFF assignment operators (`<=>`, `<<=>>`) → `AST.AssignmentOperator.CONSTANIC/SFF`

**Identifier canonicalization**: all three separator forms (`_`, `ˍ` modifier letter
low line, ` ` narrow no-break space) are normalized to `ˍ` by `ASTBuilder`:

```java
static final Pattern ID_CANONICALIZER = Pattern.compile("[ _ˍ]");
static final String INTRA_ID_SPACE = "ˍ";
```

When matching identifiers in the evaluator, always compare the canonicalized form.

### Generated parser location

After `mvn generate-sources`, the parser appears at:
```
foolish-parser-java/target/generated-sources/antlr4/org/foolish/grammar/
  FoolishLexer.java
  FoolishParser.java
  FoolishBaseVisitor.java
  FoolishVisitor.java
  FoolishListener.java
  FoolishBaseListener.java
```

These are never checked in — always regenerate with `mvn clean generate-sources`.

### How to invoke the parser from Scala

```scala
import org.antlr.v4.runtime.{CharStreams, CommonTokenStream}
import org.foolish.ast.{AST, ASTBuilder}
import org.foolish.grammar.{FoolishLexer, FoolishParser}

def parse(source: String): AST.Program = {
  val input  = CharStreams.fromString(source)
  val lexer  = new FoolishLexer(input)
  val tokens = new CommonTokenStream(lexer)
  val parser = new FoolishParser(tokens)
  val tree   = parser.program()
  new ASTBuilder().visitProgram(tree).asInstanceOf[AST.Program]
}
```

---

## Java AST: Node Types

All AST nodes are Java records in `org.foolish.ast.AST`. The full inheritance tree:

```
AST
├── Program(branes: Branes)
├── Expr
│   ├── Characterizable
│   │   ├── Literal
│   │   │   └── IntegerLiteral(characterizations, value: Long)
│   │   ├── Identifier(characterizations, id: String)
│   │   ├── Brane(characterizations, statements: List[Expr])
│   │   ├── DetachmentBrane(characterizations, statements)  ← MVP3
│   │   └── SearchUP(characterizations)                    ← MVP3
│   ├── BinaryExpr(op: String, left: Expr, right: Expr)
│   ├── UnaryExpr(op: String, expr: Expr)
│   ├── Branes(branes: List[Characterizable])              ← top-level wrapper
│   ├── Concatenation(elements: List[Expr])
│   ├── IfExpr(condition, thenExpr, elseExpr, elseIfs)     ← MVP3, reject at eval
│   ├── UnknownExpr()                                      ← ??? literal
│   ├── Stmt
│   │   └── Assignment(identifier, expr, operator, location)
│   ├── DereferenceExpr(anchor: Expr, coordinate: Identifier)  ← dot search
│   ├── RegexpSearchExpr(anchor: Expr, op: SearchOperator, pattern: String)
│   ├── SeekExpr(anchor: Expr, offset: Int)                ← anchored #N
│   ├── OneShotSearchExpr(anchor: Expr, op: SearchOperator) ← ^ $
│   ├── UnanchoredSeekExpr(offset: Int)                    ← #-N (offset is negative)
│   ├── StayFoolishExpr(expr: Expr)                        ← MVP3
│   └── StayFullyFoolishExpr(expr: Expr)                   ← MVP3
├── DetachmentStatement(identifier, expr)                  ← MVP3
└── BraneRegexpSearch(brane, operator, pattern)            ← unused in current tests
```

### SearchOperator enum

```java
HEAD("^")              — first element of a brane
TAIL("$")              — last element of a brane
REGEXP_LOCAL("?")      — backward search (from end toward start within brane)
REGEXP_FORWARD_LOCAL("~")  — forward search (from start toward end within brane)
REGEXP_GLOBAL("??")    — find-all backward (not implemented, ignore)
REGEXP_FORWARD_GLOBAL("~~") — find-all forward (not implemented, ignore)
SEEK("#")              — index access (used for anchored SeekExpr)
```

### Assignment.operator values

```java
ASSIGN("=")            — normal assignment, MVP1
CONSTANIC("<=>")       — SF assignment, MVP3
SFF("<<=>>")           — SFF assignment, MVP3
```

When `operator == CONSTANIC`, the `expr` field has already been wrapped in
`StayFoolishExpr` by `ASTBuilder`. When `operator == SFF`, wrapped in
`StayFullyFoolishExpr`. Normal assignment: `expr` is the raw RHS.

### AssignmentSugar

`=$ expr` and `=^ expr` are desugared by `ASTBuilder` into `OneShotSearchExpr`:
- `id =$ rhs` → `Assignment(id, OneShotSearchExpr(rhs, TAIL), ASSIGN)`
- `id =^ rhs` → `Assignment(id, OneShotSearchExpr(rhs, HEAD), ASSIGN)`

There is no separate sugar node — the sugar is eliminated at parse time.

### Identifier canonicalization (important)

`Identifier.id` is already canonicalized (separators normalized to `ˍ`).
`Identifier.characterizations` is a `List<String>` — empty list means no characterization.
Two identifiers are equal iff `id` and `canonicalCharacterization()` match.

### Matching Java sealed types from Scala

Scala 3 can pattern-match Java sealed interfaces via `@unchecked` if needed, but
the recommended approach is to convert to a Scala sealed hierarchy immediately on
entry to the evaluator:

```scala
def toScalaAst(expr: AST.Expr): FoolishExpr = expr match
  case lit: AST.IntegerLiteral  => IntLit(lit.value())
  case id: AST.Identifier       => Identifier(id.characterizations().asScala.toList, id.id())
  case br: AST.Brane             => Brane(br.characterizations().asScala.toList,
                                          br.statements().asScala.toList.map(toScalaAst))
  case bin: AST.BinaryExpr       => BinaryExpr(bin.op(), toScalaAst(bin.left()), toScalaAst(bin.right()))
  case un: AST.UnaryExpr         => UnaryExpr(un.op(), toScalaAst(un.expr()))
  case cat: AST.Concatenation    => Concatenation(cat.elements().asScala.toList.map(toScalaAst))
  case deref: AST.DereferenceExpr => DotSearch(toScalaAst(deref.anchor()), toScalaAst(deref.coordinate()).asInstanceOf[Identifier])
  case re: AST.RegexpSearchExpr  => RegexSearch(toScalaAst(re.anchor()), re.operator(), re.pattern())
  case seek: AST.SeekExpr        => IndexAccess(toScalaAst(seek.anchor()), seek.offset())
  case oseek: AST.UnanchoredSeekExpr => UnanchoredSeek(oseek.offset())   // offset is already negative
  case os: AST.OneShotSearchExpr  => OneShotSearch(toScalaAst(os.anchor()), os.operator())
  case _: AST.UnknownExpr        => NKExpr                                // ??? literal
  case asgn: AST.Assignment      => Assignment(toScalaAst(asgn.identifier()).asInstanceOf[Identifier],
                                               asgn.operator(), toScalaAst(asgn.expr()))
  case branes: AST.Branes        => TopLevel(branes.branes().asScala.toList.map(toScalaAst))
  // Deferred to MVP3 — reject at conversion time:
  case _: AST.IfExpr             => NotImplemented("if-then-else removed from UBC2")
  case _: AST.DetachmentBrane    => NotImplemented("detachment deferred to MVP3")
  case _: AST.SearchUP           => NotImplemented("↑ search deferred to MVP3")
  case _: AST.StayFoolishExpr    => NotImplemented("SF marker deferred to MVP3")
  case _: AST.StayFullyFoolishExpr => NotImplemented("SFF marker deferred to MVP3")
```

Do the conversion once, at the top of `BraneComputer.run`, before any evaluation.
The evaluator then works entirely in Scala sealed types.

---

## `foolish-core-scala` Module: What Exists

The existing files in `foolish-core-scala/src/main/scala/org/foolish/fvm/scubc/` are a
partial UBC1 port. They are readable for understanding Foolish semantics but should not
be inherited by the new implementation. Key inventory:

| File | What it does | New impl: use or replace |
|------|-------------|--------------------------|
| `FIR.scala` | Abstract base trait for FIR objects | Replace with your own sealed trait |
| `FiroeState.scala` | 3-value state: `Unknown`, `Value(fir)`, `Constanic` | Replace — new impl uses `Constant`, `Constanic`, `NK` |
| `BraneMemory.scala` | Append-only statement store with cursor | Reference for lookup semantics; replace with simpler List |
| `Sequencer4Human.scala` | Output formatter | **Reuse or port** — its format IS the approval test baseline |
| `UbcRepl.scala` | Simple REPL (evaluates whole files) | Replace with MVP3 REPL |
| `UnicelluarBraneComputer.scala` | Top-level evaluator entry point | Replace with your `BraneComputer` |
| `AbstractSearchFiroe.scala` | UBC1 search machinery | Read for semantics; do not port |
| `ConcatenationFiroe.scala` | Three-stage concat | Do not port — new model is sequential blocking |
| `FiroeWithBraneMind.scala` | Message-passing work queue | Do not port — premature optimization |
| `ExecutionFir.scala` | Message dispatcher | Do not port |
| `IfFiroe.scala` | if-then-else evaluator | Do not port — removed from UBC2 |

The existing `ScUbcApprovalTest.scala` in the test directory is the approval test harness
to keep and adapt for the new evaluator.

---

## Approval Test Protocol

**Never edit `.approved.foo` files directly.** The protocol is:

1. Source code or `.foo` input changes
2. Run the test → produces `.received.foo`
3. Review: `diff -y --color test.received.foo test.approved.foo`
4. Human approves → `mv test.received.foo test.approved.foo`
5. Commit message must mention "approval test updated"

The `.tbd` files (5 of them, all in the `inputs/` directory) have input but no approved
output yet. Treat them as new tests: run, review, approve when correct.

---

## Test Input Directory

```
test-resources/org/foolish/fvm/inputs/
```

Active tests (60 `.foo` files), disabled tests (7 `.foo.disabled`), and pending tests
(5 `.foo.tbd`). The approval test harness discovers files by extension — `.disabled` and
`.tbd` files are skipped unless the harness is specifically configured to include them.

---

## Identifier Separator Note

The Foolish grammar accepts three visual forms of the intra-identifier word separator:

| Form | Unicode | Visual |
|------|---------|--------|
| Underscore | U+005F `_` | `user_name` |
| Modifier letter low line | U+02CD `ˍ` | `userˍname` |
| Narrow no-break space | U+202F ` ` | `user name` |

All three are normalized to `ˍ` by `ASTBuilder.canonicalizeIdentifierName()` before
the identifier enters the AST. The evaluator always sees the canonical form.

---

## Last Updated

**Date**: 2026-04-30
**Updated By**: Claude Code 2.1.119 (Claude Code); claude-sonnet-4-6
**Changes**: Initial creation — comprehensive implementor reference covering Maven build,
grammar, Java AST node types, SearchOperator enum, Scala interop patterns, existing
Scala module inventory, and approval test protocol.
