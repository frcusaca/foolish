# Phase 2 — UBC: Depth-First Sequential Step Evaluation

> Goal: Read FIRs (in memory or via Circe deserialization), step the FIR tree
> until every node reaches a constanic state (CONSTANT, INDEPENDENT, ECONSTANIC,
> WOCONSTANIC) or NK. The hard part is **search short-circuiting** through
> WOCONSTANIC chains and **constanic cloning** when search results need to be
> placed in their searcher's context.

> Phase 2 is **depth-first, sequential, no message passing**. We adopt UBC2's
> Nyes lifecycle and FIR taxonomy but evaluate by direct function-call stepping.
> Breadth-first parallel evaluation is deferred to Phase 4. See FOOP-6.

Read [00_accumulated_specs.md](00_accumulated_specs.md) for the Nyes state
definitions before reading this document.

---

## Phase 2 Deliverable

A `Ubc.step(fir: Fir): Fir` function that performs one evaluation step on a FIR
tree, returning the (possibly partly) advanced FIR. Plus a
`Ubc.runToCompletion(fir: Fir): Fir` that loops `step` until no FIR can make
progress.

The 60 `.foo` approval tests in `src/test/resources/.../inputs/` move into Phase 2
as the validation suite. Expected output format mirrors the Java UBC1 format
(`Sequencer4Human` style — see `02_implementor_reference.md`).

---

## Evaluation Order: Depth-First, Left-to-Right, Sequential

Phase 2 evaluates a brane as follows:

1. Walk the brane's statements **left-to-right** (the order they appear in the source).
2. For each statement, **step its body to a constanic terminal state** (CONSTANT,
   INDEPENDENT, ECONSTANIC, WOCONSTANIC, or NK) before moving to the next statement.
3. Stepping a body that is itself a brane recurses into it, walking *its* statements
   depth-first left-to-right.
4. There is no concurrency, no parallelism, no message passing. Each `step()` call
   does one unit of evaluation work and returns.

This guarantees that **when statement N is being evaluated, all statements 0..N-1
in the same brane (and all their descendants) have already reached constanic
terminal states**. This is the foundation that enables search short-circuiting
to be a simple in-step operation rather than a wake-up-queue mechanism.

**Why depth-first and not breadth-first?** Breadth-first is correct (and is the
target of Phase 4), but the bookkeeping required to interleave subtree evaluations
correctly — wake-up queues, dependency tracking maps, the H-uman-style coordination
mechanism — is substantial. Phase 2 deliberately punts these to Phase 4 so the
core semantics (search resolution, constanic cloning, NK propagation, regex
matching, scope walks) can be exercised against the full approval test suite first.

---

## The Hard Part — Search Short-Circuiting and Constanic Coordination

This section is the core of Phase 2's complexity. It is best understood through a
worked example. Read it carefully before implementing.

### The Worked Example

```
{
  y = z,
  x = y,
  w = x,
  v = w + x,
  u = v + w
}
```

Walk through this statement by statement, applying the rules below.

### Statement 1: `y = z`

`z` is an unanchored backward search for the name `z`. There is nothing before
`y` in this brane, and the brane has no Ancestral Brane (it's the top level), so
the search exhausts the scope chain finding nothing.

After `y = z` is stepped:

| Field | Value |
|-------|-------|
| `y`'s body (a `SearchFir`) | state = `ECONSTANIC` |
| `y` (the `StatementFir`) | state = `WOCONSTANIC` (because its body is constanic-but-not-constant) |

### Statement 2: `x = y`

`y` is an unanchored backward search for the name `y`. The search succeeds — it
finds the statement `y = z` from line 1.

The search result is `y`'s value FIR — the `SearchFir` in ECONSTANIC state.

**Now `constanicClone` is invoked on the search result** (see "Constanic Cloning"
section below). For an ECONSTANIC FIR, `constanicClone` produces a clone with
state reset to EMBRYONIC, parented to `x`'s brane.

After `x = y` is stepped:

| Field | Value |
|-------|-------|
| `x`'s body (the `SearchFir` for `y`) | resolved; its `target` field points at the clone of `y`'s ECONSTANIC search FIR |
| `x` (the `StatementFir`) | state = `WOCONSTANIC` |

The cloned ECONSTANIC search FIR re-runs in its new context (which is the same
brane, so it finds nothing again) and remains ECONSTANIC.

### Statement 3: `w = x`

Same pattern as `x = y`. The search for `x` finds the statement `x = y` from
line 2. The search result is `x`'s body — a `SearchFir` whose `target` points at
the clone of `y`'s ECONSTANIC.

`x`'s body is in **WOCONSTANIC** state — it's a search whose target is
ECONSTANIC. This is where **search short-circuiting** kicks in:

> When a search resolves to a target whose state is WOCONSTANIC, follow the
> target's `target` chain through subsequent WOCONSTANICs until you arrive at
> the underlying ECONSTANIC (or CONSTANT). Rewrite the new search's `target`
> field to point directly at the chain's end.

After `w = x` is stepped:

| Field | Value |
|-------|-------|
| `w`'s body (the `SearchFir` for `x`) | `target` points DIRECTLY at the cloned ECONSTANIC (not at `x`'s body) |
| `w` (the `StatementFir`) | state = `WOCONSTANIC` |

