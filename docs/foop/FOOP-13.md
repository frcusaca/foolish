---
foop: 31
title: MAX_BRANE_SIZE — auto-size oversized branes into concatenations of bounded branes
author: Atlas <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-07-03
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-13: MAX_BRANE_SIZE — auto-size oversized branes into concatenations of bounded branes

## Abstract

Each UBCa FVM gains a configuration value `MAX_BRANE_SIZE`. During brane construction from the
AST, the compiler automatically converts any brane whose statement count exceeds `MAX_BRANE_SIZE`
into a concatenation of smaller branes, each within the limit. Concatenation already merges its
brane elements back into a single brane (FOOP-3, `ConcatenationFir`), so the observable evaluation
result is unchanged; only the constructed FIR shape differs. The default is unlimited, so all
existing behavior and all approved snapshots are unaffected.

## Motivation

Branes are the unit of containment, cloning, and recoordination in the UBC model. A single very
large brane is a single very large unit: it clones as one block, steps as one block, and (in future
phases) ships across evaluator boundaries as one block. Bounding brane size at construction time
gives the FVM a uniform upper bound on the granule size it manipulates, exercised through the
concatenation machinery the language already has — a brane of `n` statements is semantically the
concatenation of consecutive sub-branes covering those statements in order.

Today there is no such bound and no per-FVM configuration surface on UBCa at all
(`UbcaEvaluator` is a unit struct; `Compiler::compile` is a static function). After this FOOP, the
FVM carries a small configuration struct, `MAX_BRANE_SIZE` is its first knob, and oversized branes
are transparently rewritten at construction time.

## Specification

### Size

The **size of a brane** is its number of statements — the length of `statements` in
`Astn::Brane { characterizations, statements }`. Nested content does not count toward the size of
the outer brane; each brane is measured on its own statement list.

### Configuration

A new configuration struct in `foolish-ubca`:

```rust
/// Per-FVM configuration for UBCa (FOOP-13).
#[derive(Debug, Clone, Default)]
pub struct UbcaConfig {
    /// Maximum number of statements a constructed brane may hold.
    /// `None` (the default) means unlimited — no auto-sizing occurs.
    pub max_brane_size: Option<std::num::NonZeroUsize>,
}
```

- `UbcaEvaluator` changes from a unit struct to `pub struct UbcaEvaluator { pub config: UbcaConfig }`
  with `Default`. Existing constructions of `UbcaEvaluator` continue to work via `Default`.
- The compiler gains `Compiler::compile_with(source: &str, config: &UbcaConfig)`.
  `Compiler::compile(source)` delegates to `compile_with` with `UbcaConfig::default()` — byte-for-byte
  the current behavior.
- `NonZeroUsize` makes the degenerate `MAX_BRANE_SIZE = 0` unrepresentable; "no limit" is spelled
  `None`, not `0`.

### The auto-sizing rewrite

Auto-sizing is a **pure AST→AST rewrite** applied inside the compiler after `validate_astn` and
before `build_fir`. Let `k = max_brane_size`. The rewrite recurses structurally through every
`Astn` variant; the only rewriting arm is `Astn::Brane`:

For `Astn::Brane { characterizations, statements }` with `n = statements.len()`:

1. Recurse into each statement first (nested branes are auto-sized independently).
2. If `n <= k`, or the brane is **exempt** (below), leave the brane as-is.
3. Otherwise split `statements` (order preserved) into `m = ⌈n/k⌉` consecutive chunks — every
   chunk holds exactly `k` statements except the last, which holds the remainder — and replace the
   brane with:

```text
Astn::Concatenation {
    elements: [ Brane(chunk₁), Brane(chunk₂), …, Brane(chunkₘ) ],
}
```

Each chunk brane has empty characterizations. Chunk branes are ≤ `k` by construction, so no
re-recursion into chunks is needed (their statements were already rewritten in step 1).

Because Foolish concatenation is juxtaposition (`b1 b2 b3`) and `ConcatenationFir` merges the
statements of its brane elements, in order, into one result brane, the rewrite is the identity
under evaluation: `{s₁; …; sₙ}` and `{s₁; …; s_k} {s_{k+1}; …} …` settle to the same brane.

### Exemptions

Two brane positions are never split:

- **The root brane.** Only a `Brane` may be a root node (FOOP-62 root convention;
  `compile_standalone` rejects anything else). Rewriting the root into a `Concatenation` would
  make the program unrootable. The root's *nested* branes are still auto-sized; the root's own
  statement list is not.
- **Characterized branes** (`characterizations` non-empty). The merged brane produced by
  `ConcatenationFir` carries no characterizations, so splitting a characterized brane would drop
  its characterizations. Conservative rule: leave it whole. (Revisit if/when characterizations
  gain merge semantics.)

## FIR Impact

