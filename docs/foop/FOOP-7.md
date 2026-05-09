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

Defines the **calling contract** and **algorithm** for `constanic_clone(R)` —
the function invoked at every search-result attachment point. The contract
is:

> **Every search result is `constanic_clone`'d before being assigned to
> the Search FIR's `search_result` field. The clone is a shallow
> structural copy of the original FIR with its AB extended by the
> original's parent.**

The algorithm is **AB-extension only**. Constanic clone does not
recursively descend into children, does not reset state, and does not
dispatch on NYES state. The original FIR's children are shared by
reference; only the cloned root carries an extended AB. This is
correct because AB is consulted by descendants only when their search
walks up to the cloned root (which then provides AB-aware resolution).

This FOOP was revised on 2026-05-08. The previous version described a
recursive clone-and-reset algorithm dispatching on NYES state. That
algorithm is superseded by the AB model defined in FOOP 51. See
"Revision history" below for details.

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
    None => self.state = Nyes::Econstanic,
    Some(found) => {
        // CONTRACT: constanic_clone is called on every search result.
        // Never assign a raw search result directly to search_result.
        let cloned = constanic_clone(&found);
        self.search_result = Some(cloned);
        // Subsequent UBC steps propagate state from search_result into self.
    }
}
```

### The algorithm

```rust
/// Constanic clone: produce a new FIR identical to `source` except that
/// its AB has been extended by `source`'s parent (and the line in that
/// parent at which `source` sits).
///
/// Children of `source` are shared by reference. Only the root of the
/// cloned subtree gains an extended AB; descendants reach the new AB by
/// walking up to the cloned root via their parent pointers during their
/// own searches.
///
/// Works on any NYES state. No state is reset; no recursion; no
/// per-state dispatch.
pub fn constanic_clone(source: &FirRef) -> FirRef {
    let parent = source.borrow().parent();
    let line = source.borrow().line_in_parent();

    let new_ab = source.borrow().ab().append((parent, line));

    BuilderFrom::new(source)
        .with_ab(new_ab)
        .build()
}
```

The implementation may special-case FIRs that perform no searches
(`ConstantInt`, `Independent` literals, `Nk`) by returning the source
unchanged — the AB extension on these is observably a no-op since they
never consult AB. This is an optimization, not a semantic distinction.

### The `BuilderFrom` mechanism

FIR construction uses the Builder / BuilderFrom patterns introduced in
FOOP 51. `BuilderFrom::new(source).with_ab(new_ab).build()` produces a
shallow structural copy of `source` with the specified field overridden.
This is the canonical mechanism for constanic clone and for short-circuit
accumulation (also defined in FOOP 51).

### Per-state notes

| `source.state` | Behavior under revised algorithm |
|---|---|
| PREMBRYONIC, EMBRYONIC, BRANING | Clone with extended AB. Subsequent stepping resolves searches against the new AB. |
| ECONSTANIC | Clone with extended AB. Subsequent stepping may now find what it previously couldn't, because the new AB provides additional context. |
| WOCONSTANIC | Clone with extended AB. The cloned `search_result` pointers remain valid (FOOP 51 determinism invariant). |
| CONSTANT | Clone with extended AB. The clone's value is identical; AB extension is observably a no-op for terminal values. May be optimized to share by reference. |
| INDEPENDENT | Already detached. AB is empty; parent is None. Sharing by reference is correct. |
| NK | Terminal error. Sharing by reference is correct. |

The previous version of this FOOP required dispatching on NYES state
(reset ECONSTANIC to EMBRYONIC, reset WOCONSTANIC to BRANING, etc.).
Under FOOP 51's AB model, this is unnecessary and incorrect — state
resets violate the determinism invariant.

### Caller invariant

In Phase 2 (depth-first), callers MAY invoke `constanic_clone` on a
source at any NYES state. The clone's stepping picks up where the
original left off, advancing through its remaining states using the new
AB.

The previous caller invariant ("step the source to a constanic terminal
state before invoking `constanic_clone`") is **removed**. AB makes
mid-evaluation cloning safe.

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

A test specifically exercises constanic clone on an EMBRYONIC or
BRANING source, verifying that the clone steps to completion correctly
in its new home using the extended AB.

## Rejected Alternatives

### A. Keep the recursive clone-and-reset algorithm (the previous version)

Retain dispatch on NYES state; reset ECONSTANIC to EMBRYONIC, etc.
**Rejected**: per FOOP 51, this violates the determinism invariant and
is unnecessary now that AB carries ancestral context structurally.

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

Require callers to step the source to CONSTANT/WOCONSTANIC/ECONSTANIC
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

**2026-05-08 — Major revision (Brewing)**: Replaced the recursive
clone-and-reset algorithm with AB extension. Removed per-NYES-state
dispatch. Removed the caller invariant requiring constanic terminal
state. Added BuilderFrom mechanism. Reasoning: FOOP 51 introduces the
AB model, which makes recoordination a structural operation. The
previous algorithm's recursive descent and state reset are unnecessary
under AB and violate the determinism invariant.

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

**Date**: 2026-05-08
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 xHigh effort
**Changes**: Major revision — replaced recursive clone-and-reset with
AB extension per FOOP 51. Removed per-NYES dispatch. Added BuilderFrom
construction pattern. Updated motivation, algorithm, FIR impact, UBC
step impact, test plan, and rejected alternatives sections.

**Date**: 2026-05-01
**Updated By**: hc
**Changes**: Initial draft.
