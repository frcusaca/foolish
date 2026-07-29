---
foop: 50
title: fir module decomposition — fir_base, fir_search_base, and one file per FIR kind
author: Atlas <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-07-14
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-05: fir module decomposition — fir_base, fir_search_base, and one file per FIR kind

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

> **Roadmap Track 1 — immediately after Track 0 (FOOP-64), before Tracks 2/3 fork.** This is
> the resolution of roadmap problem P4: `fir_kinds.rs` (~6,400 lines) and `fir_trait.rs`
> (~1,100 lines) are edited by both the search-family track and the state-machine track;
> decomposing them first lets those tracks run in parallel worktrees without merge collisions.

## Abstract

A **mechanical, zero-behavior-change** decomposition of `foolish-ubca`'s two mega-files into
standard, idiomatic Rust module organization following the project's OOP discipline: shared
default behavior in base modules ("base class" files), each FIR kind in its own file under a
`firs/` directory, and the search engine in its own base module. Snapshot-invisible by
definition: every output byte identical before and after.

## Motivation

Two tracks of the 2026-07-14 roadmap edit the same two files. Beyond the merge-conflict cost,
the current layout violates the project's own organization preference (one responsibility per
module, `rust_instructions.md` §2e): `fir_kinds.rs` holds the search engine, ten FIR kinds, the
shared child-classifier, and their tests; `fir_trait.rs` holds the trait, the reference alias,
two extension traits, the stepping driver, and tests. Nobody can review a kind in isolation.

## Specification

Target layout (path-based modules, **no `mod.rs`**, per `rust_instructions.md`):

```
foolish-ubca/src/
├── fir_base.rs           # trait Fir (default method bodies = the "base class"), FirKind,
│                         #   StepReport, UbcError, Scope — the contract every kind implements
├── fir_ref.rs            # FirRef alias + FirRefExt / FirRefNavExt extension traits +
│                         #   step_inner (the stepping driver) + borrow-discipline docs
├── fir_search_base.rs    # the one-engine search: contextful_search (ContextfulSearch scan,
│                         #   SearchPredicate, ScanCtx, BraneNavigator/CandidateNavigator) +
│                         #   _decide_nyes_due_to_children (shared classifier)
├── firs.rs               # `pub(crate) mod` declarations + curated re-exports for the kinds
├── firs/
│   ├── brane_fir.rs      # BraneFir
│   ├── concat_brane.rs   # ConcatBrane + its internal storage brane
│   ├── search_fir.rs     # SearchFir (name/value/name-value, anchored/unanchored/contexted)
│   ├── index_fir.rs      # IndexFir (+ seeks)
│   ├── head_tail_fir.rs  # head/tail kind(s) as currently structured
│   ├── operator_fir.rs   # OperatorFir (incl. its NK propagation — untouched)
│   ├── fool_ref_fir.rs   # FoolRefFir
│   ├── nk_fir.rs         # NkFir
│   ├── int_fir.rs        # IndepIntFir / integer kinds
│   └── sf_fir.rs         # SF/SFF marker kinds
├── proto_brane.rs        # unchanged
├── nyes_ext.rs           # unchanged
├── compiler.rs           # unchanged (imports updated)
├── evaluator.rs          # unchanged (imports updated)
└── lib.rs                # curated pub use — the crate's public surface is UNCHANGED
```

File list under `firs/` is indicative — the executor enumerates the actual kinds from `FirKind`
and keeps one kind (plus its private helpers and its `#[cfg(test)]` tests, including the
mandatory `*_nyes_transitions` tests) per file. Kind-spanning tests move to the module that owns
the behavior under test.

### Hard rules

1. **Move-only.** No renames of types/functions, no signature changes, no visibility widening.
   Visibility may need `pub(crate)` where file boundaries now intervene — never `pub`.
2. **Zero behavior change, snapshot-invisible.** `cargo test --workspace` green after every
   commit; the einmo/insta corpus byte-identical (the Track-0 gate proves it).
3. **One module per commit**, so `git log --follow` traces every line.
4. **The crate's external API is frozen**: `lib.rs` re-exports keep every path any other crate
   uses working unchanged.
5. Line-reference churn in open FOOPs is accepted and expected; the roadmap (INDEX.md) is the
   place that says "line refs predate FOOP-05."

## FIR Impact

None. No FIR variant, state, or transition changes — files move, semantics do not.

## UBC Step Impact

None.

## Test Plan

- The entire existing suite, green at every commit — that IS the test plan. No new behavior to
  test; the einmo gate (Track 0) provides the byte-identity proof.
- One new guard: a `lib.rs` doc-comment listing the module map, so drift is visible in review.

## Rejected Alternatives

### A. Do nothing
Tracks 2 and 3 serialize behind each other or merge-conflict for months in two mega-files.

### B. Split only `fir_kinds.rs`, leave `fir_trait.rs`
Half the collision surface remains (`step_inner`, extension traits); the shuffle is cheap while
we are already moving code.

### C. Deep refactor while moving (builders, error consolidation, FOOP-82 architectural items)
Mixing behavior change into a move destroys the reviewability of both. Mechanical move now;
FOOP-82's architectural findings get their own pass later.

## Open Questions

- Exact per-kind file inventory (enumerate `FirKind` at execution time).
- Whether the `contextful_search` inner tests stay in `fir_search_base.rs` or get a sibling
  test module.

## References

- `rust_instructions.md` §2e/§2f (module structuring, private-by-default, curated re-exports).
- Roadmap P4 in `docs/foop/INDEX.md`; FOOP-82 (architectural findings deferred to a later pass).
- Atlas direction 2026-07-14: base/default behavior in `fir_base.rs`, search base in
  `fir_search_base.rs`, one FIR per file under `firs/`, standard Rust idiomatic organization.
