---
foop: 15
title: AB list, name resolution, search_result, and short-circuit accumulation
author: hc <hc.busy@gmail.com>
status: Brewing
type: Standards
created: 2026-05-08
phase: phase-2
supersedes: []
---

# FOOP-51: AB list, name resolution, search_result, and short-circuit accumulation

## Abstract

Introduces the **AB** (ancestral brane) — an immutable record carried on
every FIR that captures the ancestral context the FIR was born into.
Conceptually, AB is a single brane: imagine flattening a chain of
ancestor branes into one long brane, with all their statements lined up
end-to-end. Searching through AB is the same as calling
`parent.search_local(...)` followed by `parent.search_ancestral(...)`.
Implementation-wise, AB is represented as an immutable list of
`(brane, line_number)` pairs, but the conceptual model is one
flattened brane.

AB makes constanic clone an O(1)-shaped structural operation: cloning
extends AB rather than recursively rewriting state. The Search FIR's
resolved target is renamed `search_result` and is governed by a
determinism invariant: the stored result must always equal what
re-walking would produce.

A new step is introduced between CONSTANT and INDEPENDENT that releases
AB and the parent reference; INDEPENDENT becomes the fully-detached
terminal state. The driver loop is modified to allow this extra step.

This FOOP works in concert with the revised FOOP 7 (constanic clone),
which consumes AB as its sole modification mechanism.

### Terminology

- **Parent**: the FIR or brane that a given FIR is attached to. A FIR
  cannot move from its parent; the parent relationship is structural,
  not contextual. Where other documents might say "lexical parent,"
  this FOOP says simply "parent."
- **AB (ancestral brane)**: the conceptual flattened brane representing
  all ancestral context above the parent. Every FIR has an AB. A
  freshly compiled FIR has an empty AB. AB grows when a FIR is
  constanically cloned out of its original home; the clone's AB
  absorbs the original parent and AB.
- **Search through AB**: equivalent to calling
  `parent.search_local(pattern, from_line)` followed by
  `parent.search_ancestral(pattern, from_line)`. The flattening is
  conceptual; in practice we walk the AB list, but the meaning is the
  same as if AB were a single brane.

## Motivation

The current UBC reference implementation handles re-coordination via
recursive clone-and-reset (`constanic_clone` in `ubc.rs:458-493`,
`reset_searches` in `ubc.rs:259-322`). This has three problems:

1. **Unbounded clone cost.** Cloning a brane recursively rebuilds every
   descendant FIR and resets every Search to EMBRYONIC, forcing every
   name to be re-resolved in the new context. The cost is O(subtree size
   × cost-of-re-resolution).

2. **Non-monotonic state.** `reset_searches` is called routinely during
   `re_step_brane_bodies`, throwing away resolved targets and forcing
   re-walking. State transitions are not monotonic — the same FIR moves
   forward and backward through NYES states across iterations of the
   driver loop. This is hostile to a message-passing implementation
   (FOOP 41 / UBCb), which needs explicit, monotonic transitions.

3. **No structural account of "where this FIR came from."** When a brane
   is cloned into a new host, the cloned subtree's children must still
   resolve names against the original ancestral context. Today this is
   handled by re-resolving from scratch in the new host; correctness
   relies on the live parent chain happening to reach back through the
   right brane structure. There is no first-class representation of the
   ancestral context the FIR was born into.

AB solves all three. It makes constanic clone a structural extension
operation, makes state transitions monotonic (clearing AB happens only at
the explicit CONSTANT → INDEPENDENT detach step), and gives every FIR an
explicit ancestral-context record.

### Why we keep WOCONSTANIC despite AB

AB by itself is sufficient for **correctness** of cross-brane name
resolution. WOCONSTANIC is retained because it serves a different,
complementary role: **caching of resolved targets within an evaluation
pass**. Without WOCONSTANIC, every step would re-resolve every search;
WOCONSTANIC remembers the resolution and short-circuits future steps to
read directly from the stored `search_result`.

