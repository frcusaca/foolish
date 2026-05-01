# Foolish Scala MVP — Accumulated Specifications

> Reading notes assembled from the docs branch and main branch codebase.
> This document is the foundation for the implementation plan.
> Updated as new reading sessions accumulate more detail.

---

## Purpose

This document records what we know about the Foolish language system, drawn from:
- `docs/` branch: tutorials, engineering docs, design specs, philosophy
- `main` branch: existing Java and Scala implementations, approval tests

The goal is a single Scala implementation targeting:
- **MVP1**: A REPL (read-eval-print loop)
- **MVP2**: A web application for browsing and expanding branes

---

## What Foolish Is

Foolish is a programming language built around a single primitive: the **brane** (`{...}`).
Three axioms govern everything:

1. **Containment is information** — branes organize scope
2. **Proximity enables combination** — adjacent branes concatenate
3. **Expression powers all** — every value is a brane or evaluates to one
4. **Brane is coordinating** — expression values float around until they become part of a brane, at which point they are coordinated with other expressions on the brane.

Design philosophy: inspired by the simplicity of DNA (4 base pairs → all life), Foolish tries to
derive a rich language from a minimal set of primitives.

---

## Language Feature Inventory

Features are categorized by implementation status in the test suite.

### Tier 1: Implemented and Tested (60 approval tests, active)

| Feature | Key Behavior | Approval Tests | Comments |
|---------|-------------|----------------|
| **Literal values** | int, float, string, empty brane `{}` | `simpleIntegerIsApproved` | Expand to data types not covered |
| **Branes** | `{...}` container; statements separated by `;` or `,` or newline | `emptyBraneIsApproved`, `nestedBranesIsApproved` | |
| **Identification** | `name = expr` — ordination to a brane | `simpleIdentifierIsApproved` | |
| **Arithmetic** | `+`, `-`, `*`, `/`, `%`; unary `-` | `simpleAdditionIsApproved`, `zeroDivisionIsApproved` | |
| **NK (Not Known)** | `???` — definite unknown; div-by-zero produces it | `zeroDivisionIsApproved` | |
| **Constanic** | `🧠??` — "not known yet"; unresolved search produces it | `constanticRendering` | |
| **Comments** | `!! line comment`, `!!! block !!!` | `commentEndsStatement` | |
| **Multi-script identifiers** | Unicode letters + `_` / `ˍ` / narrow-no-break-space as word separator | `identifierSeparators` | |
| **Characterization** | `type'name` — disambiguating qualifier on identifiers | `complexIdentifierScopeIsApproved` | |
| **Shadowing** | Later definitions shadow earlier ones; backward search finds most recent | `identifierShadowingIsApproved` | |
| **Scope / retrospection** | Backward-then-up search: current brane → parent → grandparent | `nestedScopeIdentifierIsApproved` | |
| **Dot access (anchored backward search)** | `brane.name` ≡ `brane?name` — backward search within brane | `searchLocalizedVsGlobalizedIsApproved` | |
| **`?` localized search** | Backward anchored pattern/name search | `searchPatternBasicsIsApproved` | |
| **`~` forward search** | Forward anchored pattern/name search (first match) | `testTilde` | |
| **Regex patterns** | `brane?pattern`, `brane~pattern` | `regexSearchWithPatternIsApproved`, `searchRegexPatternsIsApproved` | |
| **Head `^` / tail `$`** | First / last element of brane | `oneShotSearchIsApproved` | |
| **Index access `#N`** | Nth statement (0-based forward, negative from end) | `offsetAccess` | |
| **Unanchored seek `#-N`** | N statements back from current position | `unanchoredSeekBasic` | |
| **Unanchored identifier search** | `name` without anchor — backward + parent chain | `levelSkippingSearchFound`, `levelSkippingSearchConstanic` | |
| **Anchored search on constant** | Returns `???` if not found (brane is fully known) | `anchoredSearchFailsOnConstant` | |
| **Anchored search on constanic** | Returns `🧠??` — search result may still resolve | `anchoredSearchOnConstanic` | |
| **Concatenation** | `{a}{b}` — three-stage: isolate, merge, re-evaluate | `concatenationBasics`, `concatenationResolution`, `concatenationResolutionAdv` | |
| **Assignment anchor** | `=$ expr`, `=^ expr`, `=#N expr` — syntactic sugar | `assignmentAnchor` | |
| **Shebang** | `#!/...` line ignored at top of file | `shebang` | |
| **Operator precedence** | `*`, `/`, `%` before `+`, `-`; parens override | `operatorPrecedenceIsApproved` | |

