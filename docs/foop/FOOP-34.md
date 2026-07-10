---
foop: 43
title: Recursion Upgrades
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-09
phase: phase-5
supersedes: []
begun: [ ]
---

# FOOP-34: Recursion Upgrades

> **Standalone research FOOP — dispatchable to a dedicated agent working in parallel.**
> Deliberately under-specified: the concrete content is discovered *during* the FOOP by writing
> real recursive programs. Fuller notes in `NOTES-creation-lineage-and-search-family.md` §6.
> (Implementation order: #10. Renumbered 2026-07-09.)

## For the agent (parallel-work handoff)

- **Read first:** this file; `NOTES-creation-lineage-and-search-family.md` §6 + Engineering
  guidance; `docs/vintage_legacy/ADVANCED_FEATURES.md` §Recursion / §Corecursion / §Mutual
  Recursion; FOOP-2 (why if/then/else is gone); FOOP-24 (detachment — may help recursion).
- **First concrete experiment:** write a **Fibonacci computer** — first as a Python or Rust
  reference, then attempt it in Foolish-as-it-exists. Record exactly what is impossible or awkward.
  Then repeat for ~1–2 dozen distinct recursive algorithms (list below).
- **Do NOT touch:** approved `.snap` files; `infinite_loop.foo`'s behavior (its non-termination is
  ACCEPTED — see below). Do not implement cycle detection.
- **Deliverables:** (1) the reference algorithms; (2) a friction report — what sugar/upgrades
  Foolish needs for terminating recursion; (3) a spec for those upgrades; (4) `↑` enabled if the
  experiments confirm it's the primitive. (2)+(3) are the real FOOP body, written *after* the
  experiment.
- **Report format:** update this FOOP's Specification/Open Questions with findings as you go.
- **Worktree:** yes — open a worktree per `foop.md` once you start implementing (the reference
  algorithms + friction report can be drafted before).

## Abstract

Make Foolish support **intentional, terminating recursion**. Discovery-driven: the first task is
to write ~1–2 dozen distinct recursive algorithms (Fibonacci first) as references and then in
Foolish; the friction reveals the syntactic sugars and recursion upgrades Foolish needs. Comes
**after the full search suite** (FOOP-43/53/63/73/83/93/04/14) and detachment (FOOP-24).
**No cycle detection** — accidental non-termination stays as-is.

## Motivation

Recursion is table-stakes, but the *right* Foolish spelling is not yet known: `if/then/else` was
removed (FOOP-2, branch via search), and detachment is a search filter (FOOP-24), not the
parameter-binder the vintage `[n=n-1]` examples assumed. Rather than speculate, we write the
canonical algorithms and let the pain points define the upgrades — matching the project's "tests
first" ethos (the programs are both the requirements instrument and the approval suite).

## Specification

Deferred to the discovery phase. Known anchors:

- **`↑` (upward search)** is the likely enabling primitive — it finds the current/enclosing brane
  (self-reference). Parsed today (`Astn::UpwardSearch`), compiler-rejected (`compiler.rs:29`).
  Build a FIR (or resolve `↑` to the home brane via the parent chain / `get_my_brane`).
- How a recursion variable is rebound, and how the base case is chosen without `if/then`, are
  **open — the algorithm exercise answers them empirically.** Detachment (FOOP-24) may help.
- **No cycle detection.** `infinite_loop.foo` (`{f1={f1};stuck=f1}` → `NK(ITERATION-EXCEEDED)`) is
  **accepted** behavior; a self-referential waiting-cycle legitimately runs to the step budget.

## FIR Impact

TBD by the discovery phase. At minimum, `↑` needs compiler support (and possibly a FIR). Any new
recursion FIR owes a `*_nyes_transitions` test (AGENTS.md).

## UBC Step Impact

TBD. `↑` resolution is the known piece.

## Test Plan

**The recursive-algorithm suite IS the test plan.** Reference (Python/Rust) → confirm → Foolish
`.foo` approval program that must compute correctly and settle. Candidate set (distinct *shapes*,
~1–2 dozen):

- *Numeric:* Fibonacci (naive + accumulator), factorial, GCD (Euclid), fast integer power,
  Ackermann, sum-to-n, digit-sum, Collatz-length, is-even/is-odd (mutual recursion).
- *Combinatorial:* binomial coefficient, Towers of Hanoi, permutations/subsets, Catalan numbers.
- *Structural (branes/lists/trees):* length, reverse, map, fold/reduce, member, flatten a nested
  brane, tree depth, tree traversal (pre/in/post), binary search over a sorted brane,
  quicksort/merge-sort.
- *Classic:* palindrome check.

Do **not** add a non-termination/cycle test as a bug (`infinite_loop.foo` stays ITERATION-EXCEEDED).

## Rejected Alternatives

### A. Design recursion up-front

**Rejected**: the right spelling depends on the mature search substrate and real friction;
up-front design risks the wrong abstraction. Discovery-driven is deliberate.

### B. Cycle detection for `infinite_loop`

**Rejected**: that non-termination is accepted behavior; this FOOP is about intentional
terminating recursion.

## Open Questions

Deferred to the write-the-algorithms first task: recursion-variable rebinding (does detachment
help?); base case without `if/then`; does `↑` return a brane or a search cursor; what sugar
Fibonacci reveals as missing; corecursion / mutual recursion scope (likely a later sub-phase).

## Plan (lean)

- [ ] **Write ~1–2 dozen recursive algorithms as Python/Rust references** (Fibonacci first) —
      distinct shapes (linear/binary/mutual/accumulator/generative).
- [ ] Attempt each in Foolish-as-it-exists; record what is impossible/awkward (friction report).
- [ ] Enable `↑` (compiler + FIR); `↑` `*_nyes_transitions` test if a new FIR.
- [ ] From the friction, spec + implement the needed sugar/upgrades (the real FOOP body).
- [ ] Approval programs for the algorithm suite; comprehensive `foop_34_comprehensive.foo`.
- [ ] Worktree lifecycle per `foop.md`.

## Appendix — notes toward the full spec

- **Sequenced late** — depends on the full search suite and likely the numeric/type work the
  algorithms need. **Depends on FOOP-43** (miss semantics) at minimum; coordinate with FOOP-24
  (detachment).
- Mutual recursion (even/odd) needs passing an unbound name into context, since Foolish can't
  search downward past the current line (ADVANCED_FEATURES §Mutual Recursion).

## References

- Prior: FOOP-2 (removed if/then/else), FOOP-24 (detachment ≠ binder, may help), FOOP-43 (miss).
- Docs: ADVANCED_FEATURES §Recursion/§Corecursion/§Mutual Recursion.
- Code: `compiler.rs:29` (`↑` rejected), `Astn::UpwardSearch`; `evaluator.rs::step_to_settled`
  (MAX_STEPS); `infinite_loop.foo` snapshot (accepted ITERATION-EXCEEDED).
- Notes: `NOTES-creation-lineage-and-search-family.md` §6 + Engineering guidance.

## Last Updated

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Renumbered from FOOP-93 to FOOP-34 (impl-order reorg). Added a **"For the agent"
parallel-work handoff** header (read-first list, first experiment, do-not-touch, deliverables,
report format, worktree). Discovery-driven; write ~1–2 dozen algorithms first; `↑`; no cycle
detection.
