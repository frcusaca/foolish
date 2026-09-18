---
foop: D68
title: Retire UBCa — foolish-ubca2 becomes the implementation
author: Claude Code / claude-opus-5 (directed by the human)
status: Draft
type: Standards
created: 2026-09-16
phase: phase-4
supersedes: []
begun: [ ]
---

# FOOP-86: Retire UBCa — `foolish-ubca2` becomes the implementation

FOOP numbering is little-endian; the full rules live in `foop.md` at the repository root —
**read it before creating or editing a FOOP.** The `foop:` front-matter field here is the
big-endian sort key preceded by `D` (`foop: D68`, file `FOOP-86.md`, following FOOP-76's 67).

## Abstract

**`foolish-ubca2` becomes the implementation of Foolish, and the `foolish` binary ships it.**
Three older layers retire in the same act, because each exists only to serve one of the others
and none of them survives its neighbour.

Four deliverables, bundled:

1. **`foolish-cli` evaluates through `foolish-ubca2`.** Not by swapping one `use` line — that
   route runs ubca2's arena result back through the lossy `proto_to_core_fir` bridge into the
   old sequencer, inheriting exactly the losiness FOOP-36 built the new sequencer to escape
   (§1.2). The CLI instead calls `UbcaEvaluator::evaluate_arena` and renders with
   `Ubca2Sequencer::format`.
2. **`foolish-ubca` is removed** — 14 171 lines, the crate, its 178-case einmo suite, and its
   workspace membership.
3. **The old sequencer path is removed** — `foolish_core::FirSequencer` loses its last
   evaluator consumer and ubca2's `proto_to_core_fir` family (~650 lines, `fvm_storage.rs`
   3494–4151) goes. **`SequenceMode::Detailed` is KEPT and re-implemented arena-native** — the
   FIR-internal debugging view is worth having, and rewriting it to read `FVMStorage` directly
   is precisely what lets the bridge die (§3.2).
4. **`foolish-ubca2/einmo_suite` retires and `einmo_suite2` is renamed to `einmo_suite`** —
   one suite, under the canonical name, rendered by the only sequencer left.

**What this FOOP does NOT do: it does not make Project Euler 1 or fibonacci run.** Neither
evaluator runs them today and neither will when this merges (§0.5). This FOOP clears the ground
so that the work which finally makes them run — FOOP-26 and FOOP-46 — happens once, against one
evaluator, one sequencer and one suite, rather than twice against two of each.

The cost is stated plainly rather than minimized. **357 human-signed `verified/` artifacts are
discarded** — 178 with `foolish-ubca`, 179 with ubca2's old suite. **No test coverage is lost**:
all 178 UBCa inputs are already present in the surviving suite (§4.1, measured), so every program
UBCa tested is still tested. What is discarded is **attestation** — 178 signed confirmations that
a human checked *UBCa's* answer, which is the second opinion an independent implementation gave
(§0.4). The surviving 181 are `einmo_suite2`'s, human-attested 2026-09-07 at commit `aa22b82d`;
§4.3 records the rename mechanics that let them keep verifying.
**FOOP-36 deliberately kept every one of these layers** ("Nothing is removed", §6 there); this
FOOP is the one that reverses that decision, and §0.2 argues why it is now the right call.

## Execution order

**The human has set the order. FOOP-86 runs FIRST**, ahead of the remaining `foolish-ubca2`
work:

| # | Work | Status |
|---|------|--------|
| 1 | **FOOP-86** — retire UBCa (this FOOP) | this document |
| 2 | **[FOOP-96](FOOP-96.md)** — split `foolish-ubca2/src/fvm_storage.rs` (8 282 lines), one file per concern | Draft |
| 3 | **FOOP-26** (marks/UFM) ∥ **FOOP-46** (`BraneConcatOp`) — executed in parallel | Draft |

This **inverts** the ordering that Track 6 and FOOP-36 §"Why this should land before FOOP-26"
previously implied, and it is a deliberate choice. §0.3 states why it is the sensible order and
§0.4 states the one thing it genuinely costs.

### §0.3 Why first: the subsequent work carries less

The human's own statement of the rationale is the spine of this FOOP:

> "UBCA2 and einmo_suite2 are the most clean implementation of things that we want to keep. But
> they don't run euler-1 or fib yet. Thus the deprecations to make refactoring easier."

Two claims, and the order between them matters.

**The criterion for what stays is CLEANLINESS, not capability.** `foolish-ubca2` and
`einmo_suite2` are kept because they are the cleanest expression of what the project wants to
build on — the arena/`FVMStorage` model with enum dispatch, FOOP-36's sequencer that renders FIR
as Foolish that parses back in, and a 181-case human-attested corpus written in that rendering.
They are **not** kept because they do more. Neither evaluator runs Euler-1 or fibonacci (§0.5).
This FOOP does not argue that ubca2 is better at running programs; it argues that ubca2 is the
design worth carrying forward, and that the older layers are the ones worth not carrying.

**"Thus the deprecations to make refactoring easier" is the causal claim, and it is the whole
point.** The removals are not housekeeping done because the old code is unloved. They exist to
reduce what the work *after* this FOOP has to carry. Concretely, FOOP-26, FOOP-46 and the
refactoring FOOP between them stop paying:

- **Two evaluators → one.** FOOP-26 and FOOP-46 add semantics in one place, rather than adding
  them to ubca2 while keeping a second 14 171-line implementation green. `foolish-core` sits
  under both today, so any repair reaching into core moves UBCa's 178 baselines and trips
  AGENTS.md's non-regression rule. After this FOOP there is no sibling corpus to trip.
- **Two sequencers → one, with no bridge between them.** `proto_to_core_fir` must otherwise keep
  converting every FIR shape FOOP-26 and FOOP-46 invent — new mark statuses, a `BraneConcatOp`
  with two search personalities — into a `core_fir` vocabulary designed before any of it
  existed. That is maintenance paid on a representation nothing will read.
- **Three einmo suites → one.** `foolish-ubca/einmo_suite` (178), `foolish-ubca2/einmo_suite`
  (179) and `foolish-ubca2/einmo_suite2` (181). A semantic change today re-baselines and
  re-reviews across all three, one of them in the FIR-internal dump FOOP-36 exists to stop
  people reading. Afterwards it re-baselines once.
- **A `fvm_storage.rs` that is ~650 lines smaller** before the refactoring FOOP splits it — and
  smaller by a whole module (`core_fir_conversion`) rather than one the split must find a home
  for.

Keeping a second evaluator that *also* cannot run the target programs is pure cost. That is the
argument for this order, and it does not rest on a tradeoff.

### §0.4 What it costs: attestation and a second opinion — NOT coverage

One real cost, and it is easy to state imprecisely, so it is stated precisely.

**No test coverage is lost.** Every one of `foolish-ubca`'s 178 inputs already exists in the
surviving suite, at the same relative path — measured on `jia`, §4.1 gives the numbers. The
surviving suite holds those 178 plus three of ubca2's own. Every program UBCa tested continues
to be tested after this FOOP.

**What is lost is attestation, and the second opinion behind it.** Those 178 `verified/`
artifacts are signed confirmations that a human read and accepted **UBCa's** answer to each of
those programs. For that same corpus, `foolish-ubca` is an **independent implementation** — a
second answer to "what does this program mean?", reached from a different data structure
(`Rc<RefCell<dyn Fir>>` with vtable dispatch) by different code. Two implementations disagreeing
is a signal one suite cannot produce: it catches bugs that live in the **specification** rather
than in one evaluator's code. **Retiring `foolish-ubca` gives that up, permanently.**

