# Foolish Scala MVP — Implementation Schedule

> Derived from: full docs-branch read, 60 active approval tests, 7 disabled tests,
> and implementation history archaeology (antigravity incident, concatenation woes, etc.)
>
> Key design decisions recorded here (from human who was there):
> - **Root cause of prior failures**: "early optimization" — tried reference-tracking of
>   incomplete computation. Switched to message-passing prematurely.
> - **New approach**: expressions are lazy in Phase 1. If any dependency is missing,
>   the whole expression is CONSTANIC immediately (holds AST, no partial eval).
>   Branes are eager — descend into them, evaluate each statement, hold mixed
>   CONSTANT/CONSTANIC results.
> - **Concatenation model**: block until left side is CONSTANT, then right side
>   evaluates with left as its AB (Ancestral Brane). No three-stage isolation/merge.
> - **REPL session**: each line appended as a new statement to one persistent top-level brane.
> - **Phase 2+ (not MVP)**: greedy/parallel descent into branes.

---

## Execution Model (Phase 1)

```
Brane:      breadth-first descent — evaluate all statements at this level before
            descending into nested branes. A brane can hold a mix of CONSTANT
            and CONSTANIC statements.

Expression: lazy — if ANY dependency is missing, the whole expression is CONSTANIC
            immediately (hold AST, no partial eval). No partial computation of
            the resolved operands.

Concatenation A B:  A must be CONSTANT before B starts evaluating.
                    B then evaluates with A prepended to its AB chain.
                    No three-stage isolation/merge.

REPL session:       one growing top-level brane, each line is a new statement.
```

**Why breadth-first**: branes can nest infinitely. We evaluate one level at a time —
all statements at the current brane level first, then recurse into nested branes.
This ensures outer context is established before inner branes resolve against it,
and prevents unbounded recursion from infinitely-nested structures.

This eliminates the WOCONSTANIC race condition and the three-stage concatenation
complexity that caused so much grief in prior implementations.

---

## MVP Targets

| MVP | Goal | Language features in scope |
|-----|------|---------------------------|
| **MVP1** | Language correctness demo | Branes, identification, arithmetic, backward-only search (exact + regex), constanic coordination, concatenation (sequential blocking). Target: all active approval tests pass. |
| **MVP2** | Visual brane browser | Web demo visualizing the MVP1 features — brane tree, CONSTANIC highlighting, live evaluation. No new language features. |
| **MVP3** | Usable language | Forward search (`~`), unanchored seek (`#-N`), detachment, SF/SFF marks, REPL with line editing. Language becomes practically usable. |

**Important note on identifiers**: there are no "variable references" — identifiers
are exact-match search requests. `x` is shorthand for "search backward for the last
statement named `x`". This distinction matters for implementation: lookup is always
search, never a symbol-table pointer dereference.

---

## Phase Structure

| Phase | MVP | Milestone | Description |
|-------|-----|-----------|-------------|
| **P1** | — | Parser + AST wiring | Shared parser produces AST; new Scala module wires it |
| **P2** | — | Literal evaluation | Values, empty brane, output sequencer |
| **P3** | — | Arithmetic | Operators, precedence, NK propagation |
| **P4** | — | Branes + scope | Nested branes, identification, shadowing, scope chain |
| **P5** | — | Constanic | Missing identifiers → CONSTANIC; output symbols |
| **P6** | MVP1 | Anchored search | `.`, `?` (backward), `^`, `$`, `#N`, regex |
| **P7** | MVP1 | Concatenation | Sequential blocking: A CONSTANT → B evaluates with A as AB |
| **P8** | MVP1 | Approval test pass | All 60 active `.foo` tests green; cross-validate vs Java |
| **P9** | MVP2 | Brane browser webapp | JSON API + frontend visualizing brane trees |
| **P10** | MVP3 | Forward search + seek | `~`, `#-N`, unanchored regex |
| **P11** | MVP3 | REPL | Persistent session brane, line editing, multiline input |
| **P12** | MVP3 | Detachment | `[id]{...}`, P-brane, SF/SFF marks |

