---
foop: 47
title: FIRID — atomic per-Fir identity for constanic-clone cycle detection
author: Atlas <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-07-11
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-74: FIRID — atomic per-Fir identity for constanic-clone cycle detection

> Lean draft. Numbered 74 deliberately (sort key 47) at Atlas's request — this
> leaves a gap at sort key 46; `foop_check.py check` will report non-consecutive
> numbering until that slot is filled by a future FOOP. This is accepted, not
> an error to fix.

## Abstract

Triage of a FOOP-13 snapshot bug (`concat_sf_f_more.foo`, `f1` never settles)
found a genuine **constanic-clone cycle**: `a`'s search for `"b"` finds and
clones `f1.b`; the clone's SFF-revived inner search for `"b"` finds the
**same original `f1.b`** again (proved by `FoolRefFir` pointer identity, not
inference) and clones it again, nested one level deeper — without bound. This
is invisible except as "the program never settles," discovered only by manual
`debug_front_task` tracing that measured the same statement being referenced
0 → 1 → 2 → … → 7 times over a 200-step window with no ceiling.

This FOOP gives every `Fir` an **atomic per-instance identity (`FIRID`)** and
makes `constanic_clone_at` track which FIRIDs are **currently being cloned**
on the active clone-recursion stack. When a clone-of-X is asked to clone X
again while X's own clone is still in progress, that is a genuine cycle —
raise an **alarm to the screen** (matching the project's existing
`eprintln!("ALARM: …")` convention) instead of silently spinning until a step
budget or `ITERATION-EXCEEDED` guard eventually times it out somewhere else,
possibly nowhere near the actual cause.

## Motivation

The bug this FOOP responds to took a full triage session to pin down: NYES
traces alone showed "stuck in BRANING," and even fairly deep front-task
inspection just showed "growing nesting depth" — the *cause* (a specific
statement being re-cloned by its own descendant) only became visible once a
dedicated pointer-identity counter was hand-written for that one debugging
session. That tool should be permanent, cheap, and on by default, not
reconstructed from scratch every time this class of bug recurs. A cycle
alarm turns "the program hangs, go trace it for an hour" into "ALARM: FIRID
1847 is being constanic-cloned by its own clone (cycle depth 3)" at the
moment the second re-entrant clone is attempted.

## Specification

### FIRID assignment

- A single process-wide atomic counter (`std::sync::atomic::AtomicU64` or
  equivalent; UBCa is single-threaded today but the atomic costs nothing and
  avoids relitigating this if that changes).
- Every `Fir` instance gets a `firid: u64` field, assigned once at
  construction from the counter (fetch-and-increment), never reassigned.
  Lives on `ProtoBrane` (`proto_brane.rs`) alongside the existing
  `foolish_children`/`ubc_children`/`nyes`/`tasks`/`parent`/`alarm_reason`
  fields — every FIR kind already composes one `ProtoBrane` as `core`, so
  this gives every kind a FIRID for free, no per-kind plumbing.
- **A constanic clone gets a NEW FIRID.** It is a new instance; identity is
  per-instance, not per-logical-statement. (This is what makes "the same
  original `f1.b`, not a coincidental lookalike" a decidable question in the
  first place — today that's only provable by raw pointer comparison, which
  is what the triage session had to fall back on.)
- Accessor: `ProtoBrane::firid(&self) -> u64`. Read-only after construction —
  no setter.

### The in-flight clone stack

- A **thread-local** (`thread_local!`) `RefCell<Vec<u64>>` — the FIRIDs of
  the *original* (pre-clone) Firs whose `constanic_clone_at` call is
  currently on the Rust call stack, innermost last. `constanic_clone_at`
  recursion (SF/SFF unwrap, children-cloning) is real, on-one-call-stack
  recursion (verified: `fir_kinds.rs` `constanic_clone_at`, `:158-190`), so a
  thread-local stack — not `Scope` plumbing — is the right mechanism: it
  needs no signature change to `constanic_clone_at` or any of its ~15 call
  sites, and it naturally unwinds on every return path (including early
  returns and the `?`-propagating ones) via an RAII guard.
- **On entry** to `constanic_clone_at(fir_ref, …)`: read `fir_ref`'s FIRID.
  If it is **already on the stack**, this is a cycle — raise the alarm (see
  below) and push anyway (the guard's `Drop` must still pop correctly; the
  clone proceeds, since refusing to clone would just turn the hang into a
  different bug). If not on the stack, push it via an RAII guard.
