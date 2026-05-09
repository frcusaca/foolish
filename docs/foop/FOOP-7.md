---
foop: 7
title: Constanic Clone — recoordination contract (revised to consume AB)
author: hc <hc.busy@gmail.com>
status: Brewing
type: Standards
created: 2026-05-01
phase: phase-2
supersedes: []
---

# FOOP-7: Constanic Clone — recoordination contract (revised to consume AB)

## Abstract

Defines the **calling contract** and **algorithm** for two clone
operations on FIRs:

- **`constanic_clone(source)`** — the operation invoked at every
  search-result attachment point. The clone preserves AB unchanged,
  resets WOCONSTANIC/NOTFOUNDIC state to BRANING, and shares
  CONSTANT/INDEPENDENT/NK by reference. The caller subsequently
  invokes `.setParent(new_parent).build()` to coordinate the clone
  into its new host. The new parent provides the new ancestral
  context via its own parent chain.

- **`preconstanic_clone(source)`** — a separate operation reserved
  for a future special case. Operates on pre-constanic FIRs only.
  Extends AB by `(source.parent, source.line_in_parent)` and runs
  FOOP-51 line-aware compression. Preserves NYES state unchanged.
  Implemented and tested in the FVM but not currently invoked.

The contract:

> **Every search result is `constanic_clone`'d before being assigned
> to the Search FIR's `search_result` field. The returned builder is
> chained `.setParent(host).build()` to coordinate the clone into
> its new host. The raw search result is NEVER assigned directly.**

`constanic_clone` requires its source to be in a constanic state
(WOCONSTANIC, NOTFOUNDIC, CONSTANT, INDEPENDENT, NK). Pre-constanic
sources panic. `preconstanic_clone`, conversely, requires a
pre-constanic source.

This FOOP was revised on 2026-05-08 (initial AB model) and again on
2026-05-09 (split into constanic_clone vs preconstanic_clone;
constanic_clone no longer extends AB; recursive descent introduced).
See "Revision history" for details.

## Motivation

Foolish's recoordination semantics — a FIR resolves names against the
context it was born into, not necessarily the context it currently
inhabits — require that search results be **cloned** into the searcher's
context rather than shared by reference. If shared, a future context
change to the original would affect every searcher pointing at it.

Under FOOP 51's AB model, "the context it was born into" is captured
explicitly in the FIR's `ab` field. Constanic clone's job is to extend
AB so that the clone, in its new home, can still resolve names against
the original ancestral context.

The previous algorithm — recursive descent with state reset and per-NYES
dispatch — became unnecessary once AB existed. AB makes re-coordination
a structural operation: extend the list, install the clone. No
recursion, no state reset, no dispatch. See FOOP 51's motivation
section for the three problems this revision addresses
(unbounded clone cost, non-monotonic state, no first-class ancestral
context).

## Specification

### The contract

> **At every site where a search resolves to a result, `constanic_clone`
> is called on that result. The output is assigned to the Search FIR's
> `search_result` field. The raw search result is NEVER assigned
> directly.**

```rust
// At the search-step site:
let raw_result: Option<FirRef> = scope.search(pattern);
match raw_result {
    None => self.state = Nyes::Notfoundic,
    Some(found) => {
        // CONTRACT: constanic_clone is called on every search result.
        // Never assign a raw search result directly to search_result.
        let cloned = constanic_clone(&found);
        self.search_result = Some(cloned);
        // Subsequent UBC steps propagate state from search_result into self.
    }
}
```

### Two clone operations

UBCb defines **two distinct clone operations** for distinct cases:

| Operation | Source state | Extends AB? | Resets state? | Used for |
|---|---|---|---|---|
| `constanic_clone(source)` | constanic only | **No** | WOCONSTANIC/NOTFOUNDIC → BRANING | Search results; brane recoordination after BRANING |
| `preconstanic_clone(source)` | preconstanic only | **Yes** (with compression) | No (preserves NYES) | Reserved future feature: duplicating in-progress computation |

The current FVM uses only `constanic_clone`. `preconstanic_clone` is
implemented and tested as a VM feature for a special case to be
introduced later (see "preconstanic_clone" section below).

