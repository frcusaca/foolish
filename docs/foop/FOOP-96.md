---
foop: D69
title: Split fvm_storage.rs — one file per concern
author: Claude Code / claude-opus-5 (directed by the human)
status: Draft
type: Standards
created: 2026-09-16
phase: phase-4
supersedes: []
begun: [ ]
---

# FOOP-96: Split `fvm_storage.rs` — one file per concern

FOOP numbering is little-endian; the full rules live in `foop.md` at the repository root —
**read it before creating or editing a FOOP.** The `foop:` front-matter field here is the
big-endian sort key preceded by `D` (`foop: D69`, file `FOOP-96.md`, following FOOP-86's 68).

> **This is Track 6 member 4** — the "refactoring FOOP — NOT YET WRITTEN" that
> `INDEX.md` §Track 6 and FOOP-86 §N1 both record as a hole in the sequence. This document
> fills it. It runs **after FOOP-86** and **before FOOP-26 ∥ FOOP-46**.

## Abstract

A **mechanical, behavior-preserving** decomposition of `foolish-ubca2/src/fvm_storage.rs`
(7 737 lines — 70% of the crate) into one file per concern, along the module seams the file
**already has**. Nothing is rewritten: every moved block is moved *as text*, so `git blame`
continues to attribute each line to the commit that wrote it. **No semantics change, no public
API changes, no test outcome changes** — the workspace's 467 passing tests pass identically,
in the same number, before and after every single commit of this FOOP.

**This FOOP does not make Project Euler 1 or fibonacci run, and is not trying to.** That is
the deliverable of FOOP-26 ∥ FOOP-46 (`INDEX.md` §Track 6 member 5), which this FOOP exists to
make cheaper. Judged on its own, this FOOP produces exactly one observable change: the same
code, in files a human can read.

## Motivation

### M1 — It is the file every subsequent FOOP must edit

`fvm_storage.rs` is 7 737 of the crate's 11 049 lines. The remaining Track 6 members all edit
it, and after FOOP-86 they edit *only* it plus `sequencer.rs`:

| FOOP | What it does to `fvm_storage.rs` |
|------|----------------------------------|
| FOOP-26 | marks (SF/SFF as a counter), concatenation-as-operator, the three-beat step — `FirSpec`, `fir_op_step`, the compiler |
| FOOP-46 | `BraneConcatOp` — a rewritten concatenation operator with phased search resolution |
| FOOP-76 | FIR equality — a new relation reading `FirSpec` and `foolish_children` |

**FOOP-26 and FOOP-46 are scheduled to run in parallel worktrees** (`INDEX.md` §Track 6,
revised 2026-09-16, the human's call). Two parallel branches editing one 8 000-line file
merge badly: `git` resolves hunks by line proximity, and in a file this size unrelated work
lands close enough together to conflict for no semantic reason. After the split, 26 works
mostly in the core and the compiler while 46 works in the core and concatenation — different
files, and the conflicts that remain are *real* ones worth a human's attention rather than
noise.

### M2 — Reviewability, which is a reason on its own

The human's framing: *"wouldn't we WANT to reduce the size of each file?"* An 8 283-line file
cannot be reviewed properly — not by a human scrolling it, and not by an agent that must hold
it in a context window alongside the change it is making. `rust_instructions.md` §2e states
the project's own rule and this file violates it plainly: **one responsibility per module**,
and *"a name you can't pick usually means the module does too many things"* (§2c.5). The
current file holds an arena, a step machine, cursors, a search engine, a search dispatcher, a
stepping driver, a compatibility bridge, an AST compiler, and 115 tests. There is no name for
that.

This is not a hypothetical cost. The module currently named `core_fir_conversion` prompted
the human to ask *"what does conversion mean?"* — a filename is read hundreds of times more
often than the doc comment above it, and that module's name is wrong in a way that survived
review precisely because it was buried at line 3374 of a file nobody opens whole.

### M3 — The order is deliberate: each step makes the next cheaper

The human has fixed this execution order:

1. **FOOP-86** — retire UBCa. `foolish-cli` moves to `foolish-ubca2`; `foolish-ubca`, the old
   `foolish_core::FirSequencer` bridge, and the old einmo suite retire; `einmo_suite2` takes
   the canonical name.
2. **FOOP-96 — this FOOP.** The file split.
3. **FOOP-26 ∥ FOOP-46** — executed in parallel, on top of the split.

**Why this FOOP comes after FOOP-86 and not before:** FOOP-86 deletes the `proto_to_core_fir`
family — ~650 lines at `fvm_storage.rs:3494–4151`, which exists only so the OLD sequencer can
read arena FIR. So **this FOOP splits a smaller, cleaner file than exists today**, and does
not have to find a good home for ~650 lines that are about to die. Splitting first would mean
carefully placing code, then deleting it a week later.

**Why it comes before FOOP-26 ∥ FOOP-46:** see M1. Each step makes the next cheaper, which is
the whole shape of Track 6.

### M4 — There is a precedent in this repository, of exactly this shape

**FOOP-05** decomposed `foolish-ubca`'s `fir_kinds.rs` (~6 400 lines) and `fir_trait.rs`
(~1 100 lines) into `fir_base.rs` + `fir_ref.rs` + `fir_search_base.rs` + `firs/<kind>.rs`,
for the same reason stated in the same words: *"Two tracks of the roadmap edit the same two
files. Beyond the merge-conflict cost, the current layout violates the project's own
organization preference (one responsibility per module, `rust_instructions.md` §2e)."* It
described itself as **"mechanical, zero-behavior-change"** and **"snapshot-invisible by
definition: every output byte identical before and after."**

This FOOP is that FOOP applied to the other evaluator, and it inherits FOOP-05's framing
deliberately.

## Specification

### §0 What "mechanical" means here, precisely

Three properties define the change, and each one is a constraint on execution:

1. **Move, never rewrite.** Each block's text is moved verbatim into a new file. Its `use
   super::{…}` line becomes that file's import list with `super::` rewritten to the crate
   path. **No line of logic is retyped.** This is what keeps `git blame` legible — a reviewer
   of FOOP-26 asking "why does this line exist?" must still land on the commit that wrote it,
   not on this FOOP.
2. **No visibility widening.** If a block cannot move without making something `pub(crate)`
   that is currently private, that is a discovered coupling, not a paperwork step — **STOP and
   report** (see §4).
3. **No behavior change bundled in.** Renaming a module is *behavior-adjacent* (it changes
   name resolution, and it changes what a `use` line must say). Renames happen in their own
   commits, **after** the moves land. See §3.

### §1 The verified structure of the file

**Verify, do not re-derive.** **RE-MEASURED 2026-09-24 on `jia` at `0df1865b`, after FOOP-86
merged.** The figures below replace the 2026-09-16 measurements, every one of which is now
stale: the file lost 545 lines and every module boundary moved.

| block | lines | size | what it is |
|---|---|---|---|
| top-level (the core) | 1–2236 | 2 236 | arena (`FVMStorage`, `FirPointer`, `Slot`, `ProtoBrane`), `FirSpec`, `fir_op_step`, `combine`, cursors (`FirCursor`/`FirCursorMut`), `revive_constanic`, `default_equal` |
| `mod search_engine` | 2237–2607 | 371 | `BraneNavigator`, `SearchPredicate`, `contextful_search_scan` — navigate + match |
| *(gap)* | 2608–2611 | 4 | blank + doc comment |
| `mod search_fir_dispatch` | 2612–3471 | 860 | reads a `Search` FIR's fields, builds the predicate/navigator, drives the engine |
| *(gap)* | 3472–3482 | 11 | blank + doc comment |
| `mod core_fir_conversion` | 3483–3576 | **94** | the stepping driver ONLY — see §2 |
| *(gap)* | 3577–3579 | 3 | blank + doc comment |
| `mod arena_compiler` | 3580–4328 | 749 | AST → arena FIR; `compose_program_with_system` lives here |
| *(gap)* | 4329–4336 | 8 | blank + doc comment |
| `mod tests` | 4337–7736 | 3 400 | **115** `#[test]` functions |

Drift against the 2026-09-16 table, for a reader holding the older figures:

| block | then | now | why |
|---|---|---|---|
| whole file | 8 282 | **7 737** | −545: FOOP-86 |
| `search_engine` | 377 | 371 | minor edits |
| `search_fir_dispatch` | 744 | **860** | +116: FOOP-86 §6 unsteppable routes, FOOP-94 |
| `core_fir_conversion` | 805 | **94** | **−711: the `proto_to_core_fir` family is GONE** |
| `arena_compiler` | 801 | 749 | −52 |
| `mod tests` | 3 290 | 3 400 | +110: new FOOP-86/94 tests (110 → 115 test fns) |

Measured with a brace-matching walk (`mod X {` through its closing `}`), which is what actually
moves; the doc comment above each `mod` line moves with it.

### §2 `core_fir_conversion` is misnamed — RESOLVED ITSELF, one concern now, still misnamed

**UPDATED 2026-09-24. This section's central finding has largely dissolved, and saying so is
cheaper than letting an executor rediscover it.**

As written 2026-09-16, this section reported the module bundling **two** unrelated concerns:

- **(a) the stepping driver** (~100 lines) — `step_to_constanic` plus the three `step_until*`
  breakpoints the `foolish-debugging` skill documents;
- **(b) the lossy compatibility bridge** (~650 lines) — `proto_to_core_fir` and family,
  converting arena FIR into `foolish_core::Fir` so the OLD sequencer could read it.

It also predicted **"FOOP-86 deletes it entirely"** of (b). **That prediction held.**
`grep -c proto_to_core_fir foolish-ubca2/src/fvm_storage.rs` returns **0**, and the module is
now 94 lines holding only the four stepping functions.

So the two-responsibility problem is gone; no separation work remains. What survives is
narrower: **the module is still misnamed.** `core_fir_conversion` described (b), which no
longer exists — nothing in it converts to core FIR any more. The module-level doc comment that
§2 cited as evidence of two responsibilities (*"The stepping loop **and** the FIR→core-FIR
output-serialization family"*) is now simply **wrong**, describing deleted code.

**`stepping.rs` is created regardless of its size — decided by the human 2026-09-25**, who was
asked whether 94 lines earns its own file: *"those needs their own file. The ability is very
important for debugging and have been used a lot!!! It must be maintained separately and kept in
working order."* The three `step_until*` functions are the project's Foolish debugger and the
foundation of the `foolish-debugging` skill; their `expect(dead_code)` attributes mean *no
production caller by design*, not unused code. Keeping them in their own file is what keeps them
visible and maintained.

**Consequence for the plan:** what was a split becomes a rename — `core_fir_conversion` →
something naming what it does (`stepping`, say) — plus fixing its doc comment. Per §0.3 a
rename is behavior-adjacent and belongs in its OWN commit, after the moves. At 94 lines it is
also a candidate for **not getting its own file at all**; whether four debugging entry points
justify a module is a judgement call for the executor to raise rather than settle silently.

### §3 The target layout

Path-based modules, no `mod.rs` (`rust_instructions.md` §2e.4 and §5 "Don't add `mod.rs`
files"). `fvm_storage.rs` stays a file and gains a sibling directory `fvm_storage/`:

```
foolish-ubca2/src/
├── fvm_storage.rs              ~2 240  the core: arena, FirSpec, fir_op_step, combine,
│                                       cursors, revive_constanic, default_equal
│                                       + `mod` declarations + the curated re-exports
└── fvm_storage/
    ├── search_engine.rs           371  navigate + match (BraneNavigator, SearchPredicate,
    │                                   contextful_search_scan)
    ├── search_dispatch.rs         860  Search FIR → a query the engine can run
    ├── stepping.rs                 94  step_to_constanic, step_until, step_until_line_number,
    │                                   step_until_statement_name  — see §2: at 94 lines,
    │                                   whether this earns its own file is the executor's
    │                                   call to RAISE, not to settle silently
    ├── compiler.rs                749  AST → arena FIR (compose_program_with_system,
    │                                   program_result, build_fir, …)
    └── tests.rs                 3 400  the 115 tests — see §3.2
```

**`core_fir_bridge.rs` is NOT in this layout.** The 2026-09-16 draft listed it (~660 lines,
"DELETED by FOOP-86; see §3.1 for the conditional"). FOOP-86 has since merged and deleted it,
so the conditional resolved: there is no bridge to house. §3.1's conditional is dead text.

**Naming, justified.** `rust_instructions.md` §2c.5: *modules are named by responsibility.*

- **`search_dispatch.rs`** (from `search_fir_dispatch`) — drops the redundant `fir_` infix;
  it sits beside `search_engine.rs` and the pair reads as dispatch → engine.
- **`stepping.rs`** — the responsibility is stepping. Not `driver.rs` (says nothing), and
  **deliberately not anything containing "sequencer"**: `sequencer.rs` is the real sequencer
  (`Ubca2Sequencer`, FOOP-36) and reusing that word here would be actively misleading.
- ~~**`core_fir_bridge.rs`**~~ — moot; FOOP-86 deleted the bridge (§3.1). Originally: "bridge" names what it *is*: a
  one-way adapter to another crate's FIR, kept only for compatibility. It answers the
  human's *"what does conversion mean?"* directly, and its own module doc already calls it a
  bridge (`foolish-ubca/src/fir_trait.rs:143` — *"Accessors for `proto_to_core_fir` bridge"*).
- **`compiler.rs`** (from `arena_compiler`) — inside `fvm_storage/` the `arena_` prefix is
  redundant; everything here is the arena.

**The renames are behavior-adjacent and get their own commits, after all moves land** (§0.3).
A rename changes every `use` site; bundled with a move it makes the diff unreadable and
removes the ability to say "this commit moved text and changed nothing."

#### §3.1 The bridge file, written to work either way — RESOLVED, NO LONGER APPLICABLE

> **RESOLVED 2026-09-24.** This subsection hedged against FOOP-86 landing before or after this
> FOOP. FOOP-86 merged 2026-09-21 and deleted the bridge, so the "either way" is settled: there
> is no `core_fir_bridge.rs` and no bridge to move. **Skip this subsection.** It is kept as the
> record of a conditional that resolved, not as instruction.


**Decision: give the bridge its own file, `core_fir_bridge.rs`.** Reasoning:

- **If FOOP-86 lands first (the plan):** `mod core_fir_bridge;` and `core_fir_bridge.rs` are
  deleted whole. A one-line + one-file deletion is the *cheapest possible* form for FOOP-86's
  removal to take — strictly cheaper than excising ~650 lines from the middle of a shared
  module and then repairing that module's imports and doc comment.
- **If FOOP-86 slips:** the core file is ~660 lines smaller than it would be with the bridge
  left embedded, which is the entire point of this FOOP.

The cost of moving code that is about to be deleted is **one commit of pure text movement**.
The benefit in either branch exceeds it, and crucially the decision does not have to be
revisited if the schedule moves — which is what "written to work either way" means. This is
the recommendation the brief asked for, and it is what §5 of the plan implements.

**If FOOP-86 has already landed when this FOOP begins**, the bridge does not exist and Phase 5
of the plan is struck as `[-] not needed — FOOP-86 removed the bridge`, with a note. The plan
says so explicitly rather than leaving an executor to guess.

#### §3.2 The tests — one `tests.rs`, and the import hazard is real

**Decision: move `mod tests` to a single `fvm_storage/tests.rs`**, declared in the parent as:

```rust
#[cfg(test)]
mod tests;
```

**Not split by subject.** The 115 tests are already organized by subject *within* the module
(contiguous runs with their own local `use` lines), and splitting them into subject files is a
**judgment** change — deciding which test belongs to which subject — which §0 forbids bundling
into a mechanical move. One file at 3 400 lines is not ideal, but it is a test file, it is no
longer in the way of the production code, and subdividing it is a legitimate follow-up that
should be proposed on its own merits. **This is the single biggest win — 40% of the file —
and the one most likely to have a subtle import problem**, which is precisely why it should
not also carry a reorganization.

**The import hazard, verified concretely.** `mod tests` opens with `use super::*`, and that
glob currently resolves against the parent module's items. Measured usage inside the test
block:

| item pulled in via `use super::*` | occurrences |
|---|---|
| `FirSpec` | 235 |
| `FVMStorage` | 114 |
| `FirCursor` | 76 |
| `revive_constanic` | 21 |
| `FirPointer` | 12 |
| `FirCursorMut` | 10 |
| `ConcatProvenance` | 5 |
| `fir_op_step`, `combine`, `ConcatRenderingAid` | 4 each |
| `default_equal`, `ANON_STMT_NAME` | 2 each |
| `ProtoBrane` | 1 |

**These are all parent-module items, so `use super::*` continues to resolve identically after
the move** — a `#[cfg(test)] mod tests;` in a sibling file still has the same `super`. That
part is safe.

**The genuine hazard is elsewhere**, and it is the reason this phase is called out: the test
block also reaches the *sibling* modules by **bare path**, not through `super::`:

```rust
use search_engine::{BraneNavigator, CandidateNavigator, MatchOutcome, ScanCtx,
                    ScanOutcome, SearchPredicate, contextful_search_scan,
                    contextful_search_scan_no_body_check};   // tests.rs:1250 (abs. 6242)
use core_fir_conversion::{proto_to_core_fir, step_to_constanic};        // abs. 6861
use core_fir_conversion::{step_until, step_until_line_number, step_until_statement_name};
                                                                        // abs. 6981
use arena_compiler::compile;                                            // abs. 7068
```

Those bare paths resolve **only** because `use super::*` imported the sibling module names
into scope. They survive the move for the same reason — but they are exactly the kind of line
that breaks if the glob is ever narrowed, and they must be re-pointed when §3's renames land
(`core_fir_conversion` → `stepping` — the `core_fir_bridge` half is deleted, `arena_compiler` → `compiler`,
`search_fir_dispatch` → `search_dispatch`). **The plan re-points them in the rename commits,
not the move commit.**

### §4 The safety argument — why this is safe to do mechanically

Two properties of the current file, **both independently re-verified while writing this FOOP**,
are what make this a move rather than a refactor:

**(1) Every inner module already declares exactly what it needs.** Each `mod` opens with an
explicit `use super::{…}`. Those lines become the new file's imports essentially verbatim:

| module | its current `use super::{…}` |
|---|---|
| `search_engine` | `Equality, FVMStorage, FirCursor, FirPointer, default_equal` |
| `search_fir_dispatch` | `super::search_engine::{…}` + `FVMStorage, FirCursor, FirPointer, FirSpec` |
| `core_fir_conversion` | `ANON_STMT_NAME, ConcatProvenance, FVMStorage, FirCursor, FirPointer, FirSpec, MAX_DEPTH, NyesExt, search_fir_dispatch` |
| `arena_compiler` | `ANON_STMT_NAME, ConcatProvenance, ConcatRenderingAid, FVMStorage, FirCursor, FirCursorMut, FirPointer, FirSpec, StayMarker` |

The dependency graph is therefore already written down, in the file, by whoever wrote each
module. This FOOP does not have to discover it.

**(2) Zero private-internal reaches.** The usual source of pain in a split like this is an
inner module reaching into the parent's private guts, forcing a field to be widened to
`pub(crate)` or an accessor to be invented. Grepping lines 2236–8282 — **every inner module
AND the whole test module** — for the parent's private internals:

```
grep -c "self\.slots\|\.payload\|validate(" <lines 2236-8282>   →   0
```

**Zero hits.** No field needs widening and no accessor needs inventing. The parent's private
state (`Slot.payload`, `FVMStorage.slots`, `FirPointer::validate`) is already reached
exclusively through `FirCursor`/`FirCursorMut`/`FVMStorage` methods — which is exactly the
encapsulation `rust_instructions.md` §2e.5 asks for, and it is already in place.

**This is the whole safety argument, and it is falsifiable.** If either property fails for
some block during execution, the mechanical assumption has failed for that block: **STOP and
report** rather than widening visibility to force the move through (§0.2). That stop condition
is written into every phase of the plan.

### §5 Is the 2 235-line core itself splittable?

**Assessment: not mechanically. Scope it out.** The core is one interconnected thing, and the
evidence is structural rather than a matter of taste:

- `fir_op_step` (lines 917–1494, **577 lines**) is a single `match` over every `FirSpec`
  variant. It is the step machine; it cannot be divided by kind without becoming a dispatch
  redesign, which is a **behavior-adjacent judgment change**, not a move.
- `fir_op_step`, `combine`, `decide_nyes_due_to_children`, and `nyes_from_found` form one
  mutually-recursive cluster with `step_inner`.
- `FirCursor`/`FirCursorMut` (1660–2000) are the *only* readers of `Slot`'s and `ProtoBrane`'s
  private fields. Separating them from the arena types would immediately violate §4(2) — it
  would force exactly the `pub(crate)` widening this FOOP forbids — and would break
  `rust_instructions.md` §2e.4, *"keep the type and its `impl` blocks together"*.
- The remaining items (`ArenaId`, `FirPointer`, `Slot`, `ProtoBrane`, `FVMStorage`, `FirSpec`,
  `ConcatProvenance`, `ConcatRenderingAid`, `StayMarker`) are the arena's type vocabulary,
  each small, each used by all the rest.

Post-split, `fvm_storage.rs` is ~2 240 lines with **one** nameable responsibility: *the arena
and its step machine.* That is a file a human can review, and it satisfies §2e.1. **A further
split of the core is a design exercise, and this FOOP explicitly refuses to disguise one as a
mechanical move.** If a clean seam is found later, it deserves its own FOOP and its own
argument.

## FIR Impact

**None.** No `FirSpec` variant is added, removed, renamed, or reshaped. No field changes. No
state-machine change. No serialization implication. FIR is untouched by construction: every
line that mentions FIR is moved verbatim.

## UBC Step Impact

**None.** `fir_op_step` moves at most as a whole, and in the target layout (§3) it does not
move at all — it stays in `fvm_storage.rs`. No stepping order changes, no NYES transition
changes, no step *counts* change. Step counts are part of the einmo OUTPUT contract, so this
is checkable and not merely asserted: the einmo gate must stay green byte-for-byte at every
commit, which it cannot do if any step count moved.

## Test Plan

**The existing tests ARE the test plan.** There are no new tests to write, and inventing some
would be theatre: a test written for this FOOP could only assert that moved code still does
what it did, which is precisely what the **791 existing tests already assert**, having been
written against the behavior by the FOOPs that built it. A new test here would add coverage of
nothing and would itself be unreviewed code landing in a change whose entire claim is that no
code changed.

### The invariant

> **Same tests, same count, same results — before and after every commit.**

Measured on `jia` at `8c9043d8` on 2026-09-16 via `cargo test --workspace`:

| metric | baseline |
|---|---|
| **passed** | **791** |
| failed | 0 |
| ignored | 0 (the `foolish-ubca/src/evaluator.rs` doctest went with that crate, FOOP-86) |
| `foolish-ubca2` lib tests | 184 passed, incl. all three einmo gates |
| `#[test]` fns in `fvm_storage.rs` | 115 |

**791 must not move in either direction.** A test that *vanishes* is as bad as one that fails:
a `#[cfg(test)]` block that stops being compiled in reports no error, it simply stops running,
and the suite goes green while covering less. Both the count and the result are checked.

### The discipline — this is how the tests are used

1. **One block per commit**, never bundled. Each commit moves exactly one block and does
   nothing else.
2. **`cargo test --workspace` after every single move**, checking **791 passed / 0 failed**.
   The count is read, not glanced at.
3. **`cargo fmt --all --check`** and **`cargo clippy -p foolish-ubca2 --all-targets`** clean at
   every step.
   - *Known and NOT this FOOP's to fix:* 4 pre-existing `foolish-core` clippy errors in
     `sequencer.rs` break a workspace-wide `-D warnings` gate. Scope clippy to
     `-p foolish-ubca2`. If a *new* warning appears in `foolish-ubca2`, that is this FOOP's
     and must be fixed or reported.
4. **Never bundle a behavior change with a move.** Renames are behavior-adjacent: their own
   commits, after the moves (§0.3, §3).
5. **If a block cannot move without widening visibility: STOP and report.** Do not widen a
   field to `pub(crate)` on the agent's own judgment (§0.2, §4).

### Why the einmo gate is the strongest oracle available

`foolish-ubca2`'s einmo gates compare **signed snapshot bytes**, including rendered output,
alarms, **and step counts**. For a change claiming zero behavior difference, that is a
byte-identity oracle over the whole corpus — a far stronger check than any assertion this FOOP
could write. FOOP-05 made the same observation about the same class of change
(*"snapshot-invisible by definition: every output byte identical before and after"*), and
`INDEX.md` §Track 6 member 4 records it as the intended verification method for this FOOP
specifically.

### No Promotion Review Gate

**This FOOP has no `output` → `checked` promotion, and its plan deliberately contains no
Promotion Review Gate.** Stating that explicitly rather than silently omitting it: the gate
exists for FOOPs that *generate* new expected output. This FOOP generates none. Every einmo
baseline must remain **byte-identical**, so a promotion would be the precise signal that
something went wrong.

> **If any `checked/` baseline diverges during this FOOP, that is a regression this FOOP
> introduced. Fix the move. NEVER promote.** (AGENTS.md §"Non-regression invariant";
> `rust_instructions.md` §"Phase-by-phase testing discipline".)

There is likewise **no `comprehensive.foo`**: that test demonstrates a *new feature*
interacting with old ones, and this FOOP has no feature. Writing one would add a new baseline
to a change whose contract is that no baseline changes.

## Plan of Execution for Plan

Per AGENTS.md §FOOP, phases are assigned **by complexity**, not sized as a block.

| phase | kind | agent | why |
|---|---|---|---|
| 0 — read spec, confirm FOOP-86 landed, re-measure, record baseline | judgment | **larger** (Opus/Sonnet) | decides whether the premise still holds; the bridge may be gone |
| 1–6 — the moves (one block per phase) | execution | **smaller** (Sonnata / qwen3.8-27B) | fixed target: text moves, 791 passes, zero diff in behavior |
| 7 — the renames + re-pointing `use` sites | execution | **smaller** | mechanical, compiler-verified: a missed site fails to build |
| 8 — final review, merge prep | judgment | **larger** | reads the whole diff and asserts it is a move |
| **any STOP** — a block needs visibility widening, or 791 moves | judgment | **larger + human** | a failed move indicates a real coupling; diagnosing it is design work |

**The three properties that make the small-model phases safe** (built in deliberately):

1. **Facts inline, not referenced.** The plan carries the exact line ranges, the exact
   `use super::{…}` import list for every block, the exact test baseline (791/0/1), and the
   exact command forms. An executing agent verifies these rather than rediscovering them —
   marked *verify, don't re-derive* in the plan.
2. **A fixed target per phase.** "Done" is `cargo test --workspace` reporting **791 passed, 0
   failed** plus clean `fmt`/`clippy`. No judgment is required to check it.
3. **Named stop conditions.** Each phase states what wrong looks like — a compile error naming
   a private item, a test count that is not 791, a clippy warning inside `foolish-ubca2`, a
   divergent einmo baseline — and that the answer is **STOP and report**, never improvise.

**Never delegated at any size** (AGENTS.md §"The agent is responsible for correctness"): any
decision to widen visibility; any decision to change a crate this FOOP promised not to touch;
any `einmo promote` (there are none, and an executing agent proposing one has found a bug);
marking any Verified-tier test `#[ignore]`.

## Rejected Alternatives

### A. Do nothing

The file stays 7 737 lines. FOOP-26 and FOOP-46 then run in parallel
worktrees against one enormous file, and every merge conflict between them is a line-proximity
accident rather than a real disagreement — the expensive kind, because a human must read both
sides to discover there was no conflict. It also leaves the project in standing violation of
its own `rust_instructions.md` §2e.1, in the file it edits most. **Rejected**: the cost is
paid by every later FOOP, repeatedly, and it compounds.

### B. Split into more files — decompose the 2 235-line core too

Tempting, and rejected on the evidence in §5: `fir_op_step` is one 577-line `match` over every
`FirSpec` variant, and the cursors are the only readers of the arena's private fields.
Dividing either requires a **dispatch redesign** or a **visibility widening** — both
behavior-adjacent judgment work, and both forbidden by §0. **Rejected**: it would make this a
design FOOP wearing a mechanical FOOP's clothes, and would forfeit the one property that makes
the change reviewable (that every commit provably changed nothing).

### C. Split into fewer files — e.g. one `search.rs` merging engine and dispatch

Merging `search_engine` (377) and `search_fir_dispatch` (744) into one ~1 130-line `search.rs`
is defensible in isolation. **Rejected** because the two have genuinely different
responsibilities — the engine navigates and matches while knowing nothing about `SearchFir`;
the dispatcher reads a `Search` FIR's fields and builds a query — and the existing seam already
encodes that (FOOP-23's one-engine model, AGENTS.md §"The one-engine model"). Merging them
would *destroy information* the current file already carries, which is the opposite of this
FOOP's purpose.

### D. Do the split AFTER FOOP-26 / FOOP-46 rather than before

**Rejected**, and it inverts the whole argument for doing this. The split's primary value (M1)
is enabling 26 ∥ 46 to run in parallel; performed afterwards it delivers none of it, and it
would have to be performed against a file two FOOPs have just grown, with two sets of fresh
semantics to avoid disturbing. Doing it first also means 26 and 46 are *reviewed* in small
files.

### E. Rewrite rather than move — "since we're in here anyway"

**Explicitly and firmly rejected.** The moment a block is retyped instead of moved, three
things are lost at once: `git blame` stops attributing lines to the commits that reasoned about
them; the reviewer can no longer confirm "this commit changed nothing" by inspection; and the
791-test invariant stops being a *proof* of no-change and becomes merely evidence. Any
improvement noticed while moving is written down and proposed separately — this is a move.

### F. Leave the `proto_to_core_fir` bridge embedded because FOOP-86 deletes it anyway

Considered seriously (see §3.1) — moving code that is about to be deleted is *prima facie*
wasted work. **Rejected** because the waste is one text-move commit, while the benefit lands in
both branches of the schedule: if FOOP-86 lands first the bridge file is deleted whole (a
strictly cheaper deletion than excising 650 lines from a shared module), and if FOOP-86 slips
the core is ~660 lines smaller. It is the option that does not have to be revisited when the
schedule moves.

### G. Split `mod tests` by subject into several files

**Rejected for this FOOP, and worth proposing separately.** Deciding which test belongs to
which subject is judgment work, and §0.3 forbids bundling judgment into a move — especially in
the single largest and import-riskiest block (§3.2). One 3 400-line test file that is out of
the production code's way is a real improvement; subdividing it is a further one that should
stand on its own argument.

### H. Use `mod.rs` (`fvm_storage/mod.rs`) instead of `fvm_storage.rs` + `fvm_storage/`

**Rejected by project rule**, not preference: `rust_instructions.md` §5 — *"Don't add `mod.rs`
files. → path-based modules (`foo.rs` + `foo/`)"* — and §2e.4.

## Open Questions

- **Does FOOP-86 land before this FOOP begins?** At the time of writing (2026-09-16), FOOP-86
  is `status: Draft`, authored but unexecuted. §3.1 is written so that **either answer works**,
  and the plan's Phase 0 checks which world it is in and strikes Phase 5 if the bridge is
  already gone. *This question does not block authoring; it is resolved by looking, at
  execution time.*
- **Should `mod tests` later be split by subject?** §3.2 and Rejected Alternative G defer this
  deliberately. Worth revisiting once the production files have settled after FOOP-26 ∥ 46 —
  at which point the subject boundaries will be clearer than they are now.
- **Is `core_fir_bridge.rs` worth its own module doc rewrite?** The existing doc comment opens
  by describing *two* responsibilities (§2). Splitting the module makes half of it wrong. The
  plan's rename phase corrects it, but only the sentence that is falsified by the split —
  anything more is a documentation change riding on a mechanical FOOP.

## References

- **Prior FOOPs**
  - [FOOP-05](FOOP-05.md) — `fir_kinds.rs` decomposition. The direct precedent: same change,
    same "mechanical, zero-behavior-change" framing, other evaluator.
  - [FOOP-86](FOOP-86.md) — Retire UBCa. Removes the bridge this FOOP files separately;
    **must land first** (§M3). Its §N1 records this FOOP as the gap it does not fill.
  - [FOOP-16](FOOP-16.md) — built `foolish-ubca2` and `fvm_storage.rs`; §Specification is the
    design rationale for the arena, `FirPointer`, and the encapsulation §4(2) relies on.
  - [FOOP-36](FOOP-36.md) — `Ubca2Sequencer`; §1 documents `proto_to_core_fir` as lossy.
  - [FOOP-26](FOOP-26.md), [FOOP-46](FOOP-46.md) — the parallel pair this FOOP unblocks.
  - [FOOP-23](FOOP-23.md) — the one-engine search model, the seam §C preserves.
- **Project documents**
  - `rust_instructions.md` §2e (structuring a module), §2c.5 (naming by responsibility),
    §5 (no `mod.rs`; organize by responsibility), §"Phase-by-phase testing discipline".
  - `AGENTS.md` §"Development Rules", §"Non-regression invariant (hard rule)".
  - `foop.md` — the authoritative FOOP process.
  - `docs/foop/INDEX.md` §Track 6 — the execution order and this FOOP's place in it.
- **Code locations** (all `foolish-ubca2/src/`, measured on `jia` at `8c9043d8`)
  - `fvm_storage.rs:1–2235` — the core
  - `fvm_storage.rs:2236–2612` — `mod search_engine`
  - `fvm_storage.rs:2619–3362` — `mod search_fir_dispatch`
  - `fvm_storage.rs:3374–4178` — `mod core_fir_conversion` (3395–3492 stepping; 3494–4151 bridge)
  - `fvm_storage.rs:4183–4983` — `mod arena_compiler`
  - `fvm_storage.rs:4989–4990` — the `pub(crate) use` re-exports
  - `fvm_storage.rs:4993–8282` — `mod tests`
  - `lib.rs` — `pub mod fvm_storage;` and the crate's curated re-exports

## Last Updated

**Date**: 2026-09-25
**Updated By**: Claude Code / claude-opus-5
**Changes**: §2 now records the human's ruling (2026-09-25) that **`stepping.rs` is created
regardless of its 94-line size**: the three `step_until*` functions are the project's Foolish
debugger and the foundation of the `foolish-debugging` skill, their `expect(dead_code)` attributes
mean *no production caller by design* rather than unused code, and a separate file is what keeps
them visible and maintained. Prior entry: re-measured the whole FOOP against `jia` post-FOOP-86 —
the file is **7 737** lines not 8 282, the baseline **467 / 0 / 0** not 791 / 0 / 1, `mod tests`
**115** functions in **3 400** lines not 110 in 3 290; §1's table replaced with brace-matched
measurements plus a drift table; §2's two-concerns finding resolved itself when FOOP-86 deleted
the `proto_to_core_fir` bridge, turning a split into a rename; §3 drops `core_fir_bridge.rs` and
§3.1's conditional is marked resolved.