- **On exit** (guard drop, any return path): pop.
- The recursive calls inside `constanic_clone_at` for the SF/SFF-unwrap case
  (`:173`, `:182`) and the children-cloning case
  (`clone_children_for_constanic_clone`, wherever it recurses into
  `constanic_clone_at` for each child) go through the SAME guarded entry
  point, so nested clones are tracked uniformly without special-casing which
  recursive path triggered them.

### The alarm

- On cycle detection: `eprintln!("ALARM: constanic-clone cycle detected — \
  FIRID {n} is being constanic-cloned by its own clone (stack depth {d}, \
  kind {kind:?})")` — matching the existing project convention (see the
  SF/SFF-no-children alarm at `fir_kinds.rs:190` for the established style).
- **Non-fatal.** This FOOP does not change evaluation semantics or add a new
  NK/alarm-reason outcome (that is a separate, larger design question — see
  Open Questions). It makes an existing failure mode *audible* at the moment
  it happens; it does not fix the underlying mutual-reference semantics that
  allow the cycle (that repair, if one is wanted, is out of scope here — see
  the "Relation to the triggering bug" note below).
- **Once per cycle occurrence**, not once per step: the alarm fires exactly
  when a re-entrant clone of an in-flight FIRID is attempted, not on every
  subsequent step that re-enters the same cycle. (A given cycle will
  typically fire more than once as the clone-and-revive loop repeats across
  multiple `fir_op_step` calls — that repetition is itself useful triage
  signal, not noise to suppress.)

### Relation to the triggering bug

This FOOP is diagnostic tooling, not a semantic fix. It would have turned the
`f1`/`concat_sf_f_more` triage from a multi-hour manual trace into an
immediate, actionable alarm — but it does not, by itself, make `f1` settle.
Whether the underlying mutual self-reference (`a`'s search for `b` finding a
statement whose own revived content searches back for `b`) should be
prevented, tolerated as a real language semantic (e.g. settling NK once
detected, per the eventual `EconstanicReason`/miss-family work in FOOP-43),
or left to the Foolisher to avoid, is a separate design question — explicitly
deferred, tracked as an Open Question below, NOT decided by this FOOP.

## FIR Impact

- **`ProtoBrane` gains one field**: `firid: u64`, set in `ProtoBrane::new`
  from the atomic counter. No `FirKind` changes, no new NYES state, no new
  `FirKind` variant.
- **New free items** in `fir_kinds.rs` (or a small dedicated module,
  `firid.rs`, if the encapsulation rule favors it — see Open Questions): the
  atomic counter, the thread-local clone-stack, and the RAII guard type. Per
  the project's encapsulation rule (struct owns its data, no free-floating
  functions/data), these should be encapsulated behind a single struct (e.g.
  `CloneCycleGuard`) exposing `CloneCycleGuard::enter(firid) -> Self`
  (returns the guard, alarms internally if already present) rather than
  free functions manipulating the thread-local directly.

## UBC Step Impact

None. `constanic_clone_at`'s existing behavior (what it returns, when it's
called, the resulting FIR tree) is unchanged. The only new behavior is the
alarm side-effect and each Fir instance's `firid` value. No `.snap` output
changes: FIRID is not part of any FIR's `proto_to_core_fir` rendering and
must NOT be added to it (it is an internal identity, not a Foolisher-visible
value) — see Rejected Alternatives.

## Test Plan

- Unit: `firid_is_unique_per_instance` — two Firs constructed independently
  get different FIRIDs; a constanic clone of a Fir gets a FIRID different
  from the original.
- Unit: `clone_cycle_guard_detects_reentry` — directly drive
  `CloneCycleGuard::enter` with a repeated FIRID (no full FVM step needed)
  and assert the alarm path is taken (capture `eprintln!` output, or expose
  a `#[cfg(test)]` hook that records the alarm instead of printing — see
  Open Questions).
- Unit: `clone_cycle_guard_pops_on_every_return_path` — including the
  SF/SFF-unwrap early-return recursion (`fir_kinds.rs:173`, `:182`) — after
  a normal (non-cyclic) nested clone completes, the thread-local stack is
  empty again.
