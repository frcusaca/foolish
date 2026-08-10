---
foop: D60
title: Generate to output.gen and make each stage compare against its predecessor only
author: Sisyphus <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-08-09
phase: meta
supersedes: []
begun: [ ] 
---

# FOOP-06: Generate to output.gen and make each stage compare against its predecessor only

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

## Abstract

Einmo evaluation currently writes directly into `output/`, and its three validation levels are
cumulative (`Verified ⊃ Checked ⊃ Output`), so every level re-evaluates the whole suite through
the FVM to reach an assertion about two files already on disk. This FOOP introduces a fourth,
**gitignored** stage — `output.gen/` — as the only directory evaluation writes to, promotes
`output/` to a **committed baseline**, and redefines each level to compare **against its
immediate predecessor only**:

```
output.gen  →  output  →  checked  →  verified
     (generation)  (reasonable) (reviewed)  (attested)
```

The result is a chain of three independent checks whose conjunction is today's `Verified`, each
separately reportable as a CI badge, and only the first of which writes anything. The other two
become pure file comparisons that need no FVM, no lock, and no build.

## Motivation

### The three badges

Reading the suite's status from `README.md` should be reading three badges:

| Badge | Claim |
|-------|-------|
| `output.gen ≡ output` | the build produces the same output as what is committed |
| `output ↔ checked` | all signatures check out, all sections identical |
| `checked ↔ verified` | all signatures check out, all sections identical |

By **transitivity** these three establish that the code produces human-attested results. Each
link is independently meaningful, and a red badge names *which* link broke — where today a single
cumulative failure only says "something in the chain."

### Four problems with the current model