### `constanic_clone` algorithm

```rust
/// Constanic clone: produce a new FIR derived from `source` with state
/// reset to BRANING for WOCONSTANIC/NOTFOUNDIC, or shared-by-reference
/// for CONSTANT/INDEPENDENT/NK. Recurses through the source's children,
/// rewriting their parent pointers to the clone and resetting
/// WOCONSTANIC/NOTFOUNDIC descendants to BRANING.
///
/// AB is NOT extended. The clone's AB is identical to the source's AB.
/// Re-coordination is achieved by the caller subsequently invoking
/// `setParent(new_parent)` on the returned builder before installing
/// the clone in the new host.
///
/// PRECONDITION: `source.state` MUST be a constanic state
/// (WOCONSTANIC, NOTFOUNDIC, CONSTANT, INDEPENDENT, NK).
/// Pre-constanic sources cause this function to **panic** (not just
/// debug-assert) — calling it on a NYE-state FIR is a programming
/// error that must be caught immediately.
pub fn constanic_clone(source: &FirRef) -> FirBuilder {
    if !source.borrow().state().is_constanic() {
        panic!("constanic_clone called on non-constanic source: {:?}",
               source.borrow().state());
    }

    // Terminal-immutable variants: share by reference. AB is irrelevant
    // (the value is context-immune); parent will be set by the caller's
    // .setParent(...).build() pattern.
    match source.borrow().state() {
        Nyes::Constant | Nyes::Independent | Nyes::Nk =>
            return FirBuilder::wrapping(source),
        _ => {}
    }

    // Source is WOCONSTANIC or NOTFOUNDIC. Build the cloned root with
    // AB UNCHANGED and state reset to BRANING. The clone's
    // re-coordination work happens at BRANING via parent.search() —
    // the new parent (set by caller) provides the new ancestral
    // context. The old AB stays attached but is essentially historical
    // (a constanic FIR's old context already gave what it could).
    let mut builder = BuilderFrom::new(source)
        .with_state(Nyes::Braning);
        // AB is preserved as-is; not extended.

    // Recurse into children: rewrite parent pointers to `clone`,
    // reset constanic-pending children to BRANING. Children's AB is
    // also unchanged.
    recursively_clone_children(&mut builder);

    builder
}

fn recursively_clone_children(parent_builder: &mut FirBuilder) {
    for child_slot in parent_builder.children_mut() {
        let child_state = child_slot.borrow().state();
        match child_state {
            // Context-immune children: share by reference.
            Nyes::Constant | Nyes::Independent | Nyes::Nk => continue,
            // Constanic-pending children: clone with rewritten parent
            // pointer, reset state to BRANING, recurse.
            Nyes::Woconstanic | Nyes::Notfoundic => {
                let cloned_child = BuilderFrom::new(child_slot)
                    .with_parent(parent_builder.as_parent_ref())
                    .with_state(Nyes::Braning)
                    .build();
                *child_slot = cloned_child;
                recursively_clone_children_at(child_slot);
            }
            // NYE children of a constanic parent should not occur in
            // practice (a constanic parent's children are all
            // constanic, by FOOP-61's brane-state-computation rules).
            _ => panic!("NYE child in constanic parent: {:?}", child_state),
        }
    }
}
```

**Caller usage pattern.** `constanic_clone` returns a builder, not a
finished FIR. The caller is expected to chain `.setParent(new_parent)`
before `.build()` to coordinate the clone into its new host:

```rust
// Search resolution:
let raw_target = scope.search(pattern)?;
let installed = constanic_clone(&raw_target)
    .setParent(self_as_parent_ref())
    .build();
self.search_result = Some(installed);
```

This pattern makes the re-coordination step explicit and ensures the
clone's parent pointer is set correctly before any stepping observes
it.

**Why AB is NOT extended.** A constanic FIR is one that has already
become "constant in context" — its meaning was determined relative to
its old AB and parent chain. The old context has nothing left to offer
(otherwise the FIR would still be NYE). What the clone needs is a
**new parent**, which provides a new ancestral context via the new
parent's own AB chain. The clone's `parent.search()` walks upward
through the new parent and finds new content. The old AB is preserved
but unused for resolution (it is historical).