AB and WOCONSTANIC are orthogonal:
- AB makes re-coordination cheap (just append to the list).
- WOCONSTANIC makes intra-pass repetition cheap (don't re-walk).

Together, they give us correct cross-brane semantics with efficient
stepping.

## Specification

### The AB field

Every FIR carries an immutable AB:

```rust
pub struct Ab {
    /// Conceptually a single flattened brane. Implemented as an ordered
    /// list of (brane, line_number) pairs that, when walked front-to-back,
    /// represents the same statement sequence as if all listed branes
    /// were concatenated end-to-end up to their respective line numbers.
    entries: ImmutableList<(Rc<NormalBraneFir>, usize)>,
}
```

- Each entry contributes `brane`'s statements at indices strictly less
  than `line_number` to the conceptual flattened brane.
- The list is **ordered front-to-back as oldest-to-newest** ancestral
  context. Front = the original ancestral context; back = the
  most-recently-acquired host's context.
- AB is a **read-only reference** to the listed branes; it does not own
  them in the long-term sense. Branes are owned by their parents and
  the live evaluation graph; AB extends those lifetimes only as long
  as the AB-carrying FIR is itself in a non-INDEPENDENT state.
- Implementation note: AB is specified as an immutable list. The natural
  representation (a singly-linked cons list) walks back-to-front; we
  walk front-to-back. The reference implementation may use a `Vec` and
  copy on extension. This is O(|ab|) per extension and is the simple,
  correct choice; optimization is deferred.

### The coordination invariant

> A FIR evaluated at its original home does NOT carry its parent in AB.
> The parent is reached via the live `parent` pointer.

This means:

- A freshly compiled FIR has `ab = empty`.
- AB grows only via constanic clone (see FOOP 7).
- The `parent` field captures the FIR's *current* container brane.

When a FIR is cloned out of its original home and installed elsewhere,
its old parent must be materialized into AB so that searches can still
find it. This is constanic clone's only job.

### Name resolution algorithm

Each brane provides two search primitives:

```
brane.search_local(pattern, from_line) -> Option<FirRef>:
    // Search this brane's own statements at indices < from_line for
    // a match. Does not consult parent or AB.

brane.search_ancestral(pattern, from_line) -> Option<FirRef>:
    // Search this brane's AB (the flattened ancestral brane) for a
    // match. Does not consult this brane's local statements or parent.
```

Backward (unanchored) name resolution starting from FIR `F` at line `L`
in brane `B` proceeds as:

```
resolve_search(F, pattern):
    let mut cur_brane = B
    let mut cur_line = L
    loop {
        // Phase 1: search cur_brane's local statements
        if let Some(hit) = cur_brane.search_local(pattern, cur_line) {
            return Found(hit, cur_brane, cur_line)
        }

        // Phase 2: search cur_brane's AB (the flattened ancestral brane)
        if let Some(hit) = cur_brane.search_ancestral(pattern, cur_line) {
            return Found(hit, /* via AB */)
        }

        // Phase 3: ascend to cur_brane's parent
        match cur_brane.parent_with_line() {
            Some((parent, line_in_parent)) => {
                cur_brane = parent;
                cur_line = line_in_parent;
            }
            None => return Econstanic
        }
    }
```

Equivalently, since "search ancestral" walks the AB list, and the AB
list is conceptually a flattened brane, Phase 2 can be restated as:

> For each `(anc_brane, anc_line)` in AB, front-to-back: try
> `anc_brane.search_local(pattern, anc_line)`. Return the first hit.

Key properties:

- **AB is consulted between local and parent.** Not before local (a
  name in scope wins over an ancestral one); not after parent (the
  ancestral context the FIR was born into wins over a context it was
  merely cloned into — except by the recursion that reaches that
  ancestor through AB).
- **Each AB entry's line bound is honored.** A search through
  `(anc_brane, anc_line)` sees only statements with index `< anc_line`
  in `anc_brane`. This preserves the invariant that backward search sees
  only preceding lines, even across cross-brane references.
- **AB is shallow.** Only the search-result FIR (the root of a
  cloned-in subtree) carries AB. Its descendants do not. When a
  descendant performs a search, it walks its own parent chain upward;
  eventually it reaches the cloned-in root, which then contributes its
  AB to the resolution.

### The `search_result` field

The `SearchFir::target` field is **renamed to `search_result`**. This
rename clarifies semantic role and is part of FOOP 51's implementation
diff.

```rust
pub struct SearchFir {
    pub(crate) pattern: String,
    pub(crate) direction: SearchDirection,
    pub(crate) anchored: bool,
    pub(crate) anchor: Option<FirRef>,
    pub(crate) search_result: Option<FirRef>,  // was: target
    pub(crate) state: Nyes,
    pub(crate) ab: Ab,
}
```

#### The determinism invariant (mandatory)

> The result of a search is the search result. A FIR's computational
> meaning does not change.

Operationally:

- A `SearchFir` performs its search **at most once**. Once
  `search_result` is set, it is never re-resolved by walking the
  resolution algorithm again.
- `search_result` may take different *values* (`Some(fir)` becoming
  `Some(deeper_fir)` via short-circuiting) but the *identity of "the
  search result"* is preserved.
- For any `SearchFir` `S` with `search_result = Some(R)`:
  - Clearing `S.search_result = None` and `S.state = Embryonic`, then
    re-running the resolution algorithm in the same context, must
    produce `search_result = Some(R')` where `R'` and `R` denote the
    same value (same FIR identity, or — if short-circuiting has
    progressed in either path — the same final-target identity that
    each would reach).
- This equivalence is **bidirectional**: storing the result must equal
  what re-walking produces, AND re-walking must produce what the stored
  result represents.

This invariant is what justifies caching the result without re-walking:
the walk is a deterministic function of the FIR's structure and AB.

The invariant is enforced by **mandatory unit tests** (see Test Plan).

### Short-circuit accumulation

When a `SearchFir` `S₁`'s `search_result` is itself a `SearchFir`, and
that one's `search_result` is itself a `SearchFir`, etc. — forming a
chain `S₁ → S₂ → S₃ → ... → T` where `T` is the first non-search FIR —
the chain is collapsed (short-circuited) so `S₁.search_result` points
directly to a clone of `T`.

**The accumulation rule:** every hop in the chain `S₁ → S₂ → ... → T`
contributed an AB extension when its `search_result` was originally
established. Collapsing the chain MUST preserve the union of those
extensions on the installed final target.

```
short_circuit(s1):
    let mut accumulated_extensions = []
    let mut cur = s1.search_result
    while cur is SearchFir with search_result = Some(next):
        accumulated_extensions.append_all(cur.ab_extension_at_resolution)
        cur = next
    let tail = cur
    let final_ab = tail.ab.append_all(accumulated_extensions)
    s1.search_result = clone_with_ab(tail, final_ab)
```

The order of accumulation is **traversal order** (S₁'s extension first,
then S₂'s, etc.), which corresponds to oldest-context-first since each
subsequent hop is a clone-into-newer-host.

This rule is essential for correctness: searches inside the collapsed
final target must see the same names they would have seen if the chain
were walked hop-by-hop without short-circuiting.

### The CONSTANT → INDEPENDENT detach step

CONSTANT and INDEPENDENT are no longer near-synonyms. They become two
distinct states with a defined transition:

- **CONSTANT**: value computed, FIR still anchored to its evaluation
  context. AB and `parent` are populated.
- **INDEPENDENT**: value computed, FIR detached. AB is empty, `parent`
  is None. The FIR is a self-contained value.

The transition is a discrete UBC step:

```
on step(fir) where fir.state == CONSTANT:
    fir.ab = empty
    fir.parent = None
    fir.state = INDEPENDENT
    return NoOp  // detached; no further work
```

The transition is **eager-by-default**: a FIR that has just settled into
CONSTANT will detach on its next step without external prompting. The
discrete one-step-later timing preserves the option to introduce
operations that legitimately need the parent reference *after* CONSTANT
but *before* INDEPENDENT, should such operations be discovered.

NK is unaffected by this rule; NK FIRs are born without meaningful
ab/parent references and remain at NK.

WOCONSTANIC and ECONSTANIC retain their AB. Releasing AB on these states
is an optimization that requires walking descendants to confirm no live
search depends on it; it is deferred to a future FOOP unless profiling
shows it matters.

### Driver loop change

The driver loop in `run_to_completion_with_scope` (`ubc.rs:133-157`)
currently terminates when `prev_state == new_state`. This must be
adjusted to allow the CONSTANT → INDEPENDENT detach step:

```rust
// Before:
if prev_state == Nyes::Constant ||
   prev_state == Nyes::Independent ||
   prev_state == Nyes::Nk { break; }

// After:
if fir.borrow().state().is_fully_terminal() { break; }
```

Where `is_fully_terminal()` is defined as:

```rust
impl Nyes {
    pub fn is_fully_terminal(&self) -> bool {
        matches!(self, Nyes::Independent | Nyes::Nk)
    }
}
```

CONSTANT is no longer treated as a loop-terminating state; the driver
runs one more step to perform detachment. WOCONSTANIC and ECONSTANIC
continue to terminate the loop via the existing
`has_unresolved_forward_refs` check.

### WOCONSTANIC under the AB model

A `SearchFir` that has resolved to a CONSTANIC-but-not-CONSTANT target
sits in WOCONSTANIC, holding its `search_result` pointer. When the
target eventually advances (to CONSTANT, then INDEPENDENT), the search
wakes up and reads the value without re-walking the resolution path.
This is enforced by the determinism invariant.

Under the AB model:
- WOCONSTANIC pointers **survive constanic cloning** unchanged. A
  WOCONSTANIC search inside a cloned subtree keeps its `search_result`;
  the target it points at remains valid because AB makes re-resolution
  unnecessary.
- WOCONSTANIC → ECONSTANIC re-fire becomes obsolete. There is no path
  by which a stored `search_result` becomes stale; the determinism
  invariant guarantees this.

### `reset_searches` is provably redundant

The function `reset_searches` (`ubc.rs:259-322`) clears `target` on every
Search FIR before re-stepping. Under the determinism invariant, this is
*guaranteed safe* (it produces the same observable result) but
*unnecessary* (the stored result was already correct). The function
should be deleted as part of FOOP 51 implementation.

The deletion serves as a forcing function: if any test fails after
removing `reset_searches`, the failure indicates a violation of the
determinism invariant — a real bug, not a regression.

## FIR Impact

- **All FIR variants gain an `ab` field** of type `Ab` (immutable list
  of `(Rc<NormalBraneFir>, usize)` pairs). Default value is empty.
- **`SearchFir::target` is renamed to `search_result`**.
- **`Nyes::Independent` semantics shift**: previously near-synonym for
  CONSTANT, now means "detached, AB and parent cleared." The
  serialization key remains `INDEPENDENT`.
- **Serialization**: AB is part of the FIR JSON shape. Serializing a FIR
  emits its `ab` list as an array of `[brane_id, line_number]` pairs,
  where `brane_id` is a stable identifier for the referenced brane.
  Deserialization reconstructs Rc references via the brane identifier
  table. (The exact serialization scheme is an implementation detail; a
  follow-up FOOP may codify it if needed.)

### FIR construction: Builder and BuilderFrom patterns

Adding the AB field means every existing FIR construction site must be
updated. To make this maintainable — and to keep future FIR additions
ergonomic — FIR construction is reorganized around builder patterns.

**Builder** (used at compile time and in tests):

```rust
let fir = SearchFirBuilder::new()
    .pattern("foo")
    .direction(SearchDirection::Backward)
    .anchored(false)
    .build();
// search_result, state, ab default to None, EMBRYONIC, empty
```

Builders provide sensible defaults for AB (empty), state (EMBRYONIC for
NYE-eligible FIRs, NK for NkFir, etc.), and result fields (None). The
parser/compiler uses builders so the addition of AB does not force
changes throughout the compilation pipeline.

**BuilderFrom** (used during evaluation, especially constanic clone):

```rust
// Constanic clone is a structural copy with extended AB:
let clone = SearchFirBuilderFrom::new(&original)
    .extend_ab(original.parent.clone(), original.line_in_parent)
    .build();
```

`BuilderFrom` takes an existing FIR and produces a modified copy. It is
the primary mechanism for constanic clone (FOOP 7) and short-circuit
accumulation. It enforces that fields the FOOP 51 invariants depend on
(state, search_result identity) are preserved unless explicitly changed.

**Implementation note.** The exact builder API (Rust traits, derive
macros, hand-written) is an implementation detail. The requirement is
that:

1. Adding a new field to a FIR variant should not break existing
   construction sites that don't care about the new field.
2. Constanic clone should be expressible as a single builder call,
   not as a multi-line manual struct literal.
3. Tests should be able to construct FIRs in arbitrary states for
   exercising the determinism invariant.

Crates like `derive_builder` are sufficient. Hand-written builders are
acceptable if they meet the above requirements.

## UBC Step Impact

- **Search resolution** (in `step_one` for `SearchFir`) walks
  `brane.search_local(...)` → `brane.search_ancestral(...)` (AB) →
  parent. The `Scope` abstraction (`ubc.rs:60-125`) is augmented to
  consult the searching FIR's AB between local lookup and parent
  ascent.
- **CONSTANT → INDEPENDENT** is a new step rule on every FIR variant:
  if state is CONSTANT, detach and advance to INDEPENDENT.
- **Driver loop** terminates on `is_fully_terminal()` rather than on
  CONSTANT directly.
- **`re_step_brane_bodies`** (`ubc.rs:216-255`) no longer calls
  `reset_searches`. The function is removed.
- **`constanic_clone`** is rewritten per the revised FOOP 7 to perform
  AB extension only. It no longer recursively clones descendants and no
  longer dispatches on NYES state.
- **Short-circuit logic in `SearchFir::short_circuit_self`** is rewritten
  to accumulate AB across the collapsed chain and install a clone of the
  final target with the accumulated AB.

## Test Plan

### Determinism invariant — mandatory unit tests

For each representative search scenario, construct two FIR trees and
verify byte-identical output:

- **Tree A**: stepped normally to completion.
- **Tree B**: identical to Tree A, but at a chosen mid-evaluation step,
  every Search FIR has `search_result` cleared and `state` reset to
  EMBRYONIC. Then stepping resumes to completion.

Scenarios to cover:

1. Own-brane hit: search resolves to a sibling statement.
2. AB hit: search resolves through a single AB entry.
3. Multi-AB hit: search resolves through the second or third AB entry.
4. AB-line-bound respected: search would match a statement at index ≥
   `anc_line` in an AB entry; verify it is NOT matched (only earlier
   statements are visible).
5. Parent-chain hit: search resolves through live parent ascent.
6. ECONSTANIC: search exhausts all phases without a match.
7. WOCONSTANIC pointing at NYE target.
8. WOCONSTANIC pointing at CONSTANT target.
9. Short-circuited chain (S₁ → S₂ → T) — verify accumulated AB on T.
10. Nested constanic clone (a clone of a clone of a brane).

### Approval test parity

Run the existing 60+ Phase 2 approval tests with FOOP 51 implementation.
All must pass without modification. Snapshot diffs that result from the
implementation are not acceptable; if a snapshot changes, either the
snapshot was wrong or FOOP 51 has been mis-implemented.

### Regression: `reset_searches` removal

After deleting `reset_searches`, all approval tests must still pass.
This validates that the determinism invariant holds in practice.

### Driver loop: detach step

A unit test verifies that a brane with all-CONSTANT statements steps
each statement once more after reaching CONSTANT, advancing every
descendant to INDEPENDENT, before the driver loop terminates. The test
asserts: at termination, every CONSTANT-eligible descendant has
`state == INDEPENDENT`, `ab.is_empty()`, and `parent.is_none()`.

## Rejected Alternatives

### A. Drop AB on WOCONSTANIC and ECONSTANIC also

Free AB on every constanic state, not just at the CONSTANT → INDEPENDENT
detach step. **Rejected** for now: WOCONSTANIC and ECONSTANIC FIRs may
be containers (branes, operators, concatenations) whose interior
descendants still consult their AB during evaluation. Determining when
it is safe to drop their AB requires walking descendants. Deferred to a
future FOOP if profiling shows it matters.

### B. Make AB mutable

Allow AB to grow incrementally during evaluation, not just at clone
time. **Rejected**: violates the determinism invariant. If AB can change
during evaluation, the equivalence "stored search_result = re-walked
result" no longer holds, because re-walking after AB changes would see a
different ancestral context.

### C. Eager CONSTANT → INDEPENDENT (single step, not two)

Detach immediately when a FIR computes its value, in the same step.
**Rejected**: the discrete one-step-later timing preserves the option to
introduce operations that need the parent reference *after* CONSTANT but
*before* INDEPENDENT. The cost of one extra step per CONSTANT is
negligible and may be optimized later if needed.

### D. Keep `reset_searches` as a defensive no-op

Retain the function but verify (in debug builds) that calling it does
not change observable behavior. **Rejected**: the determinism invariant
makes it dead code. Carrying dead code as defensive scaffolding bloats
the codebase and obscures intent. If the invariant is wrong, deleting
`reset_searches` will surface that fact via test failures — which is the
correct response.

### E. Use `Weak` references for AB

Hold ancestral branes via `Weak` rather than `Rc`. **Rejected**: the
CONSTANT → INDEPENDENT detach step gives us deterministic release timing
without the upgrade-failure complexity of `Weak`. AB references are
non-owning *in intent*; the discipline of detaching at CONSTANT →
INDEPENDENT enforces this in practice.

### F. Eliminate WOCONSTANIC entirely

With AB providing correctness, one might argue WOCONSTANIC is redundant.
**Rejected**: WOCONSTANIC remains essential for efficiency. It caches
resolved targets within an evaluation pass, avoiding re-walking AB and
parent chains on every step. AB is for correctness across re-coordination;
WOCONSTANIC is for efficiency within an evaluation pass. They are
orthogonal.

## Open Questions

- **Anchored search interaction (FOOP 10).** Anchored searches consult
  an anchor FIR's interior. If that anchor was constanically cloned, its
  AB needs to be considered during the anchored walk. Does FOOP 10 need
  an addendum, or is the existing FOOP 10 already compatible? Defer to
  FOOP 10 review.
- **Brane identifier scheme for AB serialization.** The JSON contract for
  AB entries needs a stable brane identifier. Likely solution: a per-FIR
  UUID or LUID, consistent with FOOP 41's LUID requirement for UBCb.
  Defer to a follow-up FOOP if needed.
- **Performance: AB list copy on extension.** O(|ab|) per extension is
  acceptable for current test sizes but may matter at scale. A
  structurally-shared list (with front-to-back walk via reverse
  iteration) is a future optimization.
- **Future: WOCONSTANIC AB release.** Profiling may justify releasing
  AB on WOCONSTANIC/ECONSTANIC under specific conditions (no live
  descendant searches). Defer until measured.

## Related issues NOT addressed by this FOOP

This FOOP focuses on the constanic-clone unboundedness, the
non-monotonicity of `reset_searches`, and the absence of a first-class
ancestral-context record. Several other state-machine concerns
identified during the FOOP 41 (UBCb) analysis remain open:

1. **PREMBRYONIC is unused; no explicit "BRANING-complete/awaiting-children"
   sub-state.** The Nyes enum has eight variants but the code only
   produces seven of them. PREMBRYONIC is never emitted. Worth a small
   cleanup FOOP. Not addressed here.

2. **EMBRYONIC/BRANING are conflated for non-brane FIRs.** Operators
   and Searches have no real BRANING phase; they go EMBRYONIC →
   {WOCONSTANIC | CONSTANT}. UBCb's stage-fairness model assumes
   universal stages. This is a UBCb design question; deferred to FOOP 41
   implementation.

3. **Scope is constructed every step; not a first-class addressable
   thing.** `Scope` is rebuilt for every `re_step_brane_bodies` call.
   UBCb needs durable brane identity for `FulfillSearch` messages.
   This FOOP gives branes the property of "being addressable through
   AB," which is a step toward UBCb's needs but does not introduce
   LUIDs or persistent brane identity. Deferred to FOOP 41
   implementation.

4. **Concatenation merging timing.** FOOP 5 (compile-time vs
   evaluation-time work), the current code, and FOOP 41's CP-3 expect
   different things. Orthogonal to AB; defer to Phase 3 review.

These are noted here so future FOOPs can reference the residual list.

## References

- FOOP 7 (revised 2026-05-08): Constanic Clone — consumes AB as its sole
  modification mechanism.
- FOOP 6: Phase 2 evaluator is depth-first sequential. Driver loop
  change in this FOOP is compatible with FOOP 6's depth-first contract.
- FOOP 10 (Brewing): Anchored search through constanic anchors. May need
  addendum for AB interaction.
- FOOP 11 (Brewing): Search stops at NK. Unchanged by FOOP 51.
- FOOP 41 (Draft): UBCb message-passing variant. AB monotonicity
  enables UBCb's message protocol; this FOOP unblocks several issues
  raised in FOOP 41's research log.
- Code: `foolish/foolish-core/src/fir.rs` — FIR struct definitions,
  Nyes enum, SearchFir.
- Code: `foolish/foolish-core/src/ubc.rs` — Scope, run_to_completion,
  re_step_brane_bodies, constanic_clone, reset_searches.
- `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` — original UBC
  ancestral context semantics.
- `docs/vintage_legacy/ECOSYSTEM.md` — original UBC2 AB/IB semantics.

## Last Updated

**Date**: 2026-05-08
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 xHigh effort
**Changes**: Initial draft.