**1. `output/` is both a work file and a committed artifact.** FOOP-64 §Retraction states plainly
that `output/` "is regenerated every run", yet 174 `.einmo` files under `output/` are tracked in
git. Every evaluation dirties the working tree with churn — the metadata header alone (`suite:`
path, `producer:` commit, `generated:` timestamp, einmo's own binary hash) changes on every run
even when the evaluated OUTPUT is byte-identical. **You never commit the work file.**

**2. "The FVM still produces what it produced yesterday" is never asserted.** Because `output/` is
overwritten before anything compares it, a change in evaluation cannot be detected on its own. It
surfaces only at `checked/`, fused with the much stronger claim "and the change is spec-correct."
The cheap, high-frequency check — *did behavior move at all?* — has nowhere to live.

**3. Cumulative levels force redundant evaluation.** `einmo_gate_verified` wants to assert
`checked ≡ verified`, a comparison of two files on disk. Because `Verified ⊃ Checked ⊃ Output` it
first re-evaluates the entire suite. Three gates therefore evaluate the same inputs three times to
produce byte-identical results.

**4. That redundancy is the sole cause of the gate race.** Commit `0a356f88` added a
`GATE_LOCK: Mutex<()>` to `foolish-ubca/src/ubca_snapshot_tester.rs` because all three gates write
the same `output/` directory and `cargo test` runs them in parallel; a catastrophe crumb dropped
by one gate is seen by another as "a previous run died." Only *writers* collide. Under this FOOP
just one check writes, and the other two — reading committed, immutable stages — parallelize
freely.

A further consequence of (3): verifying that a signed release artifact matches its reviewed
baseline requires a working Rust toolchain and a buildable interpreter, when it should need only
the two files and the public keys. That is a capability gap, not merely slowness.

## Specification

### The four stages

| Stage | Committed | Written by | A promotion into it asserts |
|-------|-----------|------------|------------------------------|
| `output.gen/` | **no** (gitignored) | evaluation | — (it is the work file) |
| `output/` | yes | `einmo promote output.gen to output` | "it ran without error, and the output is **reasonable**" — explicitly **not** a semantic or stylistic review |
| `checked/` | yes | `einmo promote output to checked` | statement-by-statement review against the in-force specification (`foop.md` §Promotion Review Gate) |
| `verified/` | yes | `einmo promote checked to verified --interactive` | human attestation under the reviewer's key |

`flagged/` continues to sit outside the escalation entirely (FOOP-64).

### Evaluation writes only to `output.gen/`

`EinmoSuite::evaluate_all` writes every generated `.einmo` to `output.gen/`. It never writes
`output/`. Catastrophe crumbs are dropped in `output.gen/`, so a crash leaves no trace in any
committed stage.

`output.gen/` is added to `.gitignore`. **The work file is never committed.**

### Each level compares against its immediate predecessor only

The levels stop being cumulative. Each states one link:

| Level | Asserts | Evaluates? | Writes? |
|-------|---------|-----------|---------|
| `Generation` | every input evaluates without runtime error, producing a self-verifying artifact in `output.gen/`; and `output.gen ≡ output` | **yes** | `output.gen/` only |
| `Checked` | `output ↔ checked`: files match up, signatures verify, sections identical | no | no |
| `Verified` | `checked ↔ verified`: files match up, signatures verify under the reviewer key, sections identical | no | no |

Two failure modes at the `Generation` level, reported distinctly:

- **runtime error** — an input failed to evaluate, or its artifact does not self-verify. The
  generation stage fails.
- **divergence** — generation succeeded but `output.gen/` differs from `output/`. The generation
  stage fails, and the remedy is either to fix the code or to *promote* (below).

Both are failures of the same stage; conflating them in the report would hide which occurred, so
`Problem` gains a variant distinguishing them.

### The new promotion: `output.gen` → `output`

```bash
einmo promote output.gen to output foolish-ubca/einmo_suite
```

This is a **deliberately weak claim**, and the wording matters: the promoter states that the run
completed and the output looks *reasonable* — a sanity check, not a semantic or stylistic review.
It is the moment a behavior change is consciously accepted as the new baseline, distinct from
(and much cheaper than) the `output → checked` promotion.

The distinction must be preserved everywhere it is documented. `output → checked` remains
governed by the **Promotion Review Gate** (`foop.md`): statement-by-statement justification
against the in-force specification, one named review sub-task per case. Nothing in this FOOP
weakens that gate; introducing a weaker-sounding sibling promotion is precisely why the two must
be kept verbally distinct.

### Badges

The three levels map one-to-one onto three CI jobs and three `README.md` badges. Because
`Checked` and `Verified` neither evaluate nor write, all three jobs may run **in parallel**, in
CI and in development.

### API shape

```rust
/// One link in the stage chain. Each level compares a stage against its
/// immediate predecessor; the levels are independent, not cumulative.
pub enum ValidationLevel {
    /// Evaluate every input into `output.gen/`, then require
    /// `output.gen ≡ output`. The only level that runs the evaluator.
    Generation,
    /// `output ↔ checked`. Reads two committed stages; does not evaluate.
    Checked,
    /// `checked ↔ verified`. Reads two committed stages; does not evaluate.
    Verified,
}
```

`compare()` (`einmo/src/compare.rs`) is already a free function reading two stages off disk and
evaluating nothing — the `Checked` and `Verified` levels are expressed in terms of it.

## FIR Impact

None.

## UBC Step Impact

None. This FOOP changes the test harness and its stage model; it does not touch the evaluator,
FIR, or any step rule. Any change in evaluated OUTPUT observed while implementing this FOOP is a
bug introduced by the implementation, not an expected consequence of it.

## Test Plan

- **einmo unit tests** (`einmo/src/einmo_suite.rs`, `einmo/src/cli.rs`): generation writes only to
  `output.gen/`; a runtime error fails the `Generation` level; `output.gen ≠ output` fails the
  `Generation` level with the divergence variant, distinct from the runtime-error variant;
  `promote output.gen to output` produces a correctly signed artifact; `Checked` and `Verified`
  levels perform no evaluation (assert the evaluator is not invoked) and write nothing.
- **`einmo::tests::parallel_and_serial_agree`** must continue to pass — it covers both
  `RawEvaluation`-producing paths.
- **UBCa gates** (`foolish-ubca/src/ubca_snapshot_tester.rs`): rename to match the three levels.
  Once `Checked` and `Verified` neither evaluate nor write, `GATE_LOCK` is needed only by the
  generation gate; the module docs added in `0a356f88` must be updated to say so rather than
  deleted, so the hazard stays recorded for whoever adds the next writing gate.
- **A parallel-gates test**: run the three gates concurrently and assert all pass — the regression
  test for the race that motivated the lock.
- **One-time migration** (see Open Questions): after `output.gen/` is populated, the existing 174
  `output/` artifacts must either be accepted as the baseline or regenerated. The suite must be
  green at all three levels before merge.

## Rejected Alternatives

### A. Do nothing — keep cumulative levels and the mutex
Works today: `cargo test --workspace` is green. But it leaves `output/` a committed work file that
dirties the tree on every run, leaves "behavior changed" undetectable except as part of a stronger
claim, keeps three redundant evaluations, and keeps attestation unverifiable without a working
FVM build. The mutex treats a symptom.

### B. Evaluate into a temp directory and copy back into `output/`
Considered and rejected in discussion. The copy-back is itself a race, merely a narrower one; and
it misdiagnoses the problem — the gates are not independent computations that collide on storage,
they are the same evaluation at three strictness levels. Per-gate temp dirs would evaluate the
identical suite three times to produce three copies of bytes that must be identical anyway.

### C. Keep levels cumulative but add a non-evaluating "verify only" mode
A narrower version of this FOOP: add a flag that skips evaluation. Rejected because it leaves two
ways to express each level and invites nonsensical combinations ("verify-only *and* require
`output↔checked`"). Naming the chain explicitly is simpler to reason about and maps directly onto
the badges.

### D. Make `output/` gitignored instead, and treat `checked/` as the first committed stage
Removes the churn without adding a stage. Rejected because it discards the cheap "did behavior
move at all?" check entirely — the very signal problem (2) is about. Every behavior change would
again surface only as a `checked/` divergence, fused with the correctness question.

## Open Questions

- **Migration of the existing 174 `output/` artifacts.** They are currently regenerated artifacts
  that happen to be tracked. Under this FOOP they become a baseline requiring a one-time
  `output.gen → output` promotion. Is today's committed content the accepted baseline, or should
  it be regenerated first? Note the tracked files carry stale `suite:` paths from the repository
  move (`/home/hcbusy/…` → `/storage1/human/hcbusy/…`), so a regeneration would change metadata
  headers even where evaluated OUTPUT is unchanged.
- **Does `Generation` failing on divergence belong in the same level as failing on a runtime
  error?** The spec above says yes (one stage, two distinct `Problem` variants) to keep the badge
  count at three. An alternative splits them into four badges.
- **`einmo.toml` signing config**: does `output.gen/` need a signing key at all, given it is never
  committed? Cheapest answer is to sign it with the computer key exactly as `output/` is signed
  today, so that `output.gen ≡ output` is a plain byte comparison including stamps — but that
  makes the comparison sensitive to metadata (timestamps, producer hash) that legitimately differs
  between runs. **This question is load-bearing and must be resolved before implementation**: it
  decides whether `output.gen ≡ output` compares whole files or only `MatchSections`.
- **Naming**: `output.gen/` vs `output.new/` — both appeared in discussion. `output.gen/` is used
  throughout this document.

## References

- FOOP-64 — *Migrate UBCa snapshot tests to a hierarchical einmo suite* (`status: complete`).
  Establishes the three-stage pipeline, the escalating `ValidationLevel`, the O/C/V requirement
  tables, retraction and its cascade. **This FOOP amends its stage model**: FOOP-64 §Retraction
  states `output/` "is regenerated every run", which this FOOP reverses.
- `foop.md` §"Promotion Review Gate" — the `output → checked` review discipline this FOOP must
  not dilute.
- `rust_instructions.md` §"Phase-by-phase testing discipline" — the three-stage contract as
  currently stated; needs updating for the fourth stage.
- Code: `einmo/src/einmo_suite.rs` (`ValidationLevel`, `evaluate_all`), `einmo/src/compare.rs`
  (`compare`, already non-evaluating), `einmo/src/cli.rs` (`promote`),
  `foolish-ubca/src/ubca_snapshot_tester.rs` (the three gates and `GATE_LOCK`).
- Commit `0a356f88` — added `GATE_LOCK`, documenting the race this FOOP removes the cause of.

## Last Updated

**Date**: 2026-08-09
**Updated By**: Claude Code / claude-opus-5
**Changes**: Initial draft. Specifies `output.gen/` as the gitignored generation stage, `output/`
as a committed baseline reached by a deliberately weak "reasonable output" promotion, and each
validation level comparing against its immediate predecessor only — yielding three independently
reportable badges whose transitivity establishes the whole claim.