For NOTFOUNDIC specifically: the clone's BRANING work will re-walk
its searches. The walk goes own-statements → AB (old, exhausted) →
parent (NEW). The new parent provides the rescue path; the old AB
contributes nothing further. Correct by construction.

**Why reset to BRANING and not EMBRYONIC.** BRANING is the universal
active-work state in UBCb. The clone's local-IB work is preserved
(its EMBRYONIC-stage statement-array building was done before it
became constanic; children are reference-shared and retain their
EMBRYONIC-stage work). Only cross-brane work (BRANING) needs to redo
against the new parent chain.

**Recursion stops at CONSTANT/INDEPENDENT/NK.** These states represent
context-immune values; AB extension or parent change cannot affect
them. They are shared by reference; recursion does not enter them.

### `preconstanic_clone` algorithm

```rust
/// Pre-constanic clone: produce a new FIR derived from `source` with
/// AB extended by (source.parent, source.line_in_parent), then
/// compressed per FOOP-51's line-aware dedup rule. State is preserved
/// unchanged. Children are recursively cloned with their parent
/// pointers rewritten.
///
/// This operation duplicates in-progress computation. The clone has
/// the source's old context (still in AB) AND will gain new context
/// when its parent is set. Used for a special case to be introduced
/// later in UBCb's evolution.
///
/// PRECONDITION: `source.state` MUST be a pre-constanic state
/// (PREMBRYONIC, EMBRYONIC, BRANING, WOBRANING). Constanic sources
/// cause this function to **panic** — use `constanic_clone` instead.
pub fn preconstanic_clone(source: &FirRef) -> FirBuilder {
    if !source.borrow().state().is_preconstanic() {
        panic!("preconstanic_clone called on non-preconstanic source: {:?}",
               source.borrow().state());
    }

    // Extend AB by source's current (parent, line) and compress.
    let parent = source.borrow().parent();
    let line = source.borrow().line_in_parent();
    let new_ab = source.borrow().ab()
        .append((parent, line))
        .compress();  // Per FOOP-51 line-aware dedup.

    // State preserved unchanged.
    let mut builder = BuilderFrom::new(source)
        .with_ab(new_ab);
        // State is NOT changed; the clone is in the same NYE phase
        // as the source.

    // Recurse into children: same pattern as constanic_clone, but
    // for preconstanic children.
    recursively_preconstanic_children(&mut builder);

    builder
}
```

**Why state is preserved.** Unlike constanic FIRs (whose old context
is exhausted), preconstanic FIRs may still have meaningful work to do
in their old context. Preserving state lets them resume that work
while *also* gaining access to the new context via extended AB.
This is "duplicating computation": the clone explores both old and
new contexts.

**Reserved feature.** `preconstanic_clone` is implemented and tested
in the FVM but is not invoked by the current FOOP set. A future FOOP
will introduce the special case that uses it.

**Implementation note.** `is_preconstanic()` is the complement of
`is_constanic()`:

```rust
impl Nyes {
    pub fn is_preconstanic(&self) -> bool {
        matches!(self, Nyes::Prembrionic | Nyes::Embryonic
                     | Nyes::Braning    | Nyes::Wobraning)
    }
}
```

### The `BuilderFrom` mechanism

FIR construction uses the Builder / BuilderFrom patterns introduced in
FOOP 51. `BuilderFrom::new(source).with_ab(new_ab).build()` produces a
shallow structural copy of `source` with the specified field overridden.
This is the canonical mechanism for constanic clone and for short-circuit
accumulation (also defined in FOOP 51).

### Per-state notes

`constanic_clone` is **only callable on FIRs in constanic states**
(WOCONSTANIC, NOTFOUNDIC, CONSTANT, INDEPENDENT, NK). Calling it on a
pre-constanic FIR raises a panic — those FIRs need `preconstanic_clone`
instead.