The distinction matters because the two failure modes are different. Losing coverage would mean
a program nobody checks; that is not what happens here. Losing the second opinion means every
program is still checked, but only ever against one evaluator's reading of the spec — so a
misreading shared by the spec and the implementation goes unnoticed.

It was not a theoretical instrument. During FOOP-36 a `verified/`-backed baseline moving caught a
real defect — an agent's change to `get_display_name` silently broke FOOP-33's no-rename rule for
named creations.

Two things bound the loss honestly:

- **`verified/` is the stronger half of that protection, and it survives.** The defect above was
  caught by a baseline *moving*, which one suite detects exactly as well as two. The surviving
  suite has 181 cases with a human-signed `verified/` tier (attested 2026-09-07, `aa22b82d`),
  covering a superset of the inputs.
- **It is not an oracle for Euler-1 or fibonacci**, and must not be described as one (§0.5).
  What is given up is a second opinion on behavior that already works, not a reference for
  behavior the project is still trying to build.

**The human has accepted this cost, in these words: "discard human attestation for UBCa"**
(2026-09-16, resolving **Q1**). The 178 attestations go, deliberately and without an archive
(§2.3). This sub-section is the accounting, not a reopening of it.

### §0.5 Euler-1 and fibonacci: what is actually in the tree

This FOOP makes a claim about these two exercises, so the claim is stated at the precision the
evidence supports — measured on `jia` at `8c9043d8`.

**They are not present as einmo cases in either suite.** A repo-wide search for `*euler*` and
`*fib*` (excluding `target/` and worktrees) returns exactly one path:

```
future_exercise_inputs/project_euler/1.foo.disabled
future_exercise_inputs/project_euler/1.py
```

Three signals, all pointing the same way: the directory is named **`future_exercise_inputs`** and
belongs to no suite; the Foolish file carries a **`.disabled`** suffix and is deliberately not
run; and `1.py` is a Python reference (`print(sum) # prints 233168`) — so the expected answer is
known, but only from Python. Neither `foolish-ubca/einmo_suite/input/` nor
`foolish-ubca2/einmo_suite2/input/` has an `exercises/` tree at all; both hold `foop/`, `misc/`
and `regression/`.

**So "Euler-1 does not work" is not a failing baseline — the exercise was never wired up as a
runnable case, on either evaluator.** There is no NK baseline on `jia` to point at. The attempt
to make it run was made on the **FOOP-55 branch against `foolish-ubca`**, over 113 commits, and
was **abandoned unmerged**; results from that branch are not results from `jia`.

The conclusion is unchanged and better supported than a failure claim would be: **retiring
`foolish-ubca` gives up no working capability for these exercises, because no such capability
exists anywhere on `jia`.**

**Recorded for the FOOPs that follow** — not in this FOOP's scope, but the shape of the program
they must eventually make run. `1.foo.disabled` uses `'cmod` and `'ite` built over `'mod`, a
`loop` constructed by `self=<<#-1>>` recursion-without-self-reference, `$=`-sugared statements,
and `INTERNAL_`-prefixed helper names (the leading-underscore lexer workaround FOOP-55
documented). **Re-enabling it is a natural acceptance test for FOOP-26/46, and belongs to them.**

## Motivation

**The whole chain — FOOP-86, the refactoring FOOP, then FOOP-26 and FOOP-46 in parallel —
exists to make Project Euler 1 and fibonacci run for the first time.** That is the goal every
step serves. FOOP-86's role in it is the ground-clearing one: not to add the missing semantics,
but to ensure that when they are added they are added **once**, to one evaluator, rendered by
one sequencer, re-baselined against one suite (§0.3).

### §0.1 Why these four are one FOOP and not four

Each of the four deliverables, done alone, leaves a stranded layer behind it.

- **The CLI switch alone** leaves `foolish-ubca` in the workspace with no consumer: a
  14 171-line crate that must still compile, still pass 178 einmo cases, and still be kept
  green by every future FOOP — for nothing.
- **Deleting `foolish-ubca` alone** is impossible while the CLI depends on it
  (`foolish-cli/Cargo.toml:8`). The dependency is the thing that forces the order.
- **Removing the bridge alone** breaks both: `foolish-cli` reaches ubca2 only through
  `Evaluator::evaluate`, which *is* the bridge, and ubca2's old-suite gates
  (`ubca_snapshot_tester.rs`) render through it too (§3.2).
- **Retiring the old suite alone** removes the diff reference for a crate that is still
  shipping — the reviewer loses the old rendering while still needing it.

Read the other way: **the CLI is the only thing that makes `foolish-ubca2` real.** Nothing in
the workspace depends on it today — it is a member crate with zero consumers, exercised only by
its own tests. Until the binary ships it, "ubca2 is the implementation" is an assertion about a
library nobody calls.

### §0.2 FOOP-36 kept all of this on purpose — what changed

FOOP-36 §6 is titled **"Nothing is removed"**, and it meant it:

> `foolish_core::sequencer` is **not modified**. Not one line. … `Detailed` mode reaches the
> identical bytes it does today, via delegation.

and §"The suite is replaced, not edited":

> **This FOOP does everything except remove `einmo_suite`.** Retiring it is a separate act for
> the human to authorize, once `einmo_suite2` has been trusted for a while — and until then the
> old suite is useful precisely as the frozen reference that makes the new one auditable.

**FOOP-86 is that separate act, and the human has authorized it.** Three things changed since
FOOP-36 was written, and each weakens a specific reason FOOP-36 had for keeping a layer:

1. **`einmo_suite2` is no longer unproven.** It carries 181 inputs, 181 `checked/`, and **181
   human-attested `verified/`** artifacts (2026-09-07, commit `aa22b82d`). The "trusted for a
   while" condition FOOP-36 named has been met by the strongest evidence the project has.
2. **The reference has already done its job.** FOOP-36's reason for freezing `einmo_suite` was
   that judging a changed baseline needs the old output to hand. That review is complete — all
   179 cases were reviewed and promoted, and the two extra cases written. The diff the
   reference existed to support has been taken.
3. **The delegation is now cost, not safety.** `Detailed` mode's promise was that
   `foolish-ubca` "cannot regress by construction." Once `foolish-ubca` is gone there is
   nothing to protect, and the delegation is 650 lines of conversion maintained for a debugging
   mode with **no non-test caller** (§3.1).

## Specification

### §1 — `foolish-cli` evaluates through `foolish-ubca2`

#### §1.1 What is there now

`foolish-cli/src/main.rs` is 139 lines. It imports `foolish_ubca::UbcaEvaluator` (line 7) and
routes all four subcommands through one 5-line helper:

```rust
fn evaluate(source: &str) -> anyhow::Result<Vec<foolish_core::FirRef>> {
    UbcaEvaluator.evaluate(source)          // foolish_core::Evaluator
        .map_err(|e| anyhow::anyhow!("{}", e))
}
```

`cmd_run`, `cmd_step` and `cmd_repl` each `clone_steppable` the returned `FirRef` and render it
with `foolish_core::FirSequencer::format`. `cmd_compile` instead serializes with `fir_to_json`.

#### §1.2 The trap: the one-line switch is WRONG

`foolish-ubca2` already implements the same trait — `impl foolish_core::Evaluator for
UbcaEvaluator` in `foolish-ubca2/src/evaluator.rs:45` — and both crates name the type
`UbcaEvaluator`. So changing line 7 to `use foolish_ubca2::UbcaEvaluator;` compiles and runs.