---

## Phase 1 — Parser + AST Wiring

**Goal**: New Scala module `foolish-core-scala2` (or rename existing) compiles and
can round-trip the parser output.

**What to build**:
- New Maven module (or clean slate in existing `foolish-core-scala`)
- Wire `foolish-parser-java` AST into Scala
- Skeleton `BraneComputer` object: `run(AST.Program): Result`
- Empty approval test harness: `ScUbc2ApprovalTest` that runs `.foo` files

**Tests to pass**: none yet (harness exists, all skip or trivially pass)

**New test to write**: `p1_parser_wiring.foo`
```foolish
{}
```
Just verifies parse → AST → no crash.

---

## Phase 2 — Literal Values + Output

**Goal**: Evaluate literals and format output correctly.

**What to build**:
- `FIR` sealed trait: `Constant(value)`, `Constanic(ast)`, `NK(reason)`
- `ValueFir`: integers, strings, floats
- `BraneFir`: holds list of statements, each is a `(Option[Name], FIR)`
- `Sequencer`: formats output in the approved indented style
- State: CONSTANT, CONSTANIC, NK — just these three for now

**Output symbols** (per UBC2 spec):
- `🧠???` = NK (definite error)
- `🧠??` = CONSTANIC (missing context)
- Value → printed as-is

**Tests to enable** (currently passing in old impl, must pass in new):
```
emptyBraneIsApproved.foo          {}
singleExpressionIsApproved.foo    {42;}
multipleExpressionsIsApproved.foo {1; 2; 3;}
simpleIntegerIsApproved.foo
```

**New test to write**: `p2_literals.foo` — strings, floats, empty brane in one file.

---

## Phase 3 — Arithmetic

**Goal**: Binary and unary operators, precedence, NK propagation.

**Key rule (Phase 1 lazy model)**: `A + B` — if either A or B is CONSTANIC, the whole
expression is CONSTANIC immediately. No partial evaluation of the other operand.
If either is NK → result is NK.

**What to build**:
- `eval(expr: AST.Expr, ab: BraneFir, ib: BraneFir): FIR`
- Binary: `+`, `-`, `*`, `/`, `%`
- Unary: `-`
- Operator precedence handled by parser (ANTLR grammar already correct)
- Division/modulo by zero → NK

**Tests to enable**:
```
simpleAdditionIsApproved.foo
simpleSubtractionIsApproved.foo
simpleMultiplicationIsApproved.foo
simpleDivisionIsApproved.foo
simpleUnaryMinusIsApproved.foo
chainedArithmeticIsApproved.foo
complexArithmeticIsApproved.foo
nestedArithmeticIsApproved.foo
mixedOperatorsIsApproved.foo
operatorPrecedenceIsApproved.foo
multipleArithmeticExpressionsIsApproved.foo
negativeResultsIsApproved.foo
mixedExpressionsIsApproved.foo
zeroDivisionIsApproved.foo
```

**New test to write**: `p3_nk_propagation.foo` — verify CONSTANIC in arithmetic stays CONSTANIC
(not partial), NK from div-zero propagates through chained ops.

---

## Phase 4 — Branes + Identification + Scope

**Goal**: Named statements, nested branes, scope chain (AB/IB model), breadth-first evaluation.

**Key concepts**:
- AB = Ancestral Brane (all context outside the current brane — the scope chain above)
- IB = Immediate Brane (the brane the current statement belongs to)
- Identification: `name = expr` — ordinate stored in IB
- Backward search within IB first (from current statement upward), then up through AB chain
- Characterization: `type'name` — qualifier on identifier
- Identifiers are not variable references — they are exact-match backward search requests

**Evaluation order**:
1. Evaluate all statements of current brane level (breadth-first within the brane)
2. Then recurse into any nested brane statements
3. A nested brane's IB is itself; its AB is the enclosing brane + all outer ABs

**What to build**:
- `Brane` data structure: ordered list of `Statement(index, name, fir)`
- Breadth-first evaluator: pass 1 = evaluate all RHS expressions at current level;
  pass 2 = recurse into nested branes