### Tier 2: Specified but Disabled (detachment tests — `.disabled` suffix)

| Feature | Status |
|---------|--------|
| **Detachment `[id]{...}`** | Specified, tests disabled |
| **P-brane `[+id]{...}`** | Specified, tests disabled |
| **Detachment with SF/SFF marks** | Specified, tests disabled |
| **Detachment forward search** | Specified, tests disabled |

### Tier 3: Documented but Not Yet Implemented

| Feature | Notes |
|---------|-------|
| **Conditional** | `if ... then ... else ...` — IfFiroe exists in UBC1 but UBC2 design removes it in favor of search-based selection |
| **Loops / recursion** | In legacy TODO; not yet designed |
| **Liberation branes** | Referenced in tutorial Chapter 3 placeholder |
| **Characterization of branes** | `char'{}` typing — partially designed |
| **`↑` upward search** | `SearchUpFiroe` exists in Scala; semantics in ADVANCED_FEATURES |
| **Constanic brackets `<...>`** | Capture semantics designed, not implemented |
| **Stay Foolish `<<...>>`** | AST capture, CMFir in Java impl |
| **Program differentiation** | Future |
| **Mutable branes** | Future |

---

## VM Architecture: UBC2 (Target Design)

The UBC2 is the reference implementation we are building toward. Key departure from UBC1:

### Lifecycle States (PREMBRYONIC → EMBRYONIC → BRANING → constanic)

| State | Meaning |
|-------|---------|
| `PREMBRYONIC` | Holds AST; atomic setup |
| `EMBRYONIC` | Resolving searches via message passing |
| `BRANING` | Stepping children; forwarding messages |
| `CONSTANIC` | Terminal: paused, may resolve in new context |
| `WOCONSTANIC` | Constanic, with constanic dependencies |
| `CONSTANT` | Fully evaluated, immutable |
| `INDEPENDENT` | Literals — always in this state |

### Four FIR Roles (all derive from ProtoBrane)

| Role | Syntax | Boundary | `value()` |
|------|--------|----------|-----------|
| **Normal Brane** | `{...}` | Yes — local namespace | Returns self (first-class) |
| **System Operator** | `🧠+`, `🧠-`, etc. | No — transparent | Returns scalar |
| **ConcatenationBrane** | `A B` | Temporary isolation | Returns merged brane |
| **DetachmentBrane** | `[id]{...}` | Filter (active in EMBRYONIC) | Returns wrapped brane |

### Message Protocol

Two message types flow between branes:

- `FulfillSearch` (child → parent): "I need identifier X"
- `RespondToSearch` (parent → child): "Here is X / not found"

MVP communication medium: **parent-to-ancestor chaining** (simple hop-by-hop). Each brane knows
only its immediate parent. Advanced optimization (delegated direct addressing) comes later.

### Key Design Decisions from UBC2 Spec

1. **No `if-then-else`** — IfFiroe removed; path selection is search-based (search returns first
   match, which acts as conditional selection)
2. **Writing-order precedence** — First-to-write wins when multiple matches exist
3. **Bandwidth limits** — Max 4 `FulfillSearch` dispatches per step; max N inbound messages
4. **Depth limit** — 96,485 nested branes max; beyond that → NK + MILD alarm
5. **Constanic vs NK distinction** — Critical: anchored-search-not-found → NK; unanchored → CONSTANIC
6. **Operators are syntactic sugar** — `1 + 2` desugars to `{🧠1, 🧠2, 🧠+}`

---

## Existing Codebase State (main branch)

### Module Structure

```
foolish-parser-java/    — ANTLR4 grammar (Foolish.g4) + Java AST records
foolish-core-java/      — Java UBC1 implementation (~60 source files)
foolish-core-scala/     — Scala UBC1 implementation (~25 source files, partial parity)
foolish-crossvalidation/ — Tests that Java and Scala produce identical output
foolish-lsp-java/       — Language Server Protocol implementation (Java)
```

### Parser (shared, not to be changed)

- ANTLR4 grammar at `foolish-parser-java/src/main/antlr4/Foolish.g4`
- Java AST records (shared by both Java and Scala impls via Java interop)
- Grammar changes require `mvn clean generate-sources`

### Scala UBC1 (existing, ~3775 lines total)