None. No new FIR variant, no new NYES state, no transition change — therefore no new or changed
`*_nyes_transitions` tests are required. The rewrite only changes *which* existing FIRs
(`ConcatenationFir` + `BraneFir`) the compiler constructs.

## UBC Step Impact

None. Evaluation of the rewritten form uses the existing `ConcatenationFir` step rules (empty /
Braning merge) unchanged. Cross-chunk name references resolve exactly the way references across an
explicit source-level concatenation already resolve (see `concatenation_references.foo.snap`);
the test plan pins that a split brane settles to the same result as the unsplit brane, including a
case where a statement references a name defined in an earlier chunk.

## Test Plan

Unit tests in `foolish-ubca` (compiler tests module, plus one settle-and-compare test where the
evaluator infrastructure lives). No `.foo` approval tests: `MAX_BRANE_SIZE` is FVM configuration,
not language surface — the default configuration leaves every existing snapshot byte-identical,
and the approval harness runs the default configuration.

- `unlimited_config_is_identity` — `compile_with` with `max_brane_size: None` produces the same
  FIR shape as `compile` (no `ConcatenationFir` introduced anywhere).
- `brane_at_or_under_max_is_not_split` — a nested brane with exactly `k` statements stays a
  single `BraneFir`.
- `oversized_brane_splits_into_chunked_concatenation` — a nested brane with 5 statements and
  `k = 2` compiles to a `ConcatenationFir` of 3 `BraneFir` elements with sizes 2, 2, 1; statement
  names and order are preserved across the chunk boundaries.
- `root_brane_is_never_split` — a root brane with `n > k` remains a root `BraneFir` (its nested
  branes may still split).
- `characterized_brane_is_never_split` — a characterized oversized brane stays whole.
- `split_brane_settles_to_same_result_as_unsplit` — compile one program twice (unlimited vs small
  `k`), step both to settled, and assert the humanized/sequenced results are identical. The
  program includes a statement that references a name defined in an earlier chunk, pinning
  cross-chunk resolution.
- `oversized_brane_inside_explicit_concatenation` — an oversized brane appearing as an element of
  a source-level concatenation (`a {big} b`) produces a nested `ConcatenationFir` element and
  still settles to the same result; documents the nested-concatenation corner.

## Rejected Alternatives

### A. Do nothing

Leaves brane granularity unbounded and leaves UBCa with no configuration surface. The first
consumer of per-FVM configuration would still have to build `UbcaConfig`; doing it now with a
small, semantics-preserving knob is the cheap path.

### B. Rewrite at FIR construction (inside `build_fir`) instead of AST→AST

Chunking inside the `Astn::Brane` arm of `build_fir` requires hand-wiring parent `Weak`s for the
synthetic `ConcatenationFir`/`BraneFir` layers inside `Rc::new_cyclic`, duplicating wiring that
`build_fir` already does when handed a `Concatenation` AST node. The AST→AST rewrite reuses all
of it and is trivially unit-testable in isolation. (Easiest to prove correct wins.)

### C. Splice chunk branes into an enclosing source-level concatenation (flattening)

When the oversized brane is already an element of an `Astn::Concatenation`, its chunks could be
spliced inline instead of nesting a concatenation. Rejected as unnecessary: nested concatenation
elements already merge correctly through the existing `Braning` arm, and the non-flattened form
keeps the rewrite a single local transformation.

### D. Split evaluated branes at step time instead of construction time

Auto-sizing during evaluation (e.g. when a merged brane exceeds `k`) touches the step rules and
constanic-clone machinery, and can oscillate against `ConcatenationFir`'s own merging. Out of
scope; this FOOP is construction-from-AST only, exactly as motivated.

## Open Questions

- Should characterized branes eventually split, with characterizations attached to the merged
  result? Deferred until characterizations have defined merge semantics.
- Should the root brane's statement list eventually be bounded too (e.g. by an implicit wrapper)?
  Deferred; requires changing the root convention or the output shape.

## References

- Prior FOOPs: FOOP-3 (concatenation merge semantics), FOOP-62 (UBCa ProtoBrane, root convention).
- Code locations: `foolish-ubca/src/compiler.rs` (`Compiler::compile`, `build_fir`,
  `compile_standalone`), `foolish-ubca/src/evaluator.rs` (`UbcaEvaluator`),
  `foolish-ubca/src/fir_kinds.rs` (`ConcatenationFir::fir_op_step`).
- Snapshots demonstrating concatenation-of-branes merging: `concatenation_three_way.foo.snap`,
  `concatenation_references.foo.snap`, `concatenation_inline_branes.foo.snap` under
  `foolish-ubca/snapshot_tests/approved/`.

## Last Updated

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Initial draft — MAX_BRANE_SIZE configuration, AST-level auto-sizing rewrite,
exemptions (root, characterized), test plan, rejected alternatives.