- Regression: reproduce the actual triggering shape (`f1`'s `a`/`b` mutual
  reference, minus the concatenation — see the sibling FOOP-13 triage note)
  and assert the alarm fires. This test may legitimately still show the
  program NOT settling (this FOOP doesn't fix that) — it asserts the alarm
  fires, not that evaluation completes.

## Rejected Alternatives

### A. FIRID rendered in `.snap` output (Foolisher-visible)

Would make FIRID part of the observable contract, subject to churn on every
internal refactor of clone order, and would leak an implementation detail
(instance identity) into language semantics. **Rejected**: FIRID is
debug/diagnostic-only, never sequenced.

### B. Detect cycles via `Rc::ptr_eq` walks instead of an ID counter

Walking the in-progress clone tree comparing raw pointers was exactly the ad
hoc approach used during the FOOP-13 triage session (see Motivation). It
works but requires holding live `FirRef` handles for the whole in-flight set,
which is awkward to thread through `constanic_clone_at`'s existing recursive
structure without a signature change. **Rejected in favor of a plain `u64`
FIRID** — cheap to carry on the thread-local stack (`Vec<u64>`, no `Rc`
lifetime entanglement), and doubles as a general debugging aid (any Fir can
be identified in `eprintln!` output by a short stable number instead of a
raw pointer address).

### C. Global (non-thread-local) mutable state for the clone stack

UBCa is single-threaded today, so a plain `static` with unsafe or a
`RefCell` behind a `Lazy`/`OnceCell` would work identically. **Rejected**:
`thread_local!` is the standard, safe idiom for exactly this shape of state
and costs nothing extra; no reason to reach for `unsafe` or a
process-global.

## Open Questions

- Should a **detected cycle become a semantic outcome** (e.g. force the
  cloned Fir to `NK` with a new `EconstanicReason`/alarm-reason variant,
  analogous to `ITERATION-EXCEEDED`) rather than only an `eprintln!`?
  Deliberately left open — this FOOP's scope is *detection and visibility*,
  not *evaluation semantics*. A follow-on FOOP may want to consume this
  FOOP's `CloneCycleGuard` to drive a real NK-settlement path.
- Where should `CloneCycleGuard` live — inlined in `fir_kinds.rs` next to
  `constanic_clone_at`, or a small dedicated module? Lean: dedicated module
  if it grows a second consumer; inline for now (single consumer,
  `constanic_clone_at`).
- Test-mode alarm capture: `eprintln!` is fine for a human watching a
  terminal but awkward to assert on in a unit test. Does the project have an
  existing pattern for capturing alarm/`eprintln!` output in tests (grep
  found none) — if not, is a `#[cfg(test)]` capture hook worth adding here,
  or is asserting the underlying `Vec<u64>` stack state (via a `#[cfg(test)]`
  accessor) sufficient without touching the alarm's I/O at all?
- Cycle-stack depth cap: should there be a hard maximum in-flight-clone stack
  depth (independent of FIRID repetition) as a secondary safety net for
  non-cyclic-but-pathologically-deep clone cascades? Deferred — out of this
  FOOP's scope, which is specifically identity-repetition detection.

## References

- Triggering bug: FOOP-13 triage session, `concat_sf_f_more.foo` snapshot
  (`f1` never settles — `a`'s search for `"b"` clones `f1.b`; the clone's
  revived `<<b + c>>` search finds the same original `f1.b` again, nesting
  without bound; measured via manual `FoolRefFir`-pointer-identity counting,
  0 → 7 references over 200 steps with no ceiling).
- Code: `foolish-ubca/src/fir_kinds.rs` (`constanic_clone_at`, `:158-190` for
  the SF/SFF-unwrap recursion), `foolish-ubca/src/proto_brane.rs`
  (`ProtoBrane` — the field-holder every FIR kind composes).
- Prior art (existing alarm style to match): `fir_kinds.rs:190`,
  `eprintln!("ALARM: SF/SFF node has no children — cloning wrapper as-is")`.
- Related: FOOP-43 (`EconstanicReason` — a plausible future home if a
  detected cycle should become a semantic NK outcome, per Open Questions).

## Last Updated

**Date**: 2026-07-11
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 5
**Changes**: Initial draft. FIRID (atomic per-Fir instance counter on
`ProtoBrane`) + thread-local in-flight clone stack + `eprintln!` alarm on
re-entrant clone of an already-in-progress FIRID. Grounded directly in the
FOOP-13 triage session that found the triggering cycle by hand
(`constanic_clone_at`, `f1`'s `a`/`b` mutual reference in
`concat_sf_f_more.foo`). Numbered 74 (sort key 47) at Atlas's explicit
request, deliberately leaving a gap at sort key 46.