So `w` skips the indirection through `x` entirely. This is the optimization that
keeps a chain of `n` identifier indirections to a single shared underlying FIR.

**Why search FIRs short-circuit but expressions and branes don't:** a search
just *points* at its result. An expression *computes* on its result. If `w + x`
were a search that "happened to point at" some FIR, short-circuiting would lose
the addition. Searches can short-circuit precisely because they do nothing with
the result except hold a reference to it.

### Statement 4: `v = w + x`

`v`'s body is a `BinaryOpFir("+", searchForW, searchForX)`. Phase 2 steps it as
follows:

1. Step `searchForW` to completion.
   - Search for `w` finds line 3's `w = x`. Result: `w`'s body, a `SearchFir`
     whose `target` already points at the cloned ECONSTANIC.
   - `constanicClone` on this WOCONSTANIC search FIR produces a clone (per the
     d0_5 WOCONSTANIC rule, clone enters BRANING).
   - The clone re-steps. Its target was already a direct reference to the
     ECONSTANIC, so the re-step preserves that reference. Clone settles to
     WOCONSTANIC.
   - `searchForW`'s `target` short-circuits through the WOCONSTANIC clone to the
     underlying ECONSTANIC.
2. Step `searchForX` to completion. Same outcome: `target` short-circuits to the
   ECONSTANIC.
3. Step the `BinaryOpFir` itself.
   - Both operands are WOCONSTANIC searches.
   - Per the rule "an expression with at least one constanic operand is itself
     constanic," the BinaryOp transitions to WOCONSTANIC.
   - **Important: no short-circuiting applies here.** `v + w` is a computation.
     Its WOCONSTANIC state means "I will compute when my operands resolve."

After `v = w + x` is stepped:

| Field | Value |
|-------|-------|
| `v`'s body (the `BinaryOpFir`) | state = `WOCONSTANIC`; operands point (short-circuited) at the underlying ECONSTANIC |
| `v` (the `StatementFir`) | state = `WOCONSTANIC` |

### Statement 5: `u = v + w`

Same pattern. `u` is a `BinaryOpFir` over two searches:

- `searchForV` resolves to `v`'s body, which is itself a `BinaryOpFir` in
  WOCONSTANIC state. **Search short-circuiting does not apply here** — the search
  result is a BinaryOpFir, not a search-pointing-at-a-search. The search target
  stays as the BinaryOp.
- `searchForW` resolves the same way as in statement 4 — short-circuits to the
  underlying ECONSTANIC.

After `u = v + w` is stepped:

| Field | Value |
|-------|-------|
| `u`'s body (the `BinaryOpFir`) | state = `WOCONSTANIC`; operands are the WOCONSTANIC v-BinaryOp and the short-circuited ECONSTANIC |
| `u` (the `StatementFir`) | state = `WOCONSTANIC` |

### Final State

After Phase 2 steps the entire brane to completion:

| Statement | State | Notes |
|-----------|-------|-------|
| `y = z` | WOCONSTANIC | body is the ECONSTANIC search for `z` |
| `x = y` | WOCONSTANIC | body's target → cloned ECONSTANIC |
| `w = x` | WOCONSTANIC | body's target → cloned ECONSTANIC (short-circuit) |
| `v = w + x` | WOCONSTANIC | BinaryOp; operands → cloned ECONSTANIC (each short-circuited) |
| `u = v + w` | WOCONSTANIC | BinaryOp; operands are v's BinaryOp + cloned ECONSTANIC |
| (the brane itself) | WOCONSTANIC | because at least one statement is constanic-not-constant |

---

## Constanic Cloning — `constanicClone(R, newParent)`

**Rule: every search result goes through `constanicClone` before being
attached as a search's target.** The function is a no-op for CONSTANT/INDEPENDENT
and does real work for ECONSTANIC/WOCONSTANIC. Calling sites do not need to
branch on the search result's state.

