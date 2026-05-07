# Phase 2 — SF/SFF/Seek Implementation Insights

> This document captures lessons learned from implementing StayFoolish (`<expr>`),
> StayFullyFoolish (`<<expr>>`), and unanchored seeks (`#-N`) in the Rust MVP.
> These features were originally planned for Phase 7 but proved implementable
> as part of Phase 2 due to their reliance on core UBC mechanics rather than
> detachment.

---

## What Was Implemented

| Feature | Syntax | Description |
|---------|--------|-------------|
| SFF | `<<expr>>` / `a=<<=>>expr` | Stringent quote — blocks ALL search expansion |
| SF | `<expr>` / `a=<=>expr` | Resolves non-brane searches, blocks brane references |
| Unanchored seek | `#-N` | Positional index relative to statement position in brane |
| Anchored seek | `a#N` | Positional index within anchored brane |

---

## Key Design Decisions

### 1. SFF Retains Independent State (not Embryonic)

`reset_searches()` resets most FIRs to Embryonic so they re-evaluate on the
second brane pass. SFF must NOT be reset — it's a terminal marker that blocks
all expansion. Setting `state: Nyes::Independent` prevents the brane from
getting stuck at BRANING waiting for SFF to resolve.

```rust
// Correct: SFF retains Independent
Fir::StayFullyFoolish { expr, state } => {
    Fir::StayFullyFoolish { expr, state: Nyes::Independent }
}
```

**Insight for Phase 5 (breadth-first):** SFF is a natural "already complete"
node. The breadth-first evaluator can skip SFF entirely without queuing.

### 2. SF Uses Step-Except-Branes (not regular stepping)

SF needs special evaluation: resolve all searches EXCEPT those targeting branes.
This is implemented as `step_except_brane_searches()` which walks the FIR tree
and econstanic's searches that would resolve to brane targets.

**Insight for Phase 7 (detachment):** SF blocking is purely local — it doesn't
need AB/IB context or detachment. This means SF can work independently of
detachment, which simplifies the Phase 7 scope.

### 3. `constanic_clone` Takes `permit_nye: bool`

Normal `constanic_clone` panics on nye (non-constanic) FIRs. For SF/SFF
coordination, we need to copy FIRs that haven't finished evaluating. The
`permit_nye` flag allows this without changing the normal panic behavior.

**Insight for Phase 5:** The `permit_nye` flag is specifically for the SF/SFF
coordination pattern. Breadth-first evaluation should use the strict (panic on
nye) version, since it only clones completed nodes.

### 4. `strip_sf_wrapper` for Binary/Unary Operations

When SFF or SF appears as an operand of an arithmetic operation, the wrapper
must be stripped before evaluation. `strip_sf_wrapper()` recursively peels
wrapper layers to reach the actual value.

```rust
fn strip_sf_wrapper(fir: Fir) -> Fir {
    match fir {
        Fir::StayFullyFoolish { expr, .. } => strip_sf_wrapper(*expr),
        Fir::StayFoolish { expr, .. } => strip_sf_wrapper(*expr),
        other => other,
    }
}
```

**Insight for Phase 5:** `resolve_to_value()` already strips SF/SFF. In breadth-
first, operands can use `resolve_to_value()` before arithmetic, eliminating the
need for `strip_sf_wrapper` as a separate function.

### 5. RefCell Borrow Management

The `step_except_brane_one` function initially held a `borrow()` of the FIR
while trying to `borrow_mut()` for mutations — causing panic. The fix:
extract variant data into a helper enum, then drop the borrow before mutating.

```rust
enum Variant {
    UnanchoredSearch(String),
    AnchoredSearch,
    BinaryOp(String, Box<Fir>, Box<Fir>),
    UnaryOp(String, Box<Fir>),
    StayFullyFoolish,
    Terminal,
    Other,
}
```

**Insight for Phase 5:** This pattern will be needed for any function that
reads the FIR variant and then mutates it. Consider a `Fir::into_variant()`
method that consumes the FIR and returns an enum for safe mutation.

### 6. Unanchored Seek Index Calculation

`#-N` from statement at position `stmt_idx` resolves to `stmt_idx + offset`.
The `Scope` struct carries `current_brane` and `current_stmt_idx` for this
calculation. Negative results are clamped to 0 by `index_in_brane()`.

**Insight for parser:** The current parser rejects positive unanchored seeks
(`#0`, `#1`). The spec says `#0` should refer to the statement's own line.
Parser fix needed: allow `#0` and positive offsets, pass them to UBC for
resolution.

### 7. Scope's `block_brane_searches` Flag

SF context sets `block_brane_searches: true` on the Scope. When `step_search()`
finds a brane target and this flag is set, it econstanic's the search instead
of resolving it. This is simpler than having SF maintain its own scope chain.

**Insight for Phase 5:** This is a perfect example of Scope carrying evaluation
context. Breadth-first will need the same flag, but it should be per-thread
context rather than a shared Scope field.

---

## Blockers for Phase 5 (Breadth-First)

1. **Current evaluation is depth-first, recursive** — `run_to_completion()`
   calls `step_boxed()` which calls `run_to_completion()` on children. Breadth-
   first needs a queue-based approach with a `braneMind` (LinkedList of FIRs).

2. **Scope is cloned for each statement** — the `with_brane()` method creates
   a new Scope. Breadth-first will need a mutable, shared Scope that multiple
   evaluators can contribute to.

3. **`step_except_brane_searches` is self-contained** — it has its own loop.
   Breadth-first will need this logic integrated into the main queue loop.

4. **No alarm system** — the Java/Scala implementations have alarms for
   evaluation events. Breadth-first will need this for observability.

---

## Blockers for Phase 7 (Detachment)

1. **Detachment brane FIR variant** — not yet implemented. Needs AB/IB tracking.

2. **Re-detachment logic** — when a detached brane is referenced in a new
   context, it needs to be cloned with new AB/IB.

3. **P-brane (partial application)** — `[+id]{...}` syntax for curried branes.

4. **Forward search liberation** — `[~pat]` syntax.

---

## Test Coverage

16 approval tests now cover SF/SFF/Seek:

| Test | What it proves |
|------|----------------|
| `sff_basic` | SFF blocks search expansion; resolves when used in arithmetic |
| `sff_nested` | Nested SFF works; inner SFF also blocks |
| `sf_brane_blocking` | SF resolves non-brane searches, blocks brane references |
| `sf_non_brane_resolves` | SF resolves simple variable references normally |
| `sff_in_binary_op` | SFF wrapper stripped for arithmetic |
| `unanchored_seek` | `#-1` resolves to previous statement |
| `anchored_seek_positive_negative` | `a#1` and `a#-1` on brane |
| `seek_negative_clamping` | `#-99` clamps to first statement |
| `seek_beyond_start` | Very negative offset clamps gracefully |

---

## What Phase 7 Now Needs

SF/SFF are implemented. Phase 7 scope reduces to:
- Detachment brane (`[id]{...}`) with AB/IB
- P-brane partial application (`[+id]`)
- Re-detachment
- Forward search liberation (`[~pat]`)
- Alarm system
- Upward search (`↑`)

---

## Last Updated

**Date**: 2026-05-06
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — implementation insights from SF/SFF/Seek work.
Documents key design decisions, borrow checker patterns, scope management,
and blockers for Phase 5 (breadth-first) and Phase 7 (detachment).
