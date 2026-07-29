---
foop: 49
title: Brane NK only when all constituents are NK — remove any-NK contamination
author: Atlas <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-07-14
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-94: Brane NK only when all constituents are NK — remove any-NK contamination

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

## Abstract

Investigate and remove the rule that marks a brane NK when it merely *contains* NK constituents.
The container-level NK has little programmatic use, no internal state relies on it, and it
complicates the state machine precisely when only a few constituents are NK. The proposed rule:
**a brane classifies NK only when every constituent is NK**; otherwise NK members count as
settled values and the brane classifies as if they were constants. Nothing else changes —
names inside an NK brane remain searchable, NK sub-branes remain searchable, and operator NK
propagation (`5 + NK → NK`) is untouched. The only observable difference is *when* a brane
transitions to NK.

## Motivation

Today a brane holding one failed division marks the entire container NK even though every other
member holds a value and remains fully searchable:

```foolish
{a = 5; b = 10 / 0; c = 7;}    !! today: the BRANE settles NK because b is NK
```

The container's NK is read by nobody: searches do not consult it (they go inside and find `a`
and `c` regardless), the humanizing sequencer prints the members either way, and the evaluator's
own stepping never branches on a *brane's* NK — only on constituents'. Meanwhile the marking
contaminates transitively (an outer brane containing this brane sees an NK child) and forces
every reasoning-about-state discussion to carry the caveat "NK, but only sort of." An all-NK
brane, by contrast, genuinely denotes "provably nothing findable here" and keeps the terminal
signal.

## Specification

### The rule change

All brane-like classification flows through one shared function,
`_decide_nyes_due_to_children` (`foolish-ubca/src/fir_kinds.rs:11`). Decision cascade, before
and after (first match wins; applies once all children are constanic or a pre-constanic child
exists):