```scala
def constanicClone(original: Fir, newParent: Fir): Fir = original.state match
  case Nyes.CONSTANT | Nyes.INDEPENDENT =>
    original                                         // share, do not clone

  case Nyes.NK =>
    original                                         // terminal, no recoordination

  case Nyes.ECONSTANIC =>
    val clone = original.deepCopyResettingTo(Nyes.EMBRYONIC, newParent)
    clone                                            // re-step to retry search in new context

  case Nyes.WOCONSTANIC =>
    val clonedChildren = original.children.map(c => constanicClone(c, /* clone */))
    val clone          = original.deepCopyWithChildren(clonedChildren, Nyes.BRANING, newParent)
    clone                                            // re-step to recurse into children

  case Nyes.PREMBRYONIC | Nyes.EMBRYONIC | Nyes.BRANING =>
    sys.error(s"constanicClone called on nigh FIR (state=${original.state})")
```

### Per-state clone behavior table

| Original state | Clone behavior | Initial Nyes of clone | Why |
|----------------|---------------|----------------------|-----|
| CONSTANT | not cloned, share reference | (n/a) | immutable, safe to share |
| INDEPENDENT | not cloned, share reference | (n/a) | literal — recoordination cannot change it |
| NK | not cloned, share reference | (n/a) | terminal, no work for recoordination to do |
| ECONSTANIC | deep copy, re-parent | EMBRYONIC | re-runs the search in new context |
| WOCONSTANIC | deep copy with cloned children, re-parent | BRANING | re-steps children which may now resolve |
| PREMBRYONIC, EMBRYONIC, BRANING | error (caller bug) | (n/a) | callers must step to constanic before cloning |

### Why uniform invocation matters

Phase 2 uses `constanicClone` even when there is no concatenation (no actual
context change). For example, `{a = unknown, y = a}` — the search `a` finds
`unknown`'s ECONSTANIC search FIR, and `constanicClone` clones it before binding
to `y`'s body. The clone re-runs the search in `y`'s context, which is the same
context, so it finds the same nothing.

This is intentional. By making `constanicClone` uniform, Phase 6 (concatenation)
inherits the recoordination machinery for free — concatenation is just another
caller of `constanicClone` with a different `newParent`.

---

## Per-FIR Step Rules

### `ConstantIntFir`

Already INDEPENDENT (or CONSTANT). `step()` is a no-op.

### `NKFir`

Already NK. `step()` is a no-op.

### `NormalBraneFir`

```
step(NormalBraneFir(chars, statements, state)):
  state' = match state
    case PREMBRYONIC -> EMBRYONIC                  // entering evaluation
    case EMBRYONIC   -> BRANING                    // immediately moves on to step children
    case BRANING     ->
      stepChildrenLeftToRight(statements)
      if all(statements.state in [CONSTANT, INDEPENDENT]) then CONSTANT
      else if any(statements.state == NK)           then BRANING  // anonymous nks; brane survives
      else if any(statements.state in [ECONSTANIC, WOCONSTANIC]) then WOCONSTANIC
      else BRANING                                 // still progressing
    case _ -> state                                // already terminal
```

A brane reaches CONSTANT only when every statement is CONSTANT or INDEPENDENT.
A brane is WOCONSTANIC if any statement is constanic-but-not-constant.

### `StatementFir`

```
step(StatementFir(name, body, state)):
  body' = stepToCompletion(body)
  state' = body'.state                             // the statement's state mirrors its body
```

### `BinaryOpFir`

```
step(BinaryOpFir(op, left, right, state)):
  left'  = stepToCompletion(left)
  right' = stepToCompletion(right)
  state' = match (left'.state, right'.state)
    case (NK, _) | (_, NK)                                                -> NK
    case (CONSTANT|INDEPENDENT, CONSTANT|INDEPENDENT) ->
      compute(op, left', right')                                          // returns ConstantIntFir or NK (e.g., div-by-zero)
    case (cs, _) if isConstanic(cs)                                       -> WOCONSTANIC
    case (_, cs) if isConstanic(cs)                                       -> WOCONSTANIC
    case _                                                                -> BRANING
```

Note `compute(op, l, r)` performs the actual arithmetic. Phase 1 deliberately
did not do this (FOOP-5); Phase 2 does.

### `UnaryOpFir`

Same shape as `BinaryOpFir` with one operand.

### `SearchFir` (unanchored)

```
step(SearchFir(pattern, Backward, anchored=false, anchor=None, state)):
  match state:
    case PREMBRYONIC | EMBRYONIC ->
      // Walk IB backward from current position; if not found, walk AB chain.
      result = scopeWalk(pattern, parentBrane)
      result match
        case None                  -> state' = ECONSTANIC; target = None
        case Some(found) ->
          // Apply constanicClone — recoordination per FOOP-7
          target = constanicClone(found, /* this search's brane */)
          state' = match target.state
            case CONSTANT | INDEPENDENT -> CONSTANT
            case ECONSTANIC | WOCONSTANIC ->
              shortCircuitWoconstanic(target)
              WOCONSTANIC
            case NK                     -> NK
    case _ -> state                                // already terminal
```