Key files and their roles:
- `FIR.scala` (261 lines) — abstract base
- `FiroeWithBraneMind.scala` (373 lines) — work queue management
- `BraneFiroe.scala` (136 lines) — brane evaluation
- `ConcatenationFiroe.scala` (256 lines) — concatenation
- `AbstractSearchFiroe.scala` (437 lines) — search base
- `BraneMemory.scala` (203 lines) — append-only statement store
- `FiroeState.scala` — 3-value sealed trait (Unknown, Value, Constanic); simpler than Java's Nyes
- `Sequencer4Human.scala` (354 lines) — output formatting
- `UbcRepl.scala` (72 lines) — working REPL (simple, line-by-line)
- `UnicelluarBraneComputer.scala` (74 lines) — top-level evaluator

### Java UBC1 (reference, fuller implementation)

Has features Scala doesn't yet have:
- `DetachmentBraneFiroe.java` — detachment (Scala doesn't have this)
- `CMFir.java` — "Stay Foolish" context manipulation
- `IfFiroe.java` — conditional (to be deprecated in UBC2)
- `SFMarkFiroe.java` — SF/SFF markers
- Cursor system (`FoolishCursor`, `SearchCursor`, `ExpressionSearchCursor`)
- `FoolishIndex` / `FoolishIndexBuilder` — search cache

### Approval Tests

- 60 active `.foo` test files in `test-resources/org/foolish/fvm/inputs/`
- Several `.disabled` files (detachment, not yet passing)
- Tests run via `UbcApprovalTest.java` (Java) and `ScUbcApprovalTest.scala` (Scala)
- Cross-validation in `foolish-crossvalidation` enforces byte-identical output

---

## Why Previous Implementations Got Stuck

From docs history and branch archaeology:

1. **UBC0/UBC1 Java**: Full implementation but accumulated complexity in the state machine.
   The intermediate states (REFERENCES_IDENTIFIED, ALLOCATED, RESOLVED) were collapsed to CHECKED
   but the code grew organically and became hard to reason about.

2. **UBC1 Scala**: Partial port — got most of the core working but detachment and CMFir
   not ported.

3. **Rust attempt**: Started but abandoned (mentioned in docs, no significant code visible in
   current branches).

4. **The stuck point**: VM microstates were not thought through. Specifically, the distinction
   between CONSTANIC (waiting for context) and CONSTANT (done), the three-stage concatenation
   protocol, and the message-passing architecture were murky. The UBC2 design docs (written in
   early 2026) are the breakthrough — they clarify these precisely.

---

## Approval Test Coverage by Feature

| Feature | # Tests | Notes |
|---------|---------|-------|
| Arithmetic | ~10 | All passing |
| Identifier/scope | ~12 | All passing |
| Search (anchored, unanchored, regex) | ~15 | All passing |
| Head/tail/index | ~5 | All passing |
| Concatenation | 4 | All passing |
| Nesting (deeply nested branes) | ~5 | All passing |
| Constanic rendering | 1 | Passing |
| Detachment | 6 | All disabled |

---

## Notes for MVP1 (REPL)

The existing `UbcRepl.scala` is a starting point but evaluates whole files, not a session.
A proper REPL needs:

1. Persistent session state — each line extends a running brane
2. Error recovery — parse errors should not kill the session
3. Formatted output — using `Sequencer4Human` (already exists)
4. Line editing — ideally with readline/jline

The existing `UbcRepl.scala` already parses and evaluates single expressions against the UBC.
The gap is session continuity and formatted output display.

---

## Notes for MVP2 (Webapp)

The webapp browses and expands branes. Natural tech choices for Scala:

- **HTTP**: http4s or Play Framework
- **Frontend**: Scala.js or a thin JS layer calling a JSON API
- **Brane rendering**: The `Sequencer4Human` format maps naturally to a tree UI

Key interaction: click on a brane → expand it → see its ordinates → navigate to sub-branes.

---

## Open Questions / Decisions Needed

1. **UBC1 vs UBC2 target**: Build UBC2 from scratch (clean, correct) or extend existing Scala UBC1
   (faster start, debt inherited)?
2. **Detachment for MVP1**: Include in scope or defer to post-MVP1?
3. **Session model for REPL**: Accumulate into one growing top-level brane, or each line is
   independent?
4. **Output format**: Keep `Sequencer4Human` format, or design a new canonical output?
5. **Build target**: Standalone JAR with embedded REPL, or module-per-concern?

---

## Last Updated

**Date**: 2026-04-30
**Updated By**: Claude Code claude-sonnet-4-6
**Changes**: Initial creation — accumulated specifications from full read of docs branch and main
branch codebase in preparation for Scala MVP implementation planning.