| # | Condition | Today | Proposed |
|---|-----------|-------|----------|
| 1 | children non-empty, **all** NK | — (falls to #5) | **NK** |
| 2 | all INDEPENDENT | INDEPENDENT | INDEPENDENT |
| 3 | all ∈ {CONSTANT, INDEPENDENT} — proposed: {CONSTANT, INDEPENDENT, **NK**} | CONSTANT | CONSTANT |
| 4 | any pre-constanic (PREMBRYONIC/EMBRYONIC/BRANING) | BRANING | BRANING |
| 5 | any ECONSTANIC/WOCONSTANIC | WOCONSTANIC | WOCONSTANIC |
| 6 | any NK, rest CONSTANT/INDEPENDENT | **NK** | — (covered by #3) |

Consequences:

- `{a = 5; b = 10 / 0; c = 7;}` → brane **CONSTANT** (today NK). The division alarm is still
  emitted (alarms come from the operator, not from brane classification), and `b` itself is
  still NK.
- `{a = 10 / 0;}` → brane **NK**, unchanged (single constituent, all-NK).
- NK does **not** count as INDEPENDENT-like: `{independents… + one NK}` classifies CONSTANT,
  not INDEPENDENT (conservative; see Open Questions).
- An empty brane remains CONSTANT (`BraneFir` handles it before classification; the all-NK
  check requires non-empty children).

### Uniform application

The shared classifier is called from four sites (`fir_kinds.rs:801` — `BraneFir`; `:2170`,
`:2237`, `:2448`). The change lands once in the classifier and applies uniformly; the
investigation phase confirms each caller's semantics under the new rule — in particular that a
concatenation containing NK elements behaves exactly like a brane (NK only when all elements
are NK). The test-helper brane in `fir_trait.rs` (its private `fir_op_step`, the `any_nk` block
near `:476`) duplicates the old rule and must be updated to match, or better, delegated to the
shared classifier.

### Explicitly out of scope

- **Operator NK propagation** (`OperatorFir::combine`, `fir_kinds.rs:530`): `5 + NK → NK`
  stays. An operator *consumes* its operands' values; a brane merely *contains* statements.
- **`NkFir` itself** — unchanged, including its reason string.
- **Search semantics** — unchanged and to be *pinned*: searches into an NK brane already reach
  non-NK members; anchored/unanchored miss outcomes (NK vs ECONSTANIC) are untouched.

### Preserved invariants (pinned by new tests)

1. Names inside an NK brane are searchable (`{nk_only…}?x` semantics unchanged).
2. NK sub-branes are searchable through (deepening into an NK sub-brane works).
3. A search into a mixed brane finds its non-NK members.
4. The humanizing sequencer prints all members of a brane regardless of the brane's own NYES.

## Investigation questions (answered during execution, recorded here)

- Does *any* code read a brane's NK status programmatically? (Expected: none — confirm by
  auditing every `get_nyes()` consumer for a brane receiver.)
- Do all four `_decide_nyes_due_to_children` call sites want the new rule? (Expected: yes.)
- Does the hfssnap header of a mixed brane change (e.g. `{NK` → `{C`), and is that the only
  sequencer-visible difference?

## FIR Impact

No new FIR variants; no NYES states added or removed. One transition-rule change in the shared
child-classifier. Per AGENTS.md, any NYES transition change **must** extend the
`*_nyes_transitions` unit tests — `brane_nyes_transitions` (and the concat variants) gain the
mixed-children and all-NK progressions.

## UBC Step Impact

`BraneFir::fir_op_step` (BRANING classification) and every other `_decide_nyes_due_to_children`
caller: before/after per the cascade table above. No stepping-order change; no task-queue
change; settlement still occurs on the same step — possibly to CONSTANT where it was NK.

## Test Plan

- **Unit (write first):** rename `brane_with_nk_child_classifies_nk` →
  `brane_with_nk_child_classifies_constant`; add `brane_all_nk_children_classifies_nk`,
  `brane_single_nk_child_classifies_nk`, concat equivalents; extend `*_nyes_transitions`;
  add the four preserved-invariant search tests.
- **Approval:** ~34 approved snapshots contain a brane-level NK; the mixed-content ones
  (e.g. `alarm_mixed_alarms_and_normals`, `alarm_division_by_zero_in_brane`,
  `division_by_zero_in_nested_brane`) will change. Generate `.snap.new`, verify each change is
  exactly a container-state change (members and alarms byte-identical), present to the human.
  **Never auto-accept.**
- **Comprehensive:** `foolish-ubca/snapshot_tests/input/foop_94_comprehensive.foo` — mixed
  NK/value branes nested, searched (anchored, contexted, value), concatenated, and fed through
  operators. (If FOOP-64's einmo suite has merged by execution time, the reserved path is
  `foolish-ubca/einmo_suite/input/foop/94/comprehensive.foo` instead.)

## Rejected Alternatives

### A. Do nothing
The any-NK contamination keeps complicating container states with no programmatic consumer, and
every future feature touching branes inherits the caveat.

### B. Remove brane NK entirely (a brane is never NK)
Loses the terminal "provably nothing findable here" signal that an all-NK brane genuinely
carries, and churns even more snapshots for no semantic gain.

### C. Treat NK as INDEPENDENT-like too
`{independent…, NK}` → INDEPENDENT overstates self-containment: an NK carries a reason bound to
the context in which the search failed. Rejected conservatively; revisit under Open Questions if
the investigation shows container INDEPENDENT status has no such sensitivity.

## Open Questions

- Should a brane of all-INDEPENDENT members plus NKs classify INDEPENDENT rather than CONSTANT?
- Should the test-helper brane in `fir_trait.rs` delegate to `_decide_nyes_due_to_children`
  instead of duplicating the cascade?
- Exact sequencer rendering of a mixed brane after the change (investigation question 3).

## References

- Code: `foolish-ubca/src/fir_kinds.rs:11` (`_decide_nyes_due_to_children`), `:801` (BraneFir),
  `:2170`/`:2237`/`:2448` (other callers), `:530` (operator NK propagation — out of scope);
  `foolish-ubca/src/fir_trait.rs:476` (test-helper duplicate).
- Prior FOOPs: FOOP-43 (search miss settles ECONSTANIC, not NK — kindred NK-footprint
  reduction), FOOP-11 (Deprecated: search stops at NK), FOOP-45 (Deadbrane — useless-element
  detection, adjacent territory; renumbered from FOOP-84 on 2026-07-29, which is the Search
  Engine Refactor).
- AGENTS.md §"NK vs ECONSTANIC miss outcomes", §"NYES transition tests".
