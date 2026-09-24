---
foop: 49
title: Brane NK only for an unsteppable statement — remove NK contamination
author: Atlas <hc.busy@gmail.com>
status: Complete
type: Standards
created: 2026-07-14
phase: phase-2
supersedes: []
begun: [x]
---

# FOOP-94: Brane NK only for an unsteppable statement — remove NK contamination

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

**AMENDED 2026-09-23 by the human, strengthening the original proposal.** As first written, this
FOOP kept an all-NK brane NK (row 1 below). The amended rule removes that too:

> **A brane is NK if and only if it contains an unsteppable statement.** No other brane is NK —
> not one with some NK members, not one whose members are ALL NK.

Brane NK therefore stops being a rollup of member states and becomes purely the record of a
run-time error (FOOP-86 §6.2a). It is written directly by `halt_if_unsteppable`, never derived
from children.

All brane-like classification flows through one shared function, `decide_nyes_due_to_children`
(`foolish-ubca2/src/fvm_storage.rs`; the original text cited `foolish-ubca/src/fir_kinds.rs:11`,
a crate FOOP-86 has since retired). Decision cascade, first match wins:

| # | Condition | Today | Amended |
|---|-----------|-------|---------|
| 1 | children non-empty, **all** NK | NK (via #6) | **CONSTANT** — the amendment |
| 2 | all INDEPENDENT | INDEPENDENT | INDEPENDENT |
| 3 | all ∈ {CONSTANT, INDEPENDENT, **NK**} | CONSTANT | CONSTANT |
| 4 | any pre-constanic | BRANING | BRANING |
| 5 | any ECONSTANIC/WOCONSTANIC | WOCONSTANIC | WOCONSTANIC |
| 6 | any NK, rest CONSTANT/INDEPENDENT | **NK** | — (covered by #3) |

**The classifier never returns NK.** A halted brane is guarded before classification
(`decide_brane_nyes`), so the halt's NK is never overwritten.

Consequences:

- `{a = 5; b = 10 / 0; c = 7;}` → brane **CONSTANT** (today NK). The division alarm is still
  emitted (alarms come from the operator, not from brane classification), and `b` itself is
  still NK.
- `{a = 10 / 0;}` → brane **CONSTANT** (today NK; the original proposal kept it NK).
- `{'K = ⬤; 'K = 3;}` → brane **NK** — an unsteppable statement, the only remaining route.
- NK does **not** count as INDEPENDENT-like: `{independents… + one NK}` classifies CONSTANT,
  not INDEPENDENT. **Conservatism confirmed by the human 2026-09-23** and to be documented:
  INDEPENDENT asserts "no context dependencies at all", which an NK member's unresolved history
  does not support.
- An empty brane remains CONSTANT.

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

- ~~Should a brane of all-INDEPENDENT members plus NKs classify INDEPENDENT rather than CONSTANT?~~
  **RESOLVED 2026-09-23 (the human): no — CONSTANT, and the conservatism is to be documented.**
- ~~Should the test-helper brane in `fir_trait.rs` delegate to `_decide_nyes_due_to_children`?~~
  **MOOT** — `fir_trait.rs` left with `foolish-ubca`, retired by FOOP-86.
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

## Last Updated

**Date**: 2026-09-24
**Updated By**: Claude Code / claude-opus-5
**Changes**: Status Draft → Complete; implemented. The rule was AMENDED by the human 2026-09-23,
strengthening the original proposal: an all-NK brane is no longer NK either, so **a brane is NK if
and only if it contains an unsteppable statement**. Brane NK is therefore purely the record of a
run-time error (FOOP-86 §6.2a), written directly by `halt_if_unsteppable`, never derived from
children — the classifier now never returns NK at all. Implementing this exposed a REGRESSION the
old rule had masked: the cascade ran after the halt and reclassified halted branes, erasing their
`!! NK:` annotations (caught by `foop/86/unsteppable_concat_merge`). Previously invisible because
the any-NK rule happened to re-derive the same NK the halt had written. Fixed with
`decide_brane_nyes`, which consults `unsteppable_cause` before classifying, plus a matching check
in the concatenation arm where a route-2 cause sits on a HELPER rather than on the node. Both open
questions resolved: Independent+NK classifies CONSTANT (conservative, per the human), and the
`fir_trait.rs` question is moot since that file retired with `foolish-ubca`. Two unit tests
rewritten rather than deleted, both having existed solely to pin the removed rule:
`brane_with_nk_child_settles_nk` → `..._settles_constant_not_nk`, and the sequencer's "rollup NK
shown under verbose" block, whose second assertion is now unsatisfiable by design — replaced with
a check that an UNSTEPPABLE brane's NK still shows under verbose, keeping that machinery pinned.
Zero einmo baselines moved; workspace 467 tests, 0 failures.