**It is still wrong**, and this is the single most important implementation fact in this FOOP.
ubca2's trait impl is (`evaluator.rs:48-54`):

```rust
fn evaluate(&self, source: &str) -> Result<Vec<CoreFirRef>, String> {
    let (storage, firs) = self.evaluate_arena(source)?;
    Ok(firs.into_iter()
        .map(|fir| core_fir::fir_to_ref(crate::fvm_storage::proto_to_core_fir(&storage, fir)))
        .collect())
}
```

It converts out through `proto_to_core_fir` — the bridge FOOP-36 §1 documents as lossy:

> Rendering before `proto_to_core_fir` is load-bearing because that compatibility conversion
> does not preserve every surface-relevant arena field.

and FOOP-36 §FIR Impact names what is lost: **search direction and contexting**. So the one-line
switch would ship ubca2's evaluation rendered through the OLD sequencer, in the OLD FIR-internal
vocabulary, having dropped fields the new renderer needs — i.e. it would deliver none of
FOOP-36 and would make the CLI's output *worse* than the einmo suite's. It would also be the one
change that makes deliverable 3 impossible, since it creates a fresh non-test caller of the
bridge this FOOP removes.

#### §1.3 What the CLI must do instead

Use the arena boundary and ubca2's own sequencer — the same pair the einmo adapter already uses
(`ubca_snapshot_tester2.rs:30-36`, verified working against 181 human-attested baselines):

```rust
use foolish_ubca2::{SequenceMode, Ubca2Sequencer, UbcaEvaluator};

fn evaluate_arena(source: &str)
    -> anyhow::Result<(foolish_ubca2::fvm_storage::FVMStorage,
                       Vec<foolish_ubca2::fvm_storage::FirPointer>)> {
    UbcaEvaluator.evaluate_arena(source).map_err(|e| anyhow::anyhow!("{}", e))
}
```

with `cmd_run` / `cmd_step` / `cmd_repl` rendering each `FirPointer` via
`Ubca2Sequencer::format(&storage, fir, SequenceMode::Foolish)`.

`evaluate_arena` is already `pub` (`evaluator.rs:16`) and returns
`Result<(FVMStorage, Vec<FirPointer>), String>`. `Ubca2Sequencer::format` is already `pub`
(`sequencer.rs:142`) with signature `format(&FVMStorage, FirPointer, SequenceMode) -> String`.
Both are re-exported from `foolish-ubca2/src/lib.rs`. **No new public API is required.**

**Consequence worth naming: `foolish-cli run` changes its output format.** It moves from the
FIR-internal rendering to FOOP-36's Foolish rendering. That is the point of the FOOP, not a side
effect — and it is also what makes the README's einmo evaluator command (which drives
`foolish-cli run` over `foolish-ubca/einmo_suite`) need updating in lockstep (§4.4).

#### §1.4 `cmd_compile` is an open question, not an assumption

`cmd_compile` emits JSON through `foolish_core::fir_to_json`, which takes a `core_fir::Fir` —
i.e. it is shaped around the very representation §3 deletes. **Do not assume it can simply move
to the arena.** Three dispositions are possible and the choice is **Q2**:

- Give ubca2 an arena-native JSON serializer, and `fir_to_json` retires with the rest.
- Keep `compile` on `core_fir` by retaining a *minimal* conversion — which would contradict
  deliverable 3 and so must be argued explicitly, not slipped in.
- Retire `compile` from the CLI. It has no einmo case and no README example; its user base may
  be zero.

The plan's Phase 1 resolves this by reading, before any code is written.

### §2 — `foolish-ubca` is removed

#### §2.1 What goes

| Item | Size / count |
|---|---|
| `foolish-ubca/src/` (10 modules) | 14 171 lines |
| `foolish-ubca/einmo_suite/` | 178 inputs, 178 `checked/`, **178 `verified/`**, 178 `output/` |
| Workspace membership (`Cargo.toml:6`) | 1 line |
| `foolish-cli`'s dependency (`foolish-cli/Cargo.toml:8`) | 1 line |

#### §2.2 What must be true before it goes

1. `foolish-cli` no longer depends on it (§1) — this is the only non-test consumer in the
   workspace.
2. Nothing in `foolish-core` exists solely to serve it. `foolish-core` is shared and **stays**;
   this FOOP does not delete `foolish_core::FirSequencer` (§3.3).
3. The surviving suite is green at all three tiers under its new name (§4).

#### §2.3 Deleted outright — decided

**The human's decision, 2026-09-16: "discard human attestation for UBCa."** `foolish-ubca` is
deleted outright. No archive directory, no frozen copy, no required tag.

AGENTS.md calls `verified/` "the project's highest-trust correctness anchor," and deleting the
crate deletes 178 of those artifacts. §0.4 states exactly what that costs and the decision
accepts it. One factual observation, not a hedge: **git history retains the crate and every one
of its artifacts in all prior commits**, so deletion is not erasure — an explicit tag would make
recovery marginally more convenient, not more possible, and is therefore optional. Keeping a
frozen directory in-tree remains rejected on its own merits (§Rejected Alternatives C): in-tree
means compiled, which means kept green, which is the cost this FOOP exists to remove.

### §3 — The old sequencer path is removed

#### §3.1 `proto_to_core_fir` — measured, not estimated

A family of four functions in `fvm_storage.rs`'s `core_fir_conversion` module: the public
`proto_to_core_fir` (3494), `_sff_body` (3504), `_sff_operand` (3551), `_inner` (3599), running
to roughly line 4151 — **~650 lines with ~30 recursive call sites**.

**Non-test callers, exhaustively — there are two:**

| Site | Purpose | Disposition |
|---|---|---|
| `sequencer.rs:160` (`format_detailed`) | `SequenceMode::Detailed` | **rewritten arena-native**; the call goes, the mode stays (§3.2) |
| `evaluator.rs:52` | `impl foolish_core::Evaluator` | removed with the trait impl (§3.4) |

Everything else is inside `#[cfg(test)]`: five unit tests at `fvm_storage.rs:6876–6973`, one at
`8273`, and `sequencer.rs:1194`. Those tests test the bridge itself and go with it.

#### §3.2 `SequenceMode::Detailed` is KEPT and re-implemented over the arena — Q4 RESOLVED

**`Detailed` stays.** `SequenceMode` keeps both variants and `Ubca2Sequencer::format`'s
signature is unchanged. What changes is the implementation underneath it.

**Why keeping it is right, and why "no caller" was the wrong test.** `Detailed` is declared at
`sequencer.rs:22`, dispatched at `155`, and referenced outside the enum only by one test at
`sequencer.rs:1198` — so it has no non-test caller today. That fact is **not** evidence of low
value, and reasoning from it here would be circular: `Detailed` has no caller *because the CLI
does not expose it*, and this FOOP rewrites the CLI. "Currently unused" describes the present
wiring, not the feature's worth.

FOOP-36 §1 kept it deliberately, and the reason still holds: **einmo is a debugging tool as well
as an approval tool.** A FIR-internal view is exactly what a Foolisher wants when the
Foolish-mode rendering is *itself* the thing under suspicion — and after this FOOP, Foolish mode
is the only rendering the project has, which makes a second view of the same FIR more valuable
than before, not less.

**The bridge still dies — and the rewrite is what makes that possible.** These two are not in
tension; the second is the enabler of the first:

| | Today | After |
|---|---|---|
| `format_detailed` reads | `proto_to_core_fir(storage, fir)` → `foolish_core::FirSequencer::format` | `FVMStorage` / `FirSpec` / `FirCursor` directly |
| Depends on `core_fir` | yes — this is the bridge's last sequencer caller | **no** |
| Bridge can be deleted | no | **yes** |

Keeping `Detailed` **as it is** would mean keeping ~650 lines of lossy conversion and defeating
most of deliverable 3. Keeping `Detailed` **as an arena-native renderer** costs the bridge
nothing and removes its last sequencer caller. So the work is: write the arena-native dump,
prove it works, *then* delete the bridge — never a window in which neither exists (the plan
sequences it that way).

**Byte-compatibility with today's `Detailed` output is NOT required.** FOOP-36 §6 demanded it,
for one specific reason: so `foolish-ubca` could not regress by construction. **That constraint
dies with `foolish-ubca`, in this same FOOP.** There is no longer any consumer whose baselines
byte-compatibility would protect — `Detailed` has no einmo case, and the surviving suite renders
in Foolish mode.

**Its output will therefore differ, and should — it can render MORE.** The current `Detailed`
is filtered through a conversion that FOOP-36 §1 documents as not preserving every
surface-relevant arena field (search direction and contexting among them, §1.2). An arena-native
renderer reads the FIR as the evaluator actually holds it, so it can show what the bridge drops.
That is an improvement, and it is the strongest argument that this is a rewrite worth doing
rather than a port worth avoiding.

**What it must render.** The arena-native `Detailed` is a FIR-internal dump: per node, the
`FirSpec` variant, the NYES state by name, and the kind-specific fields — a search's `pattern`,
`anchored`, `forward`, `is_value_search` and `contexted`; an operator's kind and operand order;
a statement's name and line number; the `foolish_children` / `ubc_children` split, which is the
two-store structure FOOP-62 specifies and the single most useful thing to see when a brane
evaluates wrongly. Exact format is the implementer's, constrained only by being readable and
deterministic.

**Q4 is RESOLVED** and no longer open: keep the mode, rewrite the implementation.

#### §3.3 `foolish_core::FirSequencer` is NOT deleted

**Scope guard.** `foolish-core/src/sequencer.rs` is 814 lines and is `foolish-core`'s, not
UBCa's. After this FOOP it has no evaluator consumer, but `foolish-core` retains its own
`sequencer_tests.rs` and its API is public. **This FOOP removes the last *evaluator* path into
it; it does not remove it.** Deciding `foolish-core`'s fate is a separate question for a
separate FOOP, and an agent executing this plan that finds itself editing `foolish-core/src/`
should STOP and report (AGENTS.md: any decision to change a crate the FOOP promised not to
touch is never delegated).

The same guard covers `clone_steppable` and `fir_to_json`, both `foolish-core` API.

#### §3.4 `impl foolish_core::Evaluator for UbcaEvaluator` is removed

With the CLI on `evaluate_arena` (§1.3) and the old-suite adapter gone (§4.2), ubca2's trait
impl has no caller. It is the bridge's last non-test user, so it goes. `evaluate_arena` becomes
the crate's one production entry point, which is what `lib.rs`'s module docs already claim of
`evaluate` — those docs get corrected in the same commit.

**Note for §FIR Impact:** after this, nothing produces a `core_fir::Fir` *from an evaluation*.
`core_fir::Fir` survives as `foolish-core`'s own type, used by `foolish-core`'s own tests. What
changes is not the type but what it is produced *for* — nothing, from ubca2's side.

### §4 — One suite, under the canonical name

#### §4.1 The corpus, measured

| Suite | inputs | checked | **verified** | Fate |
|---|---|---|---|---|
| `foolish-ubca/einmo_suite` | 178 | 178 | **178** | deleted with the crate (§2) |
| `foolish-ubca2/einmo_suite` | 179 | 179 | **179** | **deleted** |
| `foolish-ubca2/einmo_suite2` | 181 | 181 | **181** | **renamed to `einmo_suite`** |

**357 human-signed `verified/` artifacts are destroyed — and NO test coverage is lost.** Those
are two different statements and the difference is the whole point of this sub-section.

**Coverage: measured, not assumed.** Comparing the two input trees by relative path, on `jia`:

| Comparison | Count |
|---|---|
| Inputs in `foolish-ubca/einmo_suite` **not** in `einmo_suite2` | **0** |
| Inputs in `einmo_suite2` **not** in `foolish-ubca/einmo_suite` | 3 |

The three extras are ubca2's own: `foop/16/comprehensive.foo`, `foop/36/comprehensive.foo`,
`foop/36/rendering_contract.foo`. So **178 + 3 = 181**, and the surviving suite is a strict
superset of both retiring suites' inputs. **Every program UBCa tested is still tested after this
FOOP**, under ubca2's answers, human-attested 2026-09-07. This parity is not a coincidence — it
is what FOOP-36's T10 test (`einmo_suite2_has_every_einmo_suite_input`, §4.5) exists to enforce,
and it has been green throughout.

**Attestation: this is what is actually discarded.** The 357 artifacts are signed records that a
human read and accepted a *particular evaluator's* answer:

