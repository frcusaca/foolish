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
| **MVP1** | Language correctness demo | Branes, identification, arithmetic, **unanchored backward search** (bare identifier + `#-N` + regex), anchored search (`.`, `?`, `^`, `$`, `#N`), constanic coordination, concatenation (sequential blocking). Target: all active approval tests pass. |
| **MVP2** | Visual brane browser (LOD) | Web demo with **level-of-detail viewport** — screen shows a consecutive window into a brane; multiple viewports possible. Navigation powered by MVP1 search. No new language features. |
| **MVP3** | Usable language | Forward search (`~`), REPL with persistent session + line editing, detachment, SF/SFF marks. |

**Why unanchored search is MVP1, not MVP3**: it is the *only* mechanism that produces
CONSTANIC results. When you write `user_name` as a bare identifier, that is an unanchored
backward search. If `user_name` is not found in the current IB or any AB, the expression
becomes CONSTANIC — which is a core language property, not an advanced feature. Without
unanchored search there is no constanic, and without constanic there is no concatenation
semantics. It must be in MVP1.

**Important note on identifiers**: there are no "variable references" — all identifiers
are unanchored backward search requests. `x` means "search backward through IB from the
current statement, then up through each AB, and return the last statement named `x`."
This is why a reference to a CONSTANIC statement makes the referencing expression
CONSTANIC too — you found the thing you were searching for, but it hasn't resolved yet.

**MVP2 LOD design**: a brane can have thousands of statements; the screen is finite.
The viewer shows a *window* of consecutive statements within a brane, with controls to
scroll. Multiple viewports can be open simultaneously, each focused on a different brane
or a different region of the same brane. The MVP1 search operators (`?`, `~`, `#N`, etc.)
drive navigation — jump to a named statement, regex-search within a brane, go to head/tail.

---

## Phase Structure

| Phase | MVP | Milestone | Description |
|-------|-----|-----------|-------------|
| **P1** | — | Parser + AST wiring | Shared parser produces AST; new Scala module wires it |
| **P2** | — | Literal evaluation | Values, empty brane, output sequencer |
| **P3** | — | Arithmetic | Operators, precedence, NK propagation |
| **P4** | — | Branes + scope | Nested branes, identification, shadowing, breadth-first eval |
| **P5** | — | Unanchored search + constanic | Bare identifier = unanchored backward search; `#-N` seek; CONSTANIC when not found |
| **P6** | MVP1 | Anchored search | `.`, `?` (backward), `~` forward within brane, `^`, `$`, `#N`, regex |
| **P7** | MVP1 | Concatenation | Sequential blocking: A CONSTANT → B evaluates with A as AB |
| **P8** | MVP1 | Approval test pass | All 60 active `.foo` tests green; cross-validate vs Java |
| **P9** | MVP2 | LOD brane browser | JSON API + windowed viewport frontend; search-driven navigation |
| **P10** | MVP3 | REPL | Persistent session brane, line editing, multiline input |
| **P11** | MVP3 | Detachment | `[id]{...}`, P-brane `[+id]`, SF/SFF marks |

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

## Phase 5 — Unanchored Search + Constanic

**Goal**: Bare identifiers and `#-N` are unanchored backward searches. When an
unanchored search finds nothing, the expression is CONSTANIC. When it finds a
CONSTANIC result, the referencing expression is also CONSTANIC.

**This phase is the heart of the language.** Without unanchored search there is no
constanic, and without constanic there is no meaningful concatenation.