| `source.state` | Clone's state | AB | Behavior |
|---|---|---|---|
| PREMBRYONIC, EMBRYONIC, BRANING, WOBRANING | **panic** | — | Precondition violated. Use `preconstanic_clone`. |
| WOCONSTANIC | **reset to BRANING** | unchanged | Clone with state reset to BRANING; AB preserved as-is. The clone's BRANING work re-walks against new parent (set by caller). Children are recursively cloned: WOCONSTANIC/NOTFOUNDIC descendants get rewritten parent pointers and reset to BRANING; CONSTANT/INDEPENDENT/NK descendants are shared by reference. |
| NOTFOUNDIC | **reset to BRANING** | unchanged | Same as WOCONSTANIC. The clone's BRANING work re-walks the search algorithm; the rescue path is the new parent chain (not the unchanged AB, which already failed). |
| CONSTANT | shared by reference | n/a | The value is context-immune. The builder wraps the source; caller's `.setParent(...)` does not actually mutate (or, the caller may reuse without parent-setting). |
| INDEPENDENT | shared by reference | n/a | Already detached. AB is empty. |
| NK | shared by reference | n/a | Terminal error. Irrecoverable. |

**Why reset to BRANING (not EMBRYONIC or preserved).** Under FOOP-61's
universal contract, BRANING is the active-work state for cross-brane
resolution. Resetting a constanic-pending FIR to BRANING tells it "do
your cross-brane work again, against the new parent's context."
EMBRYONIC's local-IB work is preserved (children retain their
EMBRYONIC-stage statement-array building); only BRANING needs redoing.
For containers this means re-stepping children; for searches this
means re-walking AB and parent chain.

**Why AB is NOT extended for constanic clones.** A constanic FIR has
already taken its place in the value lattice — its meaning was
determined relative to its old context. The old AB exhausted whatever
it could contribute. The rescue path for a re-coordinated clone is
the **new parent**, set by the caller via the builder's
`.setParent(...)` step. The new parent's own AB chain provides the
new ancestral context; the clone's BRANING work walks up through it.

The clone's old AB stays attached but is essentially historical
(unused for resolution). Deleting it would lose forward-compatibility
with potential future inspection; keeping it is harmless because the
search walk's first AB-walking phase will simply fail (as it did
before) and proceed to the new parent.

### Caller invariant

`constanic_clone` requires its source to be in a constanic state
(WOCONSTANIC, NOTFOUNDIC, CONSTANT, INDEPENDENT, NK). Callers must
ensure this precondition; violations panic.

In practice the precondition is automatic:

- Searches deliver their results only after stepping to a constanic
  state. `search_result` always points at a constanic FIR.
- Branes only constanic-clone children at end-of-BRANING when child
  states are already collected as constanic.
- Concatenation merges only after all elements are constanic.

A caller that violates the precondition has a bug elsewhere. The
panic surfaces it immediately rather than allowing silent corruption.

### Caller usage pattern: setParent then build

`constanic_clone` returns a **builder**, not a finished FIR. This
makes re-coordination explicit:

```rust
let raw_target = scope.search(pattern)?;
let installed = constanic_clone(&raw_target)
    .setParent(self_as_parent_ref())
    .build();
self.search_result = Some(installed);
```

The caller MUST chain `.setParent(...)` (or another coordinating
operation) before `.build()`. This prevents installing a clone with
a stale or incorrect parent pointer.

Implementations may verify in debug builds that `.build()` is not
called without an intervening parent-setting operation.

## FIR Impact

See FOOP 51 for the AB field definition and the BuilderFrom mechanism.

`constanic_clone` consumes AB but does not introduce additional FIR
schema changes beyond what FOOP 51 specifies.

## UBC Step Impact

The function is invoked at every site where a search resolves to a
result (in `SearchFir::step_one`). It does NOT itself perform any
stepping — it produces a fresh structural FIR ready for the UBC driver
to step. The UBC driver continues to advance the cloned FIR through its
NYES states using the new AB.

The previous version's interaction with `re_step_brane_bodies` and
`reset_searches` is removed. FOOP 51 deletes `reset_searches` entirely;
`re_step_brane_bodies` no longer needs to reset clones because clones
arrive with valid state.

## Test Plan

### Constanic clone on every NYES state

Unit tests construct a FIR at each NYES state (PREMBRYONIC through NK)
and invoke `constanic_clone`. Assert:

- Returned FIR is structurally a copy of the source.
- Returned FIR's AB is `source.ab.append((source.parent, source.line))`.
- Returned FIR's state equals `source.state`.
- Returned FIR's children are reference-shared with the source (same
  Rc, no recursive copy).

### Approval test parity

The 60+ Phase 2 approval tests exercise constanic clone via search
resolution and concatenation. All must pass without modification.

### Determinism interaction

In conjunction with FOOP 51's determinism tests: a search that has
resolved (with `search_result = Some(cloned)`) must produce identical
output whether stepped normally or with `search_result` cleared and
re-resolved.

### Mid-evaluation clone

A test verifies that calling `constanic_clone` on a NYE-state source
(PREMBRYONIC, EMBRYONIC, BRANING) triggers the precondition assertion.
This is the caller-bug case; a test that disables the assertion is
useful only for debugging.

### NOTFOUNDIC reset behavior

A specific test constructs a Search FIR that exhausted in its
current context (state = NOTFOUNDIC, search_result = None) and
clones it into a host whose **parent chain** contains a brane that
defines the searched name. The full pattern is:

```rust
let clone = constanic_clone(&notfoundic_search)
    .setParent(new_host_parent_ref)
    .build();
```

The clone must:

1. Have its state reset from NOTFOUNDIC to BRANING.
2. Have its AB unchanged (NOT extended).
3. Have its parent set to `new_host_parent_ref`.
4. On its next step, re-walk the cross-brane resolution algorithm.
5. Resolve to the name in the new parent chain (the old AB still
   fails because nothing in AB changed; the new parent provides the
   match).
6. Reach CONSTANT (or appropriate constanic state).

A complementary test clones a NOTFOUNDIC into a host whose parent
chain still does not define the name. The clone should re-walk,
re-exhaust, and return to NOTFOUNDIC.

### preconstanic_clone behavior

Tests for `preconstanic_clone`:

1. **Precondition enforcement.** Calling `preconstanic_clone` on a
   constanic source (any of WOCONSTANIC, NOTFOUNDIC, CONSTANT,
   INDEPENDENT, NK) panics.
2. **AB extension and compression.** `preconstanic_clone(source)`
   produces a clone whose `ab` is `source.ab.append((source.parent,
   source.line)).compress()`. The compression follows FOOP-51's
   line-aware dedup rule.
3. **State preservation.** The clone's state matches the source's
   state (PREMBRYONIC stays PREMBRYONIC, EMBRYONIC stays EMBRYONIC,
   etc.).