- `BraneMemory`: append-only, supports backward name lookup with `fromLine` cursor
- AB/IB threading through eval — no mutable parent pointers, pass explicitly
- Shebang line ignored at top of file

**Tests to enable**:
```
simpleIdentifierIsApproved.foo
multipleIdentifiersIsApproved.foo
identifierInExpressionIsApproved.foo
identifierShadowingIsApproved.foo
nestedBranesIsApproved.foo
deeplyNestedBranesIsApproved.foo
nestedBranesWithArithmeticIsApproved.foo
fourLevelNestedBranesWithNamesIsApproved.foo
veryDeepNestingIsApproved.foo
nestedScopeIdentifierIsApproved.foo
nestedScopeShadowingIsApproved.foo
complexIdentifierScopeIsApproved.foo
commentEndsStatement.foo
identifierSeparators.foo
shebang.foo
```

**New test to write**: `p4_scope_chain.foo` — identifier found 3+ levels up through AB chain.

---

## Phase 5 — Constanic State

**Goal**: Missing identifiers produce CONSTANIC (not NK). Output renders correctly.

**Key distinction** (critical — this is where prior impls went wrong):
- Anchored search fails on CONSTANT brane → `NK` (`🧠???`) — brane is done, name not there
- Anchored search on CONSTANIC brane → `NK` (`🧠???`) — can't search a brane that isn't ready
- Unanchored identifier not found in any scope → `CONSTANIC` (`🧠??`) — might resolve later

**What to build**:
- Distinguish NK from CONSTANIC in output sequencer
- `eval` returns `Constanic(ast)` when unanchored lookup fails entire scope chain
- Brane's own state = worst of its statements (CONSTANIC if any statement is CONSTANIC)

**Tests to enable**:
```
constanticRendering.foo
levelSkippingSearchFound.foo
levelSkippingSearchNotFound.foo
levelSkippingSearchConstanic.foo
anchoredSearchOnConstant.foo
anchoredSearchOnConstanic.foo
anchoredSearchFailsOnConstant.foo
regression_disappearing_brane_statements.foo
test_nested_brane_boundary.foo
```

**Reorganize**: Split `levelSkippingSearch*.foo` into cleaner atomic tests:
- `p5_constanic_unanchored_not_found.foo` — bare identifier not found → `🧠??`
- `p5_constanic_chain.foo` — `a = b; b = missing` → both CONSTANIC
- `p5_nk_anchored_not_found.foo` — `brane?missing` on CONSTANT brane → `🧠???`

---

## Phase 6 — Anchored Search

**Goal**: `.`, `?` (backward), `~` (forward), `^` (head), `$` (tail), `#N` (index).

**Rules**:
- All anchored searches are **local to the specified brane only** — never cross boundary
- `brane.name` ≡ `brane?name` — backward, finds last match
- `brane~name` — forward, finds first match
- `brane^` — first statement's value
- `brane$` — last statement's value
- `brane#N` — 0-based forward; negative from end; out-of-bounds → NK
- Regex patterns: `brane?(a.*)`, `brane~e$`
- Search on NK anchor → NK; search on CONSTANIC anchor → NK