### `SearchFir` (anchored)

```
step(SearchFir(pattern, dir, anchored=true, Some(anchor), state)):
  // First step the anchor to completion
  anchor' = stepToCompletion(anchor)
  match anchor'.state:
    case NK                                  -> state' = NK
    case CONSTANT | INDEPENDENT ->
      // Search ONLY within anchor's brane statements; no scope walk.
      result = anchor'.searchLocally(pattern, dir)
      result match
        case None        -> state' = NK              // anchored miss on CONSTANT brane → NK (not ECONSTANIC)
        case Some(found) -> target = constanicClone(found, ...); state' = ...
    case ECONSTANIC | WOCONSTANIC ->
      // Cannot search a brane that itself isn't ready
      state' = NK
```

### `IndexFir`, `HeadTailFir`

Same pattern as `SearchFir` — step the anchor (if any), then locate by index or
head/tail position. CONSTANT result → CONSTANT; out-of-bounds on a CONSTANT
brane → NK.

### `CharacterizedRefFir`

Same pattern as unanchored `SearchFir` but the resolution rule includes
characterization matching (deferred design — Phase 2 implementation may treat
characterizations as part of the regex pattern with a custom matcher).

---

## Search Short-Circuit Algorithm

```scala
def shortCircuitWoconstanic(search: SearchFir): Unit =
  // Walk through chained WOCONSTANIC search targets to the underlying
  // ECONSTANIC (or CONSTANT). Rewrite search.target to skip intermediates.
  var current = search.target
  while current.isInstanceOf[SearchFir] && current.state == Nyes.WOCONSTANIC do
    current = current.asInstanceOf[SearchFir].target
  search.target = current
```

The loop terminates because each WOCONSTANIC search points to either another
WOCONSTANIC search, an ECONSTANIC search, a CONSTANT FIR, or a non-search FIR
(which terminates the chain). Cycles are impossible by Foolish's writing-order
rule (a name is not visible to its own RHS).

---

## What Phase 2 Does NOT Do

- **No concatenation** — `ConcatenationAstn` is rejected by the compiler (FOOP-3
  scope, Phase 6).
- **No detachment, SF, SFF** — Phase 7.
- **No REPL recoordination** — REPL lines are appended; backward search means a
  later definition cannot retroactively resolve an earlier ECONSTANIC. Phase 3.
- **No breadth-first / parallel** — Phase 4.
- **No wake-up queues, no dependency tracking maps, no message passing** — these
  are Phase 4 concerns.

---

## Tests

The 60 active `.foo` files in `src/test/resources/.../inputs/` move into Phase 2
as the validation suite. The `.tbd` files become approval tests as their
expected output is reviewed.

Test harness: re-introduce the `ApprovalTestRunner` Java helper plus a Scala
interpreter implementing `UbcTester` that pipes source through
`Compiler.compileToJson` then `Ubc.runToCompletion`.

Output format: `Sequencer4Human` style. The Phase 2 sequencer must distinguish:

| State | Symbol |
|-------|--------|
| ECONSTANIC | `🧠??` (the standard CONSTANIC symbol — UBC2 docs sometimes use `🧠?` for WOCONSTANIC, `🧠??` for ECONSTANIC; we reserve final symbol choice for the sequencer implementation) |
| WOCONSTANIC | (TBD per sequencer) |
| NK | `🧠???` |
| CONSTANT/INDEPENDENT integer | the number, e.g., `42` |

---

## Phase 2 Exit Criteria

- All 60 active `.foo` approval tests pass.
- All 5 `.tbd` tests have approved baselines (manually reviewed, never
  bulk-approved).
- The worked example brane at the top of this document has a unit test
  asserting the expected final-state table.
- `Ubc.step` is idempotent on terminal states (calling `step` on a CONSTANT,
  INDEPENDENT, ECONSTANIC, WOCONSTANIC, or NK FIR returns it unchanged).
- `constanicClone` has unit tests for each of the 5 input states (CONSTANT,
  INDEPENDENT, NK, ECONSTANIC, WOCONSTANIC).

---

## Last Updated

**Date**: 2026-05-01
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: Major rewrite. Adopted UBC2 Nyes terminology; added worked example
walking through ECONSTANIC/WOCONSTANIC/short-circuiting; added per-FIR step
rules; added `constanicClone` algorithm and per-state clone behavior table.
Phase 2 is depth-first sequential; breadth-first deferred to Phase 4.