4. **Recursive descent.** Children of the source that are pre-constanic
   are recursively cloned with parent pointers rewritten and AB
   extension applied at each level (TBD: confirm whether children's
   AB also extends, or only the cloned root's). State preserved on
   each recursed child.
5. **Future feature flag.** `preconstanic_clone` is currently not
   invoked from any FOOP-defined operation. A test confirms it is
   exposed as a public API on the FVM but not used by current
   stepping.

## Rejected Alternatives

### A. Have constanic_clone extend AB

Earlier drafts of this FOOP had `constanic_clone` extend the source's
AB by `(source.parent, source.line_in_parent)` and run compression.
**Rejected**: a constanic FIR's old AB is exhausted — its old context
contributed everything it could, and the FIR became "constant in
context" relative to that. The rescue path for a re-coordinated clone
is the **new parent**, set by the caller's `.setParent(...)` chain.
The new parent provides new ancestral context via its own AB chain.
Extending the clone's own AB is unnecessary work and introduces
spurious entries that resolve nothing.

The AB-extension semantics are preserved in the separate
`preconstanic_clone` operation, which has a different precondition
(pre-constanic source) and a different use case (duplicating
in-progress computation).

### B. Always share by reference; never clone

Don't clone search results at all. **Rejected**: violates Foolish
recoordination semantics — each searcher must see the result in its
own context. Sharing only works for context-immune values
(INDEPENDENT, NK), which is the optimization noted in the algorithm.

### C. Recursively extend AB on every descendant

Walk the source subtree and extend each descendant's AB at clone time.
**Rejected**: unnecessary. Descendants reach the new AB by walking up
to the cloned root during their searches. Recursive extension would
duplicate ancestral context information across every descendant,
violating the "AB is shallow" property of FOOP 51.

### D. Clone only at constanic terminal states

Require callers to step the source to CONSTANT/WOCONSTANIC/NOTFOUNDIC
before cloning. **Rejected**: the AB model makes mid-evaluation cloning
safe, and this restriction would force the depth-first driver to spin
to completion at every search site, defeating any future
breadth-first optimization.

## Open Questions

- **Performance: skip clone for context-immune FIRs.** The
  algorithm's per-state notes mention that CONSTANT/INDEPENDENT/NK
  could be returned unchanged. The exact set of "skip-clone-safe"
  variants depends on which FIRs ever consult AB. This is an
  implementation optimization, not a semantic question.
- **Scope ownership for UBCb.** The contract calls
  `scope.search(pattern)` to find the raw result. UBCb (FOOP 41) needs
  this scope walk to be addressable via messages. Deferred to FOOP 41
  follow-up.

## Revision history

**2026-05-09 — Major revision (Brewing)**: Split the operation into
`constanic_clone` (constanic sources) and `preconstanic_clone`
(pre-constanic sources). `constanic_clone` no longer extends AB —
re-coordination is achieved by the caller's `.setParent(...).build()`
chain on the returned builder; the new parent's chain provides the
new ancestral context. WOCONSTANIC and NOTFOUNDIC sources are reset
to BRANING. Added recursive descent: children get parent pointers
rewritten and WOCONSTANIC/NOTFOUNDIC descendants reset to BRANING.
Pre-constanic sources panic (use `preconstanic_clone` instead).
`preconstanic_clone` extends AB with FOOP-51 line-aware compression,
preserves NYES state, and is reserved for a future special case.

**2026-05-08 — Brewing**: Replaced the original recursive
clone-and-reset algorithm with AB extension on a single clone-root.
Removed per-NYES-state dispatch. Removed the caller invariant
requiring constanic terminal state. Added BuilderFrom mechanism.
This revision was superseded by the 2026-05-09 split-operation model.

**2026-05-01 — Initial draft (Brewing)**: Original calling contract
with recursive algorithm dispatching on NYES state. See git history
for the prior text.

## References

- FOOP 51: AB list, name resolution, search_result, short-circuit
  accumulation. The AB model that this FOOP consumes.
- FOOP 6: Phase 2 evaluator is depth-first sequential.
- FOOP 8 (planned): FIR mutability and parent-pointer representation.
- FOOP 41: UBCb message-passing variant. Constanic clone's
  AB-extension form is friendly to UBCb's message protocol.
- Code: `foolish/foolish-core/src/ubc.rs:458-493` — current
  `constanic_clone` implementation, to be replaced.
- `docs/vintage_legacy/d0_5_brane_recoordination.md` — original UBC2
  recoordination design (now superseded by FOOP 51's AB model).

## Last Updated

**Date**: 2026-05-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 xHigh effort
**Changes**: Split into two clone operations:
`constanic_clone` (constanic sources, no AB extension, returns
builder for caller's `.setParent(...).build()` pattern) and
`preconstanic_clone` (pre-constanic sources, extends AB with FOOP-51
compression, preserves NYES state — reserved for a future special
case). `constanic_clone` resets WOCONSTANIC/NOTFOUNDIC to BRANING
and recurses into children rewriting parent pointers; CONSTANT/
INDEPENDENT/NK shared by reference. Pre-constanic sources to
`constanic_clone` panic.

**Date**: 2026-05-08
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 xHigh effort
**Changes**: Major revision — replaced recursive clone-and-reset with
AB extension per FOOP 51. Removed per-NYES dispatch. Added BuilderFrom
construction pattern. Updated motivation, algorithm, FIR impact, UBC
step impact, test plan, and rejected alternatives sections.
(Note: The 2026-05-09 split superseded the AB-extension model from
this revision; AB extension moved to `preconstanic_clone`.)

**Date**: 2026-05-01
**Updated By**: hc
**Changes**: Initial draft.