**What to build**:
- `Query`: exact name match vs regex pattern
- `BraneMemory.searchBackward(query, fromLine)`: returns last match before fromLine
- `BraneMemory.searchForward(query, fromLine)`: returns first match after fromLine
- `BraneMemory.getByIndex(n)`: handles positive/negative, bounds-check
- Regex: auto-anchor with `^...$` for whole-name match (but don't double-anchor)

**Tests to enable**:
```
oneShotSearchIsApproved.foo
offsetAccess.foo
searchPatternBasicsIsApproved.foo
searchLocalizedVsGlobalizedIsApproved.foo
regexSearchWithPatternIsApproved.foo
regexSearchNotFoundIsApproved.foo
regexSearchShadowy.foo
searchRegexPatternsIsApproved.foo
testTilde.foo
assignmentAnchor.foo
anchoredSearchOnConstant.foo      (already in P5, verify here too)
anchoredSearchFailsOnConstant.foo (already in P5)
test_syntax.foo
```

**New test to write**: `p6_chained_dot.foo` — `a.b.c` multi-level dot chain.

---

## Phase 7 — Unanchored Search + Seek

**Goal**: Bare `name` identifier search (backward + AB chain), `#-N` positional seek.

**Rules**:
- `name` alone: search IB backward from current statement, then AB, then AB's AB...
- `#-N`: N statements back from current position in IB only (boundary-respecting)
- `=$ expr` sugar: `result = expr$` (tail of expr)
- `=^ expr` sugar: `result = expr^` (head of expr)
- `=#N expr` sugar: `result = expr#N`

**What to build**:
- Unanchored search threading AB chain
- `SeekFir`: positional `#-N` within IB, returns CONSTANIC if not enough prior statements
- Sugar desugaring in eval (or in parser — check grammar)

**Tests to enable**:
```
unanchoredSeekBasic.foo
test_unanchored_oneshot.foo
test_nested_brane_boundary.foo  (boundary behaviour of #-N)
```

**Reorganize**: `unanchoredSeekBasic.foo` has 8 sub-cases; split into:
- `p7_seek_basic.foo` — `#-1`, `#-2`
- `p7_seek_boundary.foo` — seek doesn't cross brane boundary
- `p7_seek_sugar.foo` — `=$`, `=^`, `=#N` syntactic sugar

---

## Phase 8 — Concatenation (Sequential Blocking)

**Goal**: `A B` — A must be fully CONSTANT before B evaluates, with A's statements
prepended to B's scope (A becomes part of B's AB).

**The simple model** (learned from prior failures):
```
eval concat(A, B):
  aResult = eval(A)
  if aResult is not CONSTANT → whole concat is CONSTANIC
  bResult = eval(B, ab = aResult prepended to existing AB)
  return bResult (merged brane)
```

No isolation stage. No three-stage process. No cloning during evaluation.
When A is CONSTANT, B just evaluates normally with A as extra scope — finding things
in A that it couldn't find before.

**What to build**:
- `ConcatFir`: evaluates left, blocks if not CONSTANT, then evaluates right with extended AB
- AB threading: when B evaluates, A's statements are the innermost AB layer
- Merging: result brane contains A's statements followed by B's statements

**Tests to enable**:
```
concatenationBasics.foo
concatenationResolution.foo
concatenationSearch.foo
concatenationResolutionAdv.foo
```

**Reorganize**: Existing concatenation tests mix many scenarios; add focused tests:
- `p8_concat_simple.foo` — `{a=1}{b=2}` → both visible
- `p8_concat_resolution.foo` — B resolves identifier from A
- `p8_concat_constanic_left.foo` — left side CONSTANIC → whole thing CONSTANIC
- `p8_concat_chain.foo` — `A B C` left-associative

---

## Phase 9 — REPL MVP1

**Goal**: Working REPL where each line extends a persistent top-level brane.

**Session model**:
```
> x = 42          ← appended as statement 1
> y = x + 1       ← appended as statement 2; sees x from statement 1
=> y = 43
> z = missing     ← appended as statement 3
=> z = 🧠??       ← CONSTANIC (might resolve if user types 'missing = ...' later)
```

**What to build**:
- `FoolishRepl`: accumulates `List[Statement]` as the session brane
- Each input line → parse → append to session brane → re-evaluate from last changed statement
- Display only the result of the last statement (or all if in verbose mode)
- Line editing: jline3 (already in JVM ecosystem, Scala-friendly)
- Error recovery: parse errors print message, don't kill session
- Multiline input: detect unclosed `{` and prompt for continuation

**New tests to write** (REPL-specific, not approval-test style):
- `ReplSessionTest.scala` — unit tests for session accumulation
- `ReplErrorRecoveryTest.scala` — parse errors don't corrupt session

---

## Phase 10 — Detachment (Post-MVP1)

The disabled tests define the scope. Implement in this sub-order:

