# Foolish Scala MVP

A clean-slate Scala implementation of the Foolish language. Drop this directory
into a new git repo and run `mvn compile` to verify the build, then begin Phase 1a.

## Quickstart

```bash
# Verify the build works (one-time sanity check)
mvn install -DskipTests

# Run a single approval test by name fragment
mvn test -pl foolish-core-scala -Dtest=ApprovalTest -Dfoolish.test.filter=emptyBrane

# Run all approval tests (initially: 60 failures expected — no .approved.foo files yet)
mvn test -pl foolish-core-scala -Dtest=ApprovalTest

# Regenerate ANTLR parser after grammar changes
mvn clean generate-sources -pl foolish-parser-java
```

The first `mvn install` is required to publish `foolish-parser-java` to the local
Maven repo so `foolish-core-scala` can resolve it. After that, plain `mvn test`
in the project root works.

## Required prerequisites

- Java 25 (managed via SDKMAN, jenv, or system install)
- Maven 3.8+
- Internet access on first build (Maven downloads dependencies)

The build uses two JVM flags that must be set on Java 25:
```
--sun-misc-unsafe-memory-access=allow
--enable-native-access=ALL-UNNAMED
```
These are already wired into the POM — no manual export needed.

## Where to start coding

Read in this order:

1. `docs/00_accumulated_specs.md` — language semantics, what Foolish is
2. `docs/01_implementation_schedule.md` — phased plan (P1a → P11)
3. `docs/02_implementor_reference.md` — Maven, grammar, AST node types

Then begin **Phase 1a (Grammar + Scala AST)**: strip `ifExpr`, detachment, and SF/SFF
markers from `foolish-parser-java/src/main/antlr4/Foolish.g4`, then write parser unit
tests. Phase 1a is pure Scala unit tests — no `.foo` approval tests yet.

`docs/01_implementation_schedule.md` lists every test to enable per phase. Approve
`.received.foo` files one by one; never bulk-approve.

## Project layout

```
foolish-scala/
├── README.md                    ← this file
├── pom.xml                      ← parent POM (defines all versions)
├── docs/                        ← specs, schedule, reference
├── foolish-parser-java/         ← ANTLR grammar + Java AST records (parser stays Java)
│   ├── pom.xml
│   └── src/main/
│       ├── antlr4/Foolish.g4    ← grammar (strip MVP3 features in P1a)
│       └── java/org/foolish/ast/
│           ├── AST.java
│           ├── ASTBuilder.java
│           ├── ASTFormatter.java
│           └── SearchOperator.java
└── foolish-core-scala/          ← Scala evaluator (you write this)
    ├── pom.xml
    └── src/
        ├── main/scala/org/foolish/fvm/scubc/
        │   ├── BraneComputer.scala     ← evaluator entry point (currently a stub)
        │   └── FoolishAst.scala        ← Scala sealed AST + fromJava() converter
        └── test/
            ├── java/org/foolish/
            │   ├── UbcTester.java          ← interface
            │   └── ApprovalTestRunner.java ← shared harness
            ├── scala/org/foolish/fvm/scubc/
            │   ├── FoolishInterpreter.scala ← bridges BraneComputer into harness
            │   └── ApprovalTest.scala      ← discovers .foo, runs, compares
            └── resources/org/foolish/fvm/
                ├── inputs/                 ← 60 active .foo + 5 .tbd test inputs
                └── scubc/                  ← .approved.foo files (you create these)
```

## Key design rules

1. **Do not modify the parser module Java code** until P1a explicitly requires it.
   The grammar strip (removing `ifExpr` etc.) happens in P1a; the Java AST records
   stay as they are.

2. **Convert Java AST to Scala AST once at entry** (`FoolishAst.fromJava`). All
   evaluation works on the Scala types. Do not pattern-match Java types throughout
   the evaluator.

3. **Scala 3 + Java sealed-record interop quirk**: `AST.Concatenation` cannot be
   referenced via the qualified `AST.Concatenation` path. It is imported by name
   in `FoolishAst.scala`. Other `AST.*` types work either way. Keep the import.

4. **Never bulk-approve `.received.foo` files**. Diff each one, understand the
   change, then `mv` it to `.approved.foo`. The approval test protocol is in
   `docs/01_implementation_schedule.md` Phase 8.

5. **Never edit `.approved.foo` files directly.** Change the input or the
   evaluator code, run the test, review the diff, then approve.

## Per-phase workflow

For each phase Pn in the schedule:

1. Read the phase section in `docs/01_implementation_schedule.md`
2. Write any new test inputs listed for that phase first
3. Implement the code in `foolish-core-scala/src/main/scala/`
4. Run filtered tests: `mvn test -pl foolish-core-scala -Dtest=ApprovalTest -Dfoolish.test.filter=<keyword>`
5. Diff and approve `.received.foo` outputs one at a time
6. Commit with a message naming the phase (e.g., "P3: arithmetic with NK propagation")

When a `.foo` input file changes or new `.foo` inputs are added, document that in
the commit message — the approval protocol cares about the lineage.

## Notes on the parser module

The parser module is mostly stable. Two things may change during the project:

- **P1a grammar strip**: remove `ifExpr`, `detach_brane`, `brane_search` (`↑`),
  SF/SFF markers, and corresponding tokens. This requires:
  ```bash
  mvn clean generate-sources -pl foolish-parser-java
  mvn install -pl foolish-parser-java -DskipTests
  ```
  After stripping, `ASTBuilder.java` will need its visitor methods for those rules
  removed too. The Java AST records can stay (unused records do no harm) or be
  removed for cleanliness.

- **MVP3 reintroduction**: when adding detachment, SF/SFF, or "value search" in
  MVP3, the corresponding grammar rules must come back. Keep a tag or branch at
  the end of MVP1 so the original full grammar is recoverable.

## License / origin

Originally from the `foolish` monorepo (see `docs/02_implementor_reference.md` for
the source layout). This is a clean-slate Scala port; no UBC1 Scala code is inherited.
