---
foop: D65
title: NYES groups — one predicate per group, and "settled" qualified everywhere
author: Claude Code / claude-opus-5 (directed by the human)
status: Draft
type: Standards
created: 2026-09-02
phase: phase-4
supersedes: []
begun: [ ] 
---

# FOOP-56: NYES groups — one predicate per group, and "settled" qualified everywhere

FOOP numbering is little-endian; the full rules live in `foop.md` at the repository root —
**read it before creating or editing a FOOP.** The `foop:` front-matter field here is the
big-endian sort key preceded by `D` (`foop: D65`, file `FOOP-56.md`, following FOOP-46's 64).

## Abstract

Foolish names four groups of NYES states. This FOOP gives each one a **predicate**, so code
asks its question by name instead of hand-rolling a state list, and **qualifies every bare use
of "settled"** with the group it means.

The four groups, and the predicates this FOOP settles on:

| Group | States | Predicate |
|---|---|---|
| **Pre-constanic** (nigh) | PREMBRYONIC, EMBRYONIC, BRANING | `is_preconstanic()`, with **`is_nye()` as an alias** |
| **Constanic** | ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, NK | `is_constanic()` |
| **Constantew** | CONSTANT, INDEPENDENT, NK | `is_constantew()` |
| **Conclusive** | CONSTANT, INDEPENDENT | `is_conclusive()` |

`is_constanic` and `is_constantew` exist. `is_conclusive` and `is_preconstanic` do not, and
their absence is visible: `foolish-ubca2/src/fvm_storage.rs` writes
`matches!(nyes, Nyes::Constant | Nyes::Independent)` at five separate sites, each a conclusive
test spelled as a state list.

Alongside that, the word **"settled"** appears 134 times in `foolish-ubca2` — always as prose,
never as a callable predicate — and it does not mean one thing. It means constanic at some
sites, conclusive at others, and at one site names a value that may still be BRANING. Each use
gains a qualifier naming its group.

Alongside the code, **every document describing `foolish-ubca2` is brought onto the same
vocabulary** — FOOP-16, FOOP-26, FOOP-36, FOOP-46, AGENTS.md's terminology entries, and the
crate's own module docs (§4). That uniformity is the point: the vocabulary is worth little if
the specifications an agent reads still say "settled" and leave the group implicit.

This is a **vocabulary and clarity** change. No behaviour changes, no FIR changes, no einmo
baseline moves.

## Motivation

### The code cannot say what it means

`fvm_storage.rs` line 816 decides whether an operator must keep stepping:

```rust
let all_settled = children
    .iter()
    .all(|&c| matches!(storage.get_nyes(c), Nyes::Constant | Nyes::Independent));
```

Three things are wrong with how that reads, none of them with what it does. The name says
"settled", which elsewhere in the same file means *constanic*. The test is actually
*conclusive* — an ECONSTANIC child is constanic and still gets queued. And the rule is
expressed as a two-state list, so a reader has to reconstruct the concept rather than read it.

Written with the vocabulary in place:

```rust
let all_foolish_children_conclusive = children
    .iter()
    .all(|&c| storage.get_nyes(c).is_conclusive());
```

Now the line states the rule: *an operator keeps stepping unless every foolish child is
conclusive.*

### "Settled" means four different things

A survey of all 134 occurrences in `foolish-ubca2` (§2) finds them split across:

- **constanic** — `settled_result` (gates `is_constanic()`), `step_to_settled`, the two
  NF-redefinition comparisons, `anchor_settled`;
- **conclusive** — `all_settled` at line 816, the one site where the distinction bites;
- **a computed classification that may still be pre-constanic** — `let settled =
  decide_nyes_due_to_children(…)`, which can hold **BRANING**, making the name actively wrong;
- **prose** — ~120 doc comments and test names, mostly accurate but imprecise.

The word is not the problem; every agent working on this codebase reached for it, and it reads
well. The problem is that it is used *bare*, leaving the group implicit.

### Two FOOPs are waiting on this

**FOOP-36** (sequencer rendering Foolish) states its central rule over *conclusive* and
*inconclusive constanic*. **FOOP-26** (marks, concatenation, three-beat step) reasons
throughout about which children must be constanic before a step proceeds. Both are clearer to
write, and to review, once the vocabulary exists in the code they describe — and both edit
`foolish-ubca2`, so a rename touching ~20 call sites is far cheaper before they begin than
after they have diverged.

Hence the ordering this FOOP asks for: **FOOP-56 → FOOP-36 → FOOP-26.**

## Specification

### §1 The four predicates

All four live on **`NyesExt`** (`foolish-ubca2/src/nyes_ext.rs`), the crate's existing
extension trait on `foolish_core::fir::Nyes`:

```rust
/// Pre-constanic (nigh): PREMBRYONIC, EMBRYONIC, BRANING — still stepping.
fn is_preconstanic(&self) -> bool {
    !self.is_constanic()
}

/// Not Yet Evaluated — the older name for the same group. An alias, kept so the
/// traditional Foolish vocabulary still reads.
fn is_nye(&self) -> bool {
    self.is_preconstanic()
}

/// All terminal states: ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, NK.
fn is_constanic(&self) -> bool;      // exists

/// Constant everywhere: CONSTANT, INDEPENDENT, NK.
fn is_constantew(&self) -> bool;     // exists

/// Conclusive: the FIR reached a value — CONSTANT or INDEPENDENT.
/// Distinct from `is_constantew()`, which also admits NK: NK is constant
/// everywhere yet never produced a value.
fn is_conclusive(&self) -> bool {
    matches!(self, Nyes::Constant | Nyes::Independent)
}
```

**`is_preconstanic` is primary; `is_nye` delegates to it.** The four then read uniformly while
the traditional name keeps working.

`is_nnk_constanic()` stays as it is — a refinement (constanic but not NK), not one of the four
groups.

**`foolish-core` is not modified.** `foolish_core::Nyes::is_nye()` exists at `fir.rs:143` and
has **zero callers anywhere in the workspace** (verified 2026-09-02), so `NyesExt`'s method
shadows nothing in practice. Leaving `foolish-core` alone keeps this FOOP off the path of
`foolish-ubca`, whose einmo baselines must not move.

### §2 "Settled" gains a qualifier

Every bare "settled" in `foolish-ubca2` is qualified with its group. The renames:

| Today | Becomes | Because |
|---|---|---|
| `FirPointer::settled_result` (639) | `settled_constanic_result` | gates `is_constanic()`; §2.1 confirms that is what the slot holds |
| `FirCursor::settled_result` (1602) | `settled_constanic_result` | delegates to the above |
| `step_to_settled` (3272) | `step_to_constanic` | loops until `is_constanic()` |
| `all_settled` (816) | `all_foolish_children_conclusive` | gates `Constant \| Independent`; it iterates `foolish_children`, so name that rather than "operands" |
| `let settled = decide_nyes_due_to_children(…)` (1070) | `let decided_nyes = …` | can hold **BRANING** — "settled" is wrong, not merely vague |
| `settled_nyes = nyes_from_found(…)` (968) | `constanic_nyes` | its output is always constanic; its input need not be |
| `anchor_settled` (3178) | `anchor_constanic` | gates `is_constanic()` on the anchor |

Test names follow: `indep_int_stepping_already_settled_is_noop` → `…_already_conclusive_…`;
`operator_pushes_tasks_for_unsettled_operands` → `…_for_inconclusive_operands`;
`revive_constanic_unwraps_stay_foolish_to_its_settled_result` → `…_settled_constanic_result`.

Doc comments using a bare "settled" gain the group adjective. Where a comment is already
unambiguous from its context, it is left alone — this is clarification, not a sweep for its
own sake.

#### §2.1 Why `settled_result` is *constanic* and not *constantew*

Two mechanisms push toward a narrower name, and it is worth recording why they do not reach it:

- **`Nyes::transform_for_clone`** preserves only CONSTANT, INDEPENDENT and NK — exactly
  constantew — turning everything else EMBRYONIC. A result arriving via `clone_stmt_result` →
  `revive_constanic` is constantew or embryonic.
- **`push_ubc_child`** (line 151) queues a non-constanic child as a task, so an embryonic entry
  is stepped onward.

**But ECONSTANIC and WOCONSTANIC do reach the slot** by a route that bypasses cloning:
`StayFoolish` "expose[s] EXPR'S OWN resolved value … adopting that value's `Nyes`" (lines
902–904), and the writes at 932 and 970 pass a found value's NYES through unchanged. So
`settled_constantew_result` would promise something the SF path does not deliver.

### §3 The five hand-rolled conclusive tests

`fvm_storage.rs` lines **818, 2007, 3739, 3810, 3950** each write
`matches!(…, Nyes::Constant | Nyes::Independent)`. Each becomes `.is_conclusive()`.

**Read each before replacing it.** Line **1375** also matches those two states but is a mapping
arm inside `nyes_from_found`, not a test — it stays as it is. The distinction is exactly the
kind a blind `sed` gets wrong.

### §4 Documentation — uniformly, across everything describing `foolish-ubca2`

The predicates and renames are half the job. The other half is that **every document describing
`foolish-ubca2` says which NYES group it means**, so the next agent reads one vocabulary rather
than inferring from context. The same pass applies to each: qualify bare "settled"; correct any
"constanic" that actually means *conclusive*; update renamed identifiers; name a group where
prose hand-rolls a state list; and **list anything genuinely ambiguous for the human rather
than guessing**.

The documents in scope, with their starting counts (measured 2026-09-02):

| Document | "settled" | "constanic" | Note |
|---|---|---|---|
| `FOOP-16.md` / `.plan.md` | 8 / 30 | 28 | built the crate; its plan is largely complete, so completed checkboxes stay as the historical record |
| `FOOP-26.md` | 21 | 87 | marks / concatenation / three-beat step — **heaviest, and where constanic-vs-conclusive matters most** |
| `FOOP-36.md` / `.plan.md` | 56 | 159 | the sequencer FOOP this one was extracted from |
| `FOOP-46.md` | 9 | 29 | BraneConcatOp; spec only, no plan yet |
| `AGENTS.md` §Foolish Terminology | — | — | the authority; gains the predicate name on each entry |
| `nyes_ext.rs`, `lib.rs`, `MAPPING.md` | — | — | module docs in the crate itself |

**Why "constanic → conclusive" is the delicate step.** FOOP-26's three-beat step waits for
`foolish_children` to become constanic; `fvm_storage.rs:818` gates that on `Constant |
INDEPENDENT`, i.e. **conclusive**. If the spec means the code's rule, it should say conclusive.
That is a **specification correction**, not a wording tweak, and every such case goes to the
human in one consolidated list before it is treated as settled.

**Deliberately out of scope:** FOOP-62, FOOP-23, FOOP-33, FOOP-55 and the other
`foolish-ubca` FOOPs — shipped, or describing the sibling crate. Rewriting them is a larger act
than this FOOP claims.

- `nyes_ext.rs`'s module doc lists "Three categories" and omits conclusive; it gains the fourth
  and the alias.
- `lib.rs` line 24 claims `NyesExt` "adds `is_settled()` to `Nyes`". **No such method exists.**
  Corrected.
- `AGENTS.md` §Foolish Terminology already defines all four groups (it is the authority); this
  FOOP adds the predicate names to those entries so a reader moves from concept to call.
- FOOP-62 §Terminology lists `is_settled()` and `is_constantew()` as predicates. `is_settled()`
  was never implemented. FOOP-62 is a shipped FOOP and is **not rewritten**; the discrepancy is
  noted here instead.

## FIR Impact

**None.** No new FIR variant, no state-machine change, no serialization change. `Nyes` itself is
untouched; only an extension trait in `foolish-ubca2` gains methods.

## UBC Step Impact

**None.** Every change is a rename, a doc comment, or a `matches!` replaced by an equivalent
predicate call. No step rule changes, no evaluation order changes, no NYES transition changes,
no step counts move.

## Test Plan

**T1 — Predicate unit tests** in `nyes_ext.rs`'s `tests` module, in the shape of the existing
`constantew_states()`, asserting over `ALL_NYES`:

- `conclusive_states()` and `preconstanic_states()`
- `is_nye_is_alias_for_preconstanic()` — the two agree on every state
- `conclusive_is_subset_of_constantew()`, mirroring `constantew_is_subset_of_constanic()`
- `conclusive_and_constantew_differ_exactly_on_nk()` — the load-bearing distinction, pinned
- `preconstanic_is_complement_of_constanic()` — every state is in exactly one

**T2 — The conclusive-vs-constanic distinction at line 816.** The existing
`operator_pushes_tasks_for_unsettled_operands` uses PREMBRYONIC operands only, so it does not
distinguish the two. Add a case with an **ECONSTANIC** operand: constanic, yet still queued as a
task. That rule is currently untested.

**T3 — Behaviour unchanged.** `cargo test -p foolish-ubca2 --lib` is **134/134 before and
after**. This FOOP renames and re-documents; **any** test movement means something else was
changed and must be reverted.

**T4 — Neighbours untouched.** `cargo test -p foolish-ubca --lib -- einmo_gate_checked` passes
unchanged, and `git diff --stat` shows changes only under `foolish-ubca2/src/` (plus the
documentation files named in §4).

## Rejected Alternatives

### A. Do nothing
Leaves five hand-rolled state lists, a variable named `settled` that can hold BRANING, and a
`lib.rs` comment promising a method that does not exist. It also leaves FOOP-36 and FOOP-26 to
be written in vocabulary their own codebase does not speak. Rejected.

### B. Do it inside FOOP-36
Where this work started. Rejected because it is not about rendering: FOOP-26 benefits equally,
and burying a cross-cutting vocabulary change inside a sequencer FOOP makes both harder to
review and forces FOOP-26 to depend on FOOP-36 for a reason unrelated to either.

### C. Put the predicates in `foolish-core`
`Nyes` lives there, so the methods arguably belong there. Rejected: it would put a change on
`foolish-ubca`'s compile path for no benefit to it, and `foolish-ubca`'s einmo baselines are
exactly what must not move. `NyesExt` already exists for precisely this.

### D. Rename `is_nye()` outright instead of aliasing
Cleaner in isolation. Rejected because "nye" is established Foolish vocabulary (`AGENTS.md`
§Foolish Terminology, and the `*_nyes_transitions` test convention); an alias costs one
delegating line and keeps both readings.

### E. Replace "settled" everywhere with the group name
Rejected as over-correction. "Settled" reads well and is what every agent working here reached
for; the fix is a qualifier, not a ban.

## Open Questions

- **Q1.** Should the four predicates eventually move to `foolish-core` so `foolish-ubca` shares
  them? Deliberately **not** done here (Rejected Alternative C). Worth revisiting once
  `foolish-ubca2` has replaced `foolish-ubca` — at which point the question dissolves.
- **Q2.** FOOP-62 §Terminology specifies `is_settled()`, which was never implemented. Leave the
  discrepancy noted (§4), or amend FOOP-62? Amending a shipped FOOP is the heavier act;
  recommendation is to leave it and let this FOOP be the current word.

## References

- **FOOP-36** — the Foolish-rendering sequencer, whose §3 rule is stated over *conclusive* and
  *inconclusive constanic*. Scheduled immediately after this FOOP.
- **FOOP-26** — marks, concatenation-as-operator, three-beat step; reasons throughout about
  which children must be constanic. Scheduled after FOOP-36.
- **FOOP-62** §Terminology — the source of *constanic* / *constantew*, and of the unimplemented
  `is_settled()` / `is_constantew()` predicate list.
- `AGENTS.md` §Foolish Terminology — the authority on all four group definitions.
- `foolish-ubca2/src/nyes_ext.rs` — where the predicates live.
- `foolish-ubca2/src/fvm_storage.rs` — the 131 "settled" uses and the five hand-rolled matches.

## Last Updated

**Date**: 2026-09-02
**Updated By**: Claude Code / claude-opus-5
**Changes**: Widened §4 from "documentation" to a **uniform pass over every document
describing `foolish-ubca2`** — FOOP-16, FOOP-26, FOOP-36, FOOP-46, AGENTS.md's terminology
entries, and the crate's module docs — with the in-scope table and starting counts. The
delicate step is correcting any "constanic" that actually means *conclusive*: that is a
specification correction, not a wording tweak, and each goes to the human in one consolidated
list. Prior entry: created FOOP-56 — one predicate per NYES group (`is_preconstanic` with `is_nye` as
its alias, `is_constanic`, `is_constantew`, `is_conclusive`) on `foolish-ubca2`'s `NyesExt`,
plus qualification of all 134 bare uses of "settled" with the group each means. Extracted from
FOOP-36, which was carrying it as a skippable phase; scheduled **before** FOOP-36, which is
before FOOP-26.