| Sub-phase | Feature | Tests |
|-----------|---------|-------|
| P10a | Basic `[id]{...}` M-brane | `detachmentAlarms.foo` (test_1) |
| P10b | P-brane `[+id]` partial application | `detachmentPBrane.foo` |
| P10c | Re-detachment | `detachmentComplexTests.foo` (test_re_detachment) |
| P10d | Forward search liberation `[~pat]` | `detachmentForwardSearch.foo` |
| P10e | SF mark `<expr>` | `detachmentSFMark.foo`, `SFMarkWithoutDetachment.foo` |
| P10f | SFF mark `<<expr>>` | `detachmentSFFMark.foo` |
| P10g | Alarm system | `detachmentAlarms.foo` (test_2, test_3) |
| P10h | Complex nested | `detachmentComplexTests.foo` (remaining) |

**Note on SF/SFF semantics**: SF mark `<f>` resolves own symbols only, does not forward
children's searches, does not step found results. SFF mark `<<f>>` skips straight to CONSTANIC
without any resolution. These interact with concatenation to enable late binding.

---

## Phase 11 — Webapp MVP2

**Goal**: Browser UI for navigating and expanding branes.

**Stack** (recommendation, to be confirmed):
- `http4s` + `circe` for JSON API (pure Scala, lightweight)
- Thin JS frontend (or Scala.js) calling REST endpoints
- Endpoints:
  - `POST /eval` — evaluate Foolish source, return brane tree as JSON
  - `GET /brane/:id` — expand a brane node
  - `POST /session` — REPL session management

**Brane tree JSON shape**:
```json
{
  "type": "brane",
  "statements": [
    {"name": "x", "value": {"type": "constant", "v": 42}},
    {"name": "y", "value": {"type": "constanic"}}
  ]
}
```

---

## Test Reorganization Plan

Some existing tests conflate multiple behaviours. Proposed splits:

| Current test | Problem | Split into |
|---|---|---|
| `levelSkippingSearch*.foo` | Three files, overlapping concerns | Merge into `p5_scope_search.foo` with labelled sections |
| `unanchoredSeekBasic.foo` | 8 cases in one file, some boundary tests mixed | Split into `p7_seek_basic`, `p7_seek_boundary`, `p7_seek_sugar` |
| `concatenationResolutionAdv.foo` | Complex interactions, hard to debug regressions | Keep but add `p8_concat_simple.foo` as regression anchor |
| `regexSearchShadowy.foo` | Large compound test, slow to debug | Keep as integration test; add `p6_regex_basic.foo` for unit coverage |

**Policy for new tests**:
- One concept per file where possible
- File name starts with phase prefix (`p3_`, `p6_`, etc.) for new tests
- Legacy test names kept as-is for continuity

---

## Dependency Graph

```
P1 (parser) → P2 (literals) → P3 (arithmetic)
                            → P4 (branes/scope) → P5 (constanic)
                                                 → P6 (anchored search)
                                                 → P7 (unanchored/seek)
                                               P5 + P6 + P7 → P8 (concatenation)
                                                             → P9 (REPL MVP1)
                                                             → P10 (detachment)
                                                             P9 → P11 (webapp MVP2)
```

---

## What NOT to Port from UBC1

Per implementation lessons:

| UBC1 component | Reason to skip |
|---|---|
| `CMFir` (two-phase re-eval) | Replaced by simple AB extension in concatenation |
| `ExecutionFir` / `FiroeWithBraneMind` | Message-passing infrastructure — Phase 2+ only |
| `IfFiroe` | Removed from UBC2 design; infinite recursion bug; search-based selection instead |
| `SearchUpFiroe` (↑) | Advanced feature, post-MVP |
| Three-stage concatenation | Replaced by sequential blocking model |
| `WOCONSTANIC` state | Phase 1 doesn't need it — expression is either CONSTANT or CONSTANIC |
| Cursor/index system | Replace with simple list indexing |

---

## Last Updated

**Date**: 2026-04-30
**Updated By**: Claude Code claude-sonnet-4-6
**Changes**: Initial creation — implementation schedule for Scala MVP based on full
docs/codebase read and human Q&A about root causes of prior failures.