- **179** (ubca2's old suite) — the same programs, the same evaluator, rendered by the **old
  FIR-internal sequencer**. What dies with them is the only signed record of that rendering.
  The surviving 181 attest the same evaluator's answers in the Foolish rendering.
- **178** (`foolish-ubca`) — the same programs, a **different, independently written
  evaluator**. What dies with them is the second opinion (§0.4) — the thing that could catch a
  bug living in the specification rather than in one implementation.

**Nothing needs porting**, because nothing is missing. What Phase 2's name-by-name diff is
genuinely for is the *other* question (**Q5**): whether, for any of the 178 shared inputs, the
two evaluators' attested answers actually **differ**. A difference would mean the two
implementations disagree about what a program means — which AGENTS.md calls a bug in at least
one of them — and it would be far better found now, while both signed corpora still exist, than
inferred later from a corpus that no longer has a counterpart.

#### §4.2 What is deleted alongside the old suite

`foolish-ubca2/src/ubca_snapshot_tester.rs` (231 lines) holds the old suite's three gates —
`einmo_gate_output`, `einmo_gate_checked`, `einmo_gate_verified` — and its adapter
(`ubca_snapshot_tester.rs:85-97`) evaluates via `foolish_core::Evaluator` + `FirSequencer`.

**That is the same trap as §1.2, already in the tree**: ubca2's old-suite gates run ubca2's
evaluator *through the lossy bridge into the old sequencer*. This is why deliverables 3 and 4
are inseparable — deleting the bridge deletes this file's adapter, and deleting this file is
what makes deleting the bridge possible.

#### §4.3 Rename mechanics — DETERMINED, with evidence

The brief flagged this as possibly unanswerable. **It was answerable, and the answer is that a
directory rename is signature-safe.** Four findings, each traced to source:

1. **Signatures are verified against the stored file's own bytes, which a rename does not
   touch.** `einmo/src/verify.rs:39-45` — `verify_bytes` parses the file and re-checks the
   stamp chain against `raw_signed_prefix(bytes, &file)`, i.e. the actual bytes read from disk.
   `git mv` changes no byte of any `.einmo` file, so every stamp that verifies before the rename
   verifies after it.

2. **The suite path IS inside the signed prefix — and that does not matter.** The header carries
   `suite: <absolute path>` (`format.rs:143`), written from `config.suite_name()`, which
   `TestConfig::new` defaults to `work_dir.to_string_lossy()` (`config.rs:178-179`). It is
   therefore part of `prior_bytes` and is signed. **The empirical proof that this is harmless is
   already in the tree:** every artifact in `einmo_suite2/verified/` carries
   `suite: /yolo/foolish_worktrees/foop-36-foolish-rendering-sequencer/foolish-ubca2/einmo_suite2`
   — a worktree path that no longer exists — and `einmo_suite2_gate_verified` passes today. A
   stale absolute path in a signed header is already the normal, working condition.

3. **Correspondence comparison excludes metadata entirely.** `einmo_suite.rs:605-607`:
   "compares only the configured sections — STAMPS and metadata are excluded by design". The
   default `MatchSections::InputOutput` (`config.rs:63-64`) compares INPUT + OUTPUT only. So a
   freshly written `output/` artifact carrying the NEW suite path still corresponds to a
   `checked/` artifact carrying the OLD one. **This is INDEX's recorded property P6** — and P6's
   own warning applies: nobody may "fix" compare into byte-exactness, or this breaks.

4. **`einmo.toml` needs no edit — and travels with the directory.** Neither suite's toml names
   its own directory. `foolish-ubca2/einmo_suite2/einmo.toml` sets
   `[signing.checked] passphrase = "foolish-ubca2-suite2"`; the retiring
   `foolish-ubca2/einmo_suite/einmo.toml` sets `"foolish-ubca"`. Because the file moves *with*
   the artifacts it signed, the passphrase stays matched to them. **The one hazard is the
   opposite of an edit: do NOT "tidy" the surviving passphrase to `"foolish-ubca2"` or
   `"foolish-ubca"` after the rename** — the 181 `checked/` stamps were made under
   `"foolish-ubca2-suite2"` and changing the string invalidates all of them. It reads like
   leftover naming after the rename, which is exactly why the plan names it as a stop condition.
   Both suites already use einmo's default `①` (U+2460) + LF separator (confirmed in the toml
   comments and in a real artifact's `#einmo 1 … separator=①\n` header line), so **no separator
   change is involved** — the `!!` separator belongs to `foolish-ubca/einmo_suite`, which is
   being deleted, not renamed.

**Residual uncertainty, stated:** the above is read from source and corroborated by the live
stale-path evidence, but it has not been *executed*. The plan therefore makes the rename its own
phase whose first action after `git mv` is to run all three gates unchanged, before any other
edit — so that if this analysis is wrong, it is wrong in isolation and immediately visible.

#### §4.4 The code and docs that name the suites

The rename is not just a `git mv`; these must move with it:

| Location | Change |
|---|---|
| `ubca_snapshot_tester2.rs:7-9` (`einmo_suite2_dir`) | join `"einmo_suite"` |
| `ubca_snapshot_tester2.rs:75, 81, 103` (gate fns) | `einmo_suite2_gate_*` → `einmo_gate_*` |
| `ubca_snapshot_tester2.rs` → renamed file | `ubca_snapshot_tester.rs` (the old one having been deleted) |
| `lib.rs:41-44` (`mod` declarations) | one `#[cfg(test)] mod ubca_snapshot_tester;` |
| `README.md` §"Running specific tests" | every `foolish-ubca/einmo_suite` path and the `foolish-cli run` evaluator command |
| AGENTS.md §"Approval Tests (einmo)" | suite paths and gate command |

**The gate rename matters beyond tidiness:** `cargo test -p foolish-ubca --lib --
einmo_gate_checked` is the command AGENTS.md, `foop.md`, README and every open FOOP plan name as
*the* einmo gate. After this FOOP the crate is `foolish-ubca2` and the test name must still be
`einmo_gate_checked`, so that the substring every document already uses keeps selecting the real
gate. **Q6** asks whether AGENTS.md/`foop.md` updates belong in this FOOP or a follow-on; the
recommendation is that README and AGENTS.md move here (they would otherwise be actively wrong
the moment this merges) and that the many FOOP plans' `-p foolish-ubca` invocations are left
alone as historical record.

#### §4.5 Two tests read `einmo_suite/` directly and must be dealt with

Both live in `ubca_snapshot_tester2.rs` and both break on deletion — **loudly, which is the good
case:**

- **`einmo_suite2_has_every_einmo_suite_input`** (T10, line 166). Walks
  `foolish-ubca2/einmo_suite/input` and asserts the survivor is a superset. With the directory
  gone, `foo_inputs_under` returns an **empty set** (its inner `walk` swallows the `read_dir`
  error and returns), so the `missing.is_empty()` assertion passes **vacuously** — but the next
  line, `assert_eq!(original.len(), 179, …)`, fails with "0 inputs found". **The test dies loud,
  not silent.** Its purpose — "the suite that replaces it does not quietly test less" — is
  discharged the moment the replacement is complete, so it is **deleted**, and its final green
  run *before* the deletion is the record that parity held (the plan makes that ordering
  explicit).
- **`t12_report_value_differences_old_vs_new`** (line 305). Reads `einmo_suite/checked` against
  `einmo_suite2/output` and reports value-level differences. It is FOOP-36's old-vs-new review
  instrument; with no old side it compares nothing. **Deleted**, for the same reason §0.2(2)
  gives: the diff it supported has been taken.

Deleting a test is exactly the move AGENTS.md's triage discipline is suspicious of, so the
justification is recorded per test rather than in bulk: each is an *instrument of the migration*
whose referent is being removed, not a correctness check whose subject survives.

### §5 — Scope boundaries: what this FOOP does NOT change

Two exclusions, stated here so neither is inferred from the removals around them.

**The crate keeps its name.** `foolish-ubca2` remains `foolish-ubca2`, and its public types keep
their current names — `Ubca2Sequencer`, `SequenceMode`, `UbcaEvaluator`. This FOOP performs **no
renaming of the crate or of its public types**, here or in a follow-on. Human decision,
2026-09-16. Two reasons, and the `2` suffix is **vestigial and acknowledged as such**. First,
**consistency with the FOOP corpus**: roughly 18 files under `docs/` refer to the crate and its
types by these names, several of them Complete FOOPs that AGENTS.md requires be left as written —
renaming would either falsify that record or leave it stale. Second, in the human's own framing,
**it conveys a sense of progress**: the name records that this was the second attempt and that
the second attempt is the one that won. "This is a purely human thing, feeling accomplished."
That is a legitimate reason, not a concession.

To be precise about what that excludes, since this FOOP does rename two things: the exclusion
covers the **crate** and its **public types**. It does not touch deliverable 4's
`einmo_suite2/` → `einmo_suite/` (§4.3), nor the consequent
`ubca_snapshot_tester2.rs` → `ubca_snapshot_tester.rs` (§4.4). Those concern the *suite* and its
test file, not the crate's identity, and both proceed as specified.

**`foolish-core` is not restructured.** §3.3's scope guard: this FOOP removes the last
*evaluator* path into `foolish_core::FirSequencer`; it does not remove or reorganize
`foolish-core`. An agent that finds itself editing `foolish-core/src/` stops and reports.

## FIR Impact

**None.** No new FIR variant, no `FirSpec` arm, no NYES state, no state-machine change, no
serialization change to the arena.

What changes is not a type but a **purpose**. `core_fir::Fir` is not deleted — it remains
`foolish-core`'s type, with `foolish-core`'s own tests. But after this FOOP **no evaluation
produces one**: `proto_to_core_fir` was the only path from ubca2's arena into that
representation (§3.1) and `foolish-ubca` was the only other producer. `core_fir::Fir` ceases to
be an evaluation result and becomes a `foolish-core`-internal FIR type awaiting its own
disposition FOOP (§3.3).

## UBC Step Impact

**None.** No step rule changes, no evaluation order changes, no NYES transition changes. Step
counts in einmo output are unaffected — and that is a **testable claim**: the surviving suite's
181 `checked/` baselines record step counts, and they must not move. Any movement is a bug this
FOOP introduced, never a baseline to promote over.

## Test Plan

The crux: **after the removals, the one remaining evaluator must still pass everything.** There
is no second implementation to fall back on and no second rendering to diff against, so the
tests below have to carry alone what two suites carried before.

### T1 — The surviving suite, all three tiers, under the new name

```
cargo test -p foolish-ubca2 --lib -- einmo_gate_output
cargo test -p foolish-ubca2 --lib -- einmo_gate_checked
cargo test -p foolish-ubca2 --lib -- einmo_gate_verified
```

All 181 cases, `output` ↔ `checked` ↔ `verified`. **`einmo_gate_verified` must pass without
promotion and without `#[ignore]`.** It is the whole point: if the rename were signature-hostile
(§4.3), this is the test that says so. AGENTS.md forbids an agent marking it `#[ignore]` under
any justification, and `ubca_snapshot_tester2.rs:94-100` already carries that instruction in its
doc comment — which must survive the file rename.

### T2 — No promotion happens in this FOOP

**This FOOP promotes nothing.** It writes no new einmo case and changes no OUTPUT. Every
baseline that exists must stay byte-identical in its INPUT and OUTPUT sections.

Stated as a rule for the executing agent: **if `einmo_gate_checked` goes red at any point, that
is a regression this FOOP introduced.** It is never remedied by `einmo promote`. Per
`rust_instructions.md` §"Phase-by-phase testing discipline" and AGENTS.md's non-regression hard
rule, the fix is always the code. The plan therefore installs **no Promotion Review Gate** —
there is nothing to promote, and a gate naming no cases is not a gate (`foop.md`).

### T3 — CLI behavior

`foolish-cli` has no test module today; this FOOP adds one, because the CLI stops being a
5-line pass-through and becomes the thing that decides what a Foolisher sees.

- **T3a** — `run` on a small program renders Foolish, not FIR internals: assert the output
  contains no `?(pattern=`, no `Op`, no bare NYES token.
- **T3b** — the CLI's rendering **agrees with the einmo adapter's** for the same source. Both
  call `evaluate_arena` + `Ubca2Sequencer::format(.., Foolish)`; a divergence means the CLI
  grew its own path. This is the test that pins §1.3.
- **T3c** — `step` and `repl` render through the same path (no second sequencer call site).
- **T3d** — whatever Q2 decides for `compile` gets a test asserting that decision.

### T4 — Record the exercise baseline for the FOOPs that follow

§0.5 establishes that Euler-1 and fibonacci are **not einmo cases on either evaluator** — the
only artifact on `jia` is `future_exercise_inputs/project_euler/1.foo.disabled` plus a Python
reference. There is therefore nothing for this FOOP to regress and nothing for it to fix.

What is worth doing, once, is cheap and useful to FOOP-26/46: **run `1.foo.disabled` under the
new CLI and write down exactly what happens** — the rendered output, the alarms, the step count,
or the failure mode. Two reasons. First, it confirms §0.5's claim against the actual binary
rather than against a file listing. Second, the recorded result is the **before** picture that
FOOP-26 and FOOP-46 will be measured against; it is far more useful written into this plan than
rediscovered later.

**This is a recording task, not an acceptance criterion.** Whatever it prints, FOOP-86 is not
blocked by it, and the file stays `.disabled` — re-enabling it belongs to FOOP-26/46 (§0.5).

### T4b — The arena-native `Detailed` renderer (§3.2)

`Detailed` is rewritten, so it needs tests of its own — it had exactly one before, and that one
asserted delegation to the very code being deleted.

- **T4b-i** — `Detailed` renders without panicking for every FIR kind. The cheapest thorough
  form is to run it over the whole surviving corpus: every input, rendered in `Detailed`, must
  produce output. This is the `Detailed` analogue of the existing corpus-wide Foolish-rendering
  property test.
- **T4b-ii** — `Detailed` shows what the bridge dropped. Assert on a case with a contexted or
  forward search that the output names the search's direction and contexting — fields §1.2
  records `proto_to_core_fir` as not preserving. **This is the test that proves the rewrite was
  worth doing** rather than being a port of the old view.
- **T4b-iii** — `Detailed` and `Foolish` are genuinely different renderings of the same FIR
  (a guard against the mode silently collapsing to one implementation).
- **T4b-iv** — `Detailed` is deterministic: rendering the same settled FIR twice is identical.

**No byte-compatibility test against the old `Detailed`**, deliberately — §3.2 records that the
constraint FOOP-36 §6 imposed dies with `foolish-ubca`, and the new output is expected to differ.

### T5 — Nothing references the removed crate

- `cargo build --workspace` and `cargo test --workspace` green with `foolish-ubca` absent from
  `Cargo.toml`.
- `grep -rn "foolish.ubca\b"` over `*.rs`, `Cargo.toml`, `*.sh` and CI returns only historical
  prose in `docs/`.
- `cargo clippy --workspace -- -D warnings`. **Known hazard:** MEMORY records four pre-existing
  clippy errors in `foolish-core/src/sequencer.rs` that break the workspace `-D warnings` gate.
  This FOOP does not fix them (that is `foolish-core`, §3.3's scope guard) — but it must not be
  *blamed* for them either, so the plan records the pre-existing count before touching anything.

### T6 — Workspace-wide, the standing gate

`cargo test --workspace` — currently **791 tests** passing. After the removals the count drops
(foolish-ubca's tests leave with it, as do the bridge's ~7 and the two instruments of §4.5).
**The plan records the before and after counts and accounts for the difference**, so "fewer
tests pass" is never mistaken for "tests were lost."

## Plan of Execution for Plan

Per AGENTS.md, phases are assigned by complexity, not the FOOP sized to one model. This FOOP
has an unusual shape: **the removals are mechanical, but the decision that each removal is safe
is judgment**, and the two are easy to confuse because a green suite follows either way.

| Phase | Character | Needs |
|---|---|---|
| **0** — Baselines; confirm Q2/Q6/Q7 | Q1, Q3, Q4 already resolved by the human; three lesser questions remain | **Larger model** — it records the resolved decisions and asks only what is still open |
| **1** — CLI switch (§1) | Read `fir_to_json`'s shape, resolve Q2, write the new evaluate path | **Larger model.** §1.2 is a trap a small model would walk into — the wrong version compiles and passes |
| **2** — Answer comparison (§4.1, Q5) | Confirm input parity one last time, then compare the two evaluators' attested ANSWERS | **Larger model.** A disagreement is a semantic finding, not a diff |
| **3** — Suite rename (§4.3, §4.4) | `git mv`, then re-point names. High risk, fully specified | **Smaller model** — the target is fixed (three gates green) and the stop condition is named |
| **4a** — Arena-native `Detailed` (§3.2) | Write a FIR-internal dump over `FVMStorage` | **Larger model.** New code against the arena API, with no fixed target to match |
| **4b** — Bridge removal (§3.1, §3.4) | Delete named functions; two call sites, one already repointed by 4a | **Smaller model**; the call sites are enumerated in §3.1 |
| **5** — Crate removal (§2) | Delete a directory and two Cargo lines | **Smaller model** |
| **6** — CLI tests + T4 (§T3, §T4) | Write tests; record `1.foo.disabled`'s current behavior | **Larger model** for T3b's agreement test, smaller for the rest |
| **7** — Docs (§4.4) | README + AGENTS.md paths and commands | **Smaller model** |
| **8** — Merge | Mechanical, with a human STOP | **Smaller model** |

### What makes the small-model phases safe

1. **Facts inline, not referenced.** The plan carries the exact line numbers (§3.1's two call
   sites, §4.4's table, §4.5's two tests), the exact signatures of `evaluate_arena` and
   `Ubca2Sequencer::format`, and the three gate commands — all marked *verify, don't re-derive*.
2. **A fixed target per phase.** Phase 3's target is "three gates green after `git mv`, with no
   other edit." Phase 4's is "workspace compiles with these two call sites gone." Neither asks
   an agent to judge whether an output is *right*, only whether it *matches*.
3. **Named stop conditions.** Phase 3: *if any gate goes red after the `git mv`, STOP* — §4.3's
   analysis is then wrong and that is a finding, not a thing to work around. Phase 3 also names
   the passphrase trap explicitly. Phase 4: *if removing the bridge requires editing
   `foolish-core`, STOP* (§3.3). Phase 5: *if anything outside `foolish-ubca/` must change to
   delete it, STOP.*

### What must not be delegated, at any size

- **Reopening Q1, Q3 or Q4.** All three are settled by the human (2026-09-16). An agent that
  finds itself re-arguing whether the attestations should be discarded, whether an archive is
  needed, or whether `Detailed` is worth keeping has exceeded its remit — implement the
  decisions, and raise new *evidence* to the human if any appears, not new opinions.
- **Any `einmo promote`.** There is nothing to promote (T2); an agent reaching for `promote` in
  this FOOP has mistaken a regression for a stale baseline.
- **Marking `einmo_gate_verified` `#[ignore]`** — never an agent's call (AGENTS.md), and this
  FOOP is precisely the situation that would tempt it.
- **Any edit to `foolish-core/src/`** (§3.3).
- **Any decision arising from Q5** if the two evaluators' attested answers disagree — that is a
  semantic finding about the language, reported to the human, never resolved by an agent
  choosing which evaluator was right.

## Rejected Alternatives

### A. Do these as four separate FOOPs

The natural instinct, and wrong here: the four deliverables are **mutually blocking**, not
merely related (§0.1). The CLI cannot leave `foolish-ubca` until it can reach ubca2 without the
bridge; the bridge cannot go while the old-suite gates render through it; the old suite cannot
go while it is the reference for a shipping crate; the crate cannot go while the CLI depends on
it. Four FOOPs would each open with "blocked on the other three." The human asked for them "all
together in one go" for this reason.

The plan keeps the *benefit* of separation without the deadlock: each deliverable is its own
phase with its own green gate, so the tree is green at every step and any phase can be the one
that stops.

### B. Keep `SequenceMode::Detailed` and the bridge indefinitely

`Detailed` costs ~650 lines of conversion to serve **zero non-test callers** (§3.2), and the
cost is not static: every FIR shape FOOP-26 and FOOP-46 add must be taught to a conversion
targeting a vocabulary that predates them (§0.3). Keeping it also keeps alive the §1.2 trap —
as long as `Evaluator::evaluate` exists, the next person wiring something to ubca2 reaches for
the trait and silently gets the lossy path.

**Note what is and is not rejected here.** Keeping the **bridge** is rejected. Keeping
**`Detailed`** is not — Q4 is resolved in favour of keeping it and rewriting it arena-native
(§3.2), which is the better version of this idea and is **in scope for this FOOP**, not a
follow-on. The two are separable precisely because the rewrite removes the bridge's last
sequencer caller.

### C. Keep `foolish-ubca` in-tree as a frozen reference forever

Superficially the conservative choice, and it is what FOOP-36 §6 did — correctly, *then*. It
does not stay cheap. In-tree means compiled, which means every `cargo test --workspace` runs its
178 cases, every `foolish-core` change must not move them, and every future FOOP inherits a
second implementation to keep green. "Frozen" is not a state a compiled crate can be in.

The recoverable form of the same instinct is §2.3's tag, which costs one command and no
maintenance. That is why §0.4 recommends **(c)**: archive, don't retain.

### D. Deprecate the CLI instead of upgrading it

**Explicitly rejected by the human.** Recorded here with its reason, because the reason is the
FOOP's thesis: **the CLI is the only thing that makes `foolish-ubca2` real.** Nothing in the
workspace depends on ubca2 today (§0.1); it is a member crate with zero consumers. Retiring the
binary instead of repointing it would leave the project with an evaluator that no artifact ships
and no user can invoke — "the implementation" in name only. The CLI is also the einmo evaluator
driver in README's documented commands, so deprecating it would strand the documented review
workflow too.

### E. Do nothing

Two implementations, two sequencers, three einmo suites, one lossy bridge between them, and one
of the two evaluators unreachable from the binary. Every FOOP that follows pays that tax, and
FOOP-26 and FOOP-46 — which change what programs *mean* — pay it twice, in two renderings.
Rejected, but it names the alternative honestly: doing nothing keeps `foolish-ubca` as an
independent cross-check over its 178 cases (§0.4), which is a real thing to give up.

## Open Questions

**Three of the seven are settled** by the human on 2026-09-16 (Q1, Q3, Q4) and are kept here,
marked RESOLVED, as the record of what was decided and why. **Q2, Q5, Q6 and Q7 remain open**,
and none of them blocks starting work.

- **Q1 — RESOLVED 2026-09-16 (the human): "discard human attestation for UBCa."** The 178
  human-signed `verified/` attestations for `foolish-ubca` are discarded deliberately. §0.4
  states what that gives up — an independent implementation's second opinion — and that
  statement stands as the honest accounting of the cost; it is accepted, not mitigated. **No
  longer an open question.**
- **Q2 — What becomes of `cmd_compile`?** It emits `core_fir`-shaped JSON via `fir_to_json`
  (§1.4). Arena-native serializer, retain a minimal conversion, or retire the subcommand?
  Resolved by reading in Phase 1; the human is asked only if the answer is "retain a
  conversion", since that contradicts deliverable 3.
- **Q3 — RESOLVED 2026-09-16 (the human): deleted outright, no archive.** The human asked for
  discard, not preservation. Recorded as a plain fact rather than a safety net: **git history
  retains the crate and its 178 artifacts in every prior commit**, so deletion is not erasure
  and an explicit tag is optional rather than protective. **No longer an open question.**
- **Q4 — RESOLVED 2026-09-16 (the human): `SequenceMode::Detailed` is KEPT and re-implemented
  arena-native.** The draft proposed dropping it on the grounds that it has no non-test caller;
  the human's answer is that "`SequenceMode::Detailed` can be very useful… should be
  implemented," and the "no caller" reasoning was circular — it has none *because the CLI does
  not expose it*, and this FOOP rewrites the CLI. `SequenceMode` keeps both variants;
  `format_detailed` is rewritten over `FVMStorage`/`FirSpec`/`FirCursor`; the bridge still dies,
  and the rewrite is what makes that possible. Byte-compatibility with the old output is not
  required and the new output can render more. See §3.2 — **no longer an open question.**
- **Q5 — REFRAMED: do the two evaluators' attested answers AGREE on the 178 shared inputs?**
  The draft asked whether any UBCa cases should be "ported first." **That question is answered
  and retired: nothing needs porting.** All 178 UBCa inputs are already in the surviving suite
  (§4.1, measured — the difference is 0), so no coverage is lost. The question worth asking in
  its place is about *answers*, not inputs: for those 178 shared programs, does UBCa's signed
  baseline mean the same thing as ubca2's? A disagreement would mean the two implementations
  differ about what a program means — a bug in at least one of them (AGENTS.md) — and now is the
  last moment both signed corpora exist to compare. Phase 2 produces the comparison. **Not
  blocking**, and a clean result is the expected one; but if a genuine disagreement surfaces,
  STOP and report it, because it is information that cannot be recovered after the merge.
- **Q6 — Do AGENTS.md and `foop.md` updates belong in this FOOP?** §4.4's table.
  **Recommendation: README and AGENTS.md yes** (they are wrong the moment this merges);
  `foop.md` and historical FOOP plans no.
- **Q7 — Is the `zweimomo` crate affected?** AGENTS.md lists it as a top-level crate, but there
  is **no `zweimomo` directory in the tree** and it is not a workspace member. It appears to
  have left with einmo's extraction to its own repository (cf. FOOP-25's note). If so,
  AGENTS.md's crate list is stale and should be corrected — but that is a documentation finding
  raised here, not a deliverable claimed here.

## Proposed Next Steps

### N1. FOOP-96 — split `fvm_storage.rs` (step 2 of §Execution order)

**[FOOP-96](FOOP-96.md) — "Split `fvm_storage.rs` — one file per concern"** now exists (Draft,
2026-09-16) and occupies step 2. FOOP-86 does not perform any of it.

The relationship runs one way and is worth stating: **FOOP-86 makes FOOP-96 smaller.** It removes
~650 lines of `fvm_storage.rs` and removes the `core_fir_conversion` module **entirely** (§3.1),
so FOOP-96 neither splits that code nor has to find a home for a module about to be deleted.
Running them the other way round would have FOOP-96 carefully rehome code FOOP-86 then deletes.

### N2. `foolish-core`'s disposition

After this FOOP, `foolish_core::FirSequencer` (814 lines), `core_fir::Fir`, `clone_steppable`
and `fir_to_json` have no evaluator consumer (§3.3). Whether `foolish-core` shrinks to the
parser/AST boundary it actually still serves is a real question, and a separate FOOP's. It also
interacts with MEMORY's note about four pre-existing clippy errors in
`foolish-core/src/sequencer.rs` that break the workspace `-D warnings` gate (§T5) — which that
FOOP would be the natural place to fix.

### N3. Richer arena-native `Detailed` output

The arena-native `Detailed` renderer is delivered **by this FOOP** (§3.2), not deferred. What
remains open-ended is how far it goes: freed from the lossy conversion, it *can* show more than
the old one did, and §3.2 names a reasonable floor (FirSpec variant, NYES, search fields,
operator kinds, the `foolish_children`/`ubc_children` split). Extending it further — brane
provenance, mark status once FOOP-26 lands, arena pointer identity for spotting sharing — is
natural follow-on work and pairs with the `foolish-debugging` skill's unit-test-driven method.
Non-blocking.

## References

- **FOOP-36** ([FOOP-36.md](FOOP-36.md)) — the Foolish-rendering sequencer. §1 (two modes,
  `Detailed`'s delegation, the lossiness of `proto_to_core_fir`), §2/§2.1 (round-trip
  properties), §6 ("Nothing is removed"), §"The suite is replaced, not edited". **This FOOP
  reverses §6's decision**; §0.2 argues why.
- **FOOP-16** ([FOOP-16.md](FOOP-16.md)) — created `foolish-ubca2`; the two-crate relationship
  ("two independent implementations of the same Foolish evaluator", `foolish-ubca2/src/lib.rs`).
- **FOOP-26** ([FOOP-26.md](FOOP-26.md)) and **FOOP-46** ([FOOP-46.md](FOOP-46.md)) — marks/UFM
  and `BraneConcatOp`. Executed in parallel **after** this FOOP and the refactoring FOOP
  (§Execution order). Making Euler-1 and fibonacci run is **their** deliverable, not this
  FOOP's (§0.5); re-enabling `future_exercise_inputs/project_euler/1.foo.disabled` is their
  acceptance test.
- **FOOP-55** — the abandoned, unmerged branch that attempted Euler-1 against `foolish-ubca`
  over 113 commits. Its results are not results on `jia` (§0.5).
- **FOOP-92** / **FOOP-64** — einmo's signed-snapshot design and the suite hierarchy.
- AGENTS.md §"Development Rules" (non-regression hard rule; Verified-tier rules),
  §"Approval Tests (einmo)"; `foop.md` §"Promotion Review Gate";
  `rust_instructions.md` §"Phase-by-phase testing discipline".
- Code: `foolish-cli/src/main.rs` (139 lines); `foolish-ubca2/src/evaluator.rs:16,45-54`;
  `foolish-ubca2/src/sequencer.rs:22,142,155,160`;
  `foolish-ubca2/src/fvm_storage.rs:3494-4151` (`core_fir_conversion`);
  `foolish-ubca2/src/ubca_snapshot_tester.rs` (old gates);
  `foolish-ubca2/src/ubca_snapshot_tester2.rs:7,75,81,103,166,305`;
  `einmo/src/verify.rs:39-45`; `einmo/src/einmo_suite.rs:605-607`; `einmo/src/config.rs:63,178`.

## Last Updated

**Date**: 2026-09-16
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created FOOP-86 — retire UBCa; `foolish-ubca2` becomes the implementation. Four
bundled deliverables: the CLI onto ubca2 via `evaluate_arena` + `Ubca2Sequencer` (**not** the
lossy `Evaluator`-trait one-liner, §1.2), removal of `foolish-ubca`, removal of the
`proto_to_core_fir` bridge, and retirement of `foolish-ubca2/einmo_suite` with `einmo_suite2`
renamed onto the canonical name.

**Three human decisions recorded 2026-09-16.** **Q4**: `SequenceMode::Detailed` is **KEPT and
re-implemented arena-native** — the draft proposed dropping it for having no caller, which was
circular reasoning (it has none because the CLI does not expose it, and this FOOP rewrites the
CLI); the rewrite over `FVMStorage`/`FirSpec`/`FirCursor` is what *lets* the bridge die, so the
two are enabler and consequence rather than a tension, and byte-compatibility with the old
output is explicitly not required (§3.2, T4b). **Q1**: "discard human attestation for UBCa" —
the 178 attestations are given up deliberately; §0.4 remains the honest accounting of what that
costs and now ends in a decision. **Q3**: deleted outright, no archive; git history retains the
crate regardless, so a tag would be convenience, not protection (§2.3).

**Attestation-vs-coverage corrected throughout.** §4.1 now carries the measured comparison: **0**
UBCa inputs are absent from the surviving suite (178 + 3 native = 181), so **no test coverage is
lost** — what is discarded is 357 signed *attestations*, and for the 178 the second opinion of an
independent implementation. **Q5 reframed** accordingly from "should cases be ported" (nothing
needs porting) to "do the two evaluators' attested answers agree on the shared 178."

§0.3 records the human's own rationale verbatim — ubca2 and einmo_suite2 are kept for
**cleanliness, not capability**, and the deprecations exist to make the subsequent refactoring
easier. §0.5 establishes from the tree that Euler-1 and fibonacci are **not einmo cases on
either evaluator**, so there is no capability regression. §4.3 determines the einmo rename
mechanics from source: a `git mv` is signature-safe because `verify_bytes` checks stored bytes
and correspondence excludes metadata, with the surviving suite's `[signing.checked]` passphrase
named as a stop condition.
