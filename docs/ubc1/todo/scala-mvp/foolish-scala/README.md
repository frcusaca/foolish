# Foolish Scala MVP

A clean-slate Scala implementation of the Foolish language. Drop this directory
into a new git repo and run `mvn install -DskipTests` to verify the build.

## Quickstart

```bash
# Verify the build works (one-time sanity check; installs parser jar locally)
mvn install -DskipTests

# Run all unit tests (currently: FIR Circe roundtrip — 3 tests, should pass)
mvn test -pl foolish-core-scala

# Run a specific test class
mvn test -pl foolish-core-scala -Dtest=FirRoundtripTest

# Regenerate ANTLR parser after grammar changes
mvn clean generate-sources -pl foolish-parser-java
```

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
2. `docs/01_phases_overview.md` — the 5-phase roadmap with phase-by-phase scope table
3. `docs/02_implementor_reference.md` — Maven, grammar, AST node types
4. `docs/phase1_compiler.md` — the active phase: source → FIR JSON

Then begin **Phase 1**: implement `Compiler.compileAstToFir` step by step (P1.2
through P1.11). Each step adds one or two FIR variants and tests at three
independent layers:

- **Layer 1**: `.foo` source → Scala AST (parser correctness)
- **Layer 2**: Scala AST → FIR (translator correctness)
- **Layer 3**: FIR → JSON → FIR roundtrip (Circe codec correctness)

Phase 1 has no `.foo` approval tests — those arrive in Phase 2 (UBC step
evaluation). See `docs/phase2_ubc.md`.

## Project layout

```
foolish-scala/
├── README.md                    ← this file
├── pom.xml                      ← parent POM (defines all versions)
├── docs/
│   ├── 00_accumulated_specs.md
│   ├── 01_phases_overview.md
│   ├── 02_implementor_reference.md
│   ├── phase1_compiler.md       ← active phase
│   ├── phase2_ubc.md
│   ├── phase3_cli.md
│   ├── phase4_browser.md
│   ├── phase4.5_concatenation.md
│   └── phase5_detachment.md
├── foolish-parser-java/         ← ANTLR grammar + Java AST records (parser stays Java)
│   ├── pom.xml
│   └── src/main/
│       ├── antlr4/Foolish.g4
│       └── java/org/foolish/ast/
│           ├── AST.java
│           ├── ASTBuilder.java
│           ├── ASTFormatter.java
│           └── SearchOperator.java
└── foolish-core-scala/          ← Scala compiler + (later) evaluator
    ├── pom.xml
    └── src/
        ├── main/scala/org/foolish/fvm/scubc/
        │   ├── FoolishAst.scala       ← Scala sealed AST + fromJava() converter
        │   ├── Fir.scala              ← Foolish Internal Representation + Circe codecs
        │   └── Compiler.scala         ← Phase 1: source → AST → FIR → JSON
        └── test/
            ├── scala/org/foolish/fvm/scubc/
            │   └── FirRoundtripTest.scala  ← Phase 1, Layer 3 tests
            └── resources/org/foolish/fvm/
                └── inputs/            ← 60 active .foo + 5 .tbd test inputs (used in Phase 2)
```

## Key design rules

1. **Phase 1 does NOT evaluate.** The compiler produces a FIR tree where every
   node is `Initialized`, except integer literals (`Constant`) and `???` (`NK`).
   Phase 2 steps the states forward.

2. **The contract between Phase 1 and Phase 2 is the JSON FIR**, round-trip
   tested via Circe. There is no schema design — Circe's generic derivation on
   sealed case classes IS the contract.

3. **Bare identifiers compile to fully-configured search FIRs.**
   `a_config` → `SearchFir(pattern = "^a_config$", Backward, anchored = false, anchor = None)`.
   The pattern is a regex with `^...$` anchors so the search engine matches it
   directly. No "is this a name or a pattern" branching at evaluation time.

4. **Convert Java AST to Scala AST once at entry** (`FoolishAst.fromProgram`).
   All compilation works on the Scala types (`*Astn` suffix). Do not pattern-match
   Java types throughout the compiler.

5. **Three test layers in Phase 1.** A failing test points unambiguously at
   parser, translator, or codec. See `docs/phase1_compiler.md`.

6. **Scala 3 + Java sealed-record interop quirk**: `AST.Concatenation` cannot be
   referenced via the qualified `AST.Concatenation` path. It is imported by name
   in `FoolishAst.scala`. Other `AST.*` types work either way. Keep the import.

## Phase 1 implementation steps

See `docs/phase1_compiler.md` for the full list. Summary:

| Step | Adds |
|------|------|
| P1.1 | Skeleton (already done in scaffold) |
| P1.2 | `IntLitAstn → ConstantIntFir` |
| P1.3 | `BraneAstn → NormalBraneFir`, anonymous statements |
| P1.4 | `AssignmentAstn(Normal) → StatementFir` (named) |
| P1.5 | `BinaryExprAstn / UnaryExprAstn → BinaryOpFir / UnaryOpFir` (tree only, no compute) |
| P1.6 | `IdentifierAstn → SearchFir` (unanchored, `^id$` pattern) |
| P1.7 | `UnanchoredSeekAstn → IndexFir` (unanchored) |
| P1.8 | Anchored search operators: `.`, `?`, `^`, `$`, `#N` |
| P1.9 | Assignment sugar `=$` and `=^` |
| P1.10 | `???` literal → `NKFir` |
| P1.11 | Reject Phase 4.5 / 5 features with clear errors |

## Phase 1 → Phase 2 handoff

When Phase 1 exits:
- `Compiler.compileToJson(source)` works for every Phase 1 construct
- All three test layers are green
- Phase 2 work begins by reading `docs/phase2_ubc.md`, re-introducing the
  `ApprovalTestRunner` Java helper, and writing a `Ubc.step` function that
  walks the FIR tree

## License / origin

Originally from the `foolish` monorepo (see `docs/02_implementor_reference.md`
for the source layout). This is a clean-slate Scala port; no UBC1 Scala code
is inherited.