**Key distinction** (critical — this is where prior impls went wrong):
- `user_name` (bare identifier) = unanchored backward search through IB then AB chain
  - Found, CONSTANT result → expression is CONSTANT
  - Found, CONSTANIC result → expression is CONSTANIC (you found it, but it's not ready)
  - Not found anywhere → expression is CONSTANIC (might resolve in a new context)
- `brane?name` (anchored) fails on CONSTANT brane → `NK` (`🧠???`) — definitely not there
- `brane?name` on CONSTANIC brane → `NK` (`🧠???`) — can't search a brane that isn't ready
- `10 / 0`, depth exceeded, etc. → `NK` (`🧠???`) — definitively unknown

**What to build**:
- Unanchored search: walk IB backward from `fromLine`, then each AB in order
- `#-N` seek: N steps back within IB only; CONSTANIC if not enough prior statements
- Distinguish NK from CONSTANIC in output sequencer (`🧠???` vs `🧠??`)
- `eval` returns `Constanic(ast)` when unanchored lookup finds nothing
- When found result is itself CONSTANIC → propagate CONSTANIC
- Brane's state = worst of its statements (CONSTANIC beats CONSTANT)

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
unanchoredSeekBasic.foo
test_unanchored_oneshot.foo
```

**Reorganize**: Split `levelSkippingSearch*.foo` into cleaner atomic tests:
- `p5_constanic_unanchored_not_found.foo` — bare identifier not found → `🧠??`
- `p5_constanic_chain.foo` — `a = b; b = missing` → both CONSTANIC
- `p5_constanic_found_constanic.foo` — `a = missing; b = a` → b is CONSTANIC because a is
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

## Phase 7 — Concatenation (Sequential Blocking)

**Goal**: `A B` — A must be fully CONSTANT before B evaluates, with A's statements
prepended to B's scope (A becomes innermost layer of B's AB chain).

**The simple model** (learned from prior failures):
```
eval(concat(A, B), ab, ib):
  aResult = eval(A, ab, ib)
  if aResult is not CONSTANT → whole concat is CONSTANIC (hold AST)
  bResult = eval(B, ab = aResult :: ab, ib)
  return merged brane (A statements followed by B statements)
```

No isolation stage. No cloning. No three-stage process. B's unanchored searches
find things in A first (A is the nearest AB layer), then look further up.

**What to build**:
- `eval` case for `AST.Concat`: evaluate left, gate on CONSTANT, evaluate right with extended AB
- Merged result: A's statements followed by B's, presented as a single brane value

**Tests to enable**:
```
concatenationBasics.foo
concatenationResolution.foo
concatenationSearch.foo
concatenationResolutionAdv.foo
```

**New focused tests**:
- `p7_concat_simple.foo` — `{a=1}{b=2}` → both visible in result
- `p7_concat_resolution.foo` — B finds identifier from A
- `p7_concat_constanic_left.foo` — left CONSTANIC → whole concat CONSTANIC
- `p7_concat_chain.foo` — `A B C` left-associative: `(A B) C`

---

## Phase 8 — Approval Test Pass (MVP1 complete)

**Goal**: All 60 active `.foo` tests pass. Cross-validate output matches Java impl.

**What to do**:
- Run `ScUbc2ApprovalTest` against all active inputs
- For any mismatch: read `.received.foo` vs `.approved.foo` side-by-side, understand before approving
- Do NOT bulk-approve — each baseline must be understood (lesson from commit `0032e474`)
- Run cross-validation against Java UBC1 output; document any intentional differences

**Sugar forms to verify** (all covered by existing tests):
- `=$ expr` — tail sugar (`assignmentAnchor.foo`, `test_unanchored_oneshot.foo`)
- `=^ expr` — head sugar
- `=#N expr` — index sugar

---

## Phase 9 — LOD Brane Browser (MVP2)

**Goal**: Web viewer with level-of-detail windowed viewport into brane trees.

**Core insight**: a brane can have thousands of statements; the screen is finite.
The viewer shows a *window* of consecutive statements. Multiple viewports can be
open simultaneously, each focused on a different brane or region.

**Navigation is powered by MVP1 search**:
- Jump to named statement via `?name` or `~name`
- Go to `^` (head) or `$` (tail)
- Jump to `#N` by index
- Regex search within a brane

**What to build**:
- `POST /eval` — evaluate Foolish source, return brane tree as JSON
- `GET /brane/:id/window?from=N&size=M` — windowed view of consecutive statements
- `GET /brane/:id/search?q=pattern&dir=backward` — search within a brane, return index
- Frontend: scrollable statement list, CONSTANIC statements visually distinct (`🧠??`),
  click nested brane → open new viewport, search box drives navigation

**Brane window JSON shape**:
```json
{
  "braneId": "abc123",
  "totalStatements": 1200,
  "window": { "from": 40, "size": 20 },
  "statements": [
    {"index": 40, "name": "x", "state": "constant", "value": 42},
    {"index": 41, "name": "y", "state": "constanic"},
    {"index": 42, "name": null, "state": "constant", "value": {"type": "brane", "id": "def456", "size": 5}}
  ]
}
```

**Stack**: http4s + circe (pure Scala, lightweight); thin JS or htmx frontend.

---

## Phase 10 — REPL (MVP3)

**Goal**: Interactive session where each line extends a persistent top-level brane.

**Session model**:
```
> x = 42          ← statement 1; CONSTANT
> y = x + 1       ← statement 2; sees x → CONSTANT 43
=> y = 43
> z = missing     ← statement 3; CONSTANIC
=> z = 🧠??
> missing = 7     ← statement 4; now z can re-evaluate
=> missing = 7
=> z = 🧠??       ← still CONSTANIC: re-eval on next step or explicit :eval command
```

**What to build**:
- Session brane accumulates statements; later lines see earlier names via unanchored search
- jline3 for line editing and history
- Error recovery: parse errors print message, don't kill session
- Multiline: detect unclosed `{`, prompt for continuation with `..` prefix

---

## Phase 11 — Detachment (MVP3)

The 7 disabled tests define the scope. Implement in sub-order:

| Sub-phase | Feature | Tests |
|-----------|---------|-------|
| P11a | Basic `[id]{...}` M-brane | `detachmentAlarms.foo` (test_1) |
| P11b | P-brane `[+id]` partial application | `detachmentPBrane.foo` |
| P11c | Re-detachment | `detachmentComplexTests.foo` (test_re_detachment) |
| P11d | Forward search liberation `[~pat]` | `detachmentForwardSearch.foo` |
| P11e | SF mark `<expr>` | `detachmentSFMark.foo`, `SFMarkWithoutDetachment.foo` |
| P11f | SFF mark `<<expr>>` | `detachmentSFFMark.foo` |
| P11g | Alarm system | `detachmentAlarms.foo` (test_2, test_3) |
| P11h | Complex nested + curry chains | `detachmentComplexTests.foo` (remaining) |

**Note on SF/SFF**: SF `<f>` resolves own symbols only, no child forwarding, does not
step found results. SFF `<<f>>` skips straight to CONSTANIC. Both interact with
concatenation to enable late binding and partial application.

---

## Phase 12 — Forward Search (MVP3 polish)

**Goal**: `~` operator (forward search within a named brane, finds first match).
Deferred to MVP3 because: it requires a named brane to already be CONSTANT, doesn't
interact with constanic/concatenation semantics, and is not needed for the approval
tests that cover the core language.

**Tests to enable**: `testTilde.foo`, `detachmentForwardSearch.foo` (partially)

---

## Phase 13 — Webapp MVP2

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
                            → P4 (branes/scope) → P5 (unanchored search + constanic)  ← MVP1 foundation
                                                 → P6 (anchored search)
                                               P5 + P6 → P7 (concatenation)
                                                       → P8 (approval test pass)      ← MVP1 complete
                                                       → P9 (LOD brane browser)       ← MVP2
                                                       → P10 (REPL)                   ← MVP3
                                                       → P11 (detachment)             ← MVP3
                                                       → P12 (forward search)         ← MVP3 polish
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
