---
foop: 44
title: Macros — research and design
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-09
phase: phase-6
supersedes: []
begun: [ ]
---

# FOOP-44: Macros — research and design

> **Standalone research FOOP — dispatchable to a dedicated agent working in parallel.**
> The deliverable is a design, not code. Fuller notes in
> `NOTES-creation-lineage-and-search-family.md` §3. (Implementation order: #11. Renumbered
> 2026-07-09.)

## For the agent (parallel-work handoff)

- **Read first:** this file; `NOTES-creation-lineage-and-search-family.md` §3 + Engineering
  guidance; how Foolish matching works (searches, value search, characterizations — FOOP-23,
  FOOP-63); the all-results search (FOOP-14); Rust `macro_rules!` vs proc-macros as prior art.
- **First concrete experiment:** hand-write, in prospective Foolish syntax, two illustrative
  macros — (a) a **repetition macro** (repeat a statement N times), (b) a **boilerplate
  generator** (e.g. generate getter statements for a brane's fields). Try to express them as
  *branes that transform branes* using the existing matching substrate. Record what's missing.
- **Do NOT touch:** engine internals yet — this is research/design first. No approved snapshots.
- **Deliverables:** (1) a **research memo** comparing declarative pattern-rewrite (brane→brane)
  vs a distinct expansion phase (AST / `Astn`→FIR / compute-time rewriting FIR); (2) a decision on
  **which layer expansion happens at**; (3) a minimal macro spec + prototype plan.
- **Report format:** update this FOOP's Specification/Open Questions with findings.
- **Worktree:** yes once prototyping; the research memo can precede it.

## Abstract

Research and specify **how to write macros in Foolish**, leveraging the language's name/value/
characterization matching and — once it lands — all-results search (FOOP-14). The central design
decision: **at which layer expansion happens** (parse-time AST, compile-time `Astn`→FIR, or a FIR
that rewrites at compute-time), and whether a macro is a **first-class brane that transforms
branes** (very Foolish) or a separate expansion mechanism.

## Motivation

Macros are the natural next abstraction once Foolish has rich matching: "match a pattern over a
brane's statements, rewrite/expand it." Because Foolish already has a matching substrate, macros
may be expressible *in the language* rather than as a bolted-on preprocessor — worth researching
before committing to a design. (Note: FOOP-73 already showed boolean operators can be *table
searches* — macros may similarly be expressible as brane transforms, in the same Foolish-native
spirit.)

## Specification

Deferred — first deliverable is a **research memo**, then a design. Directions:

- **Prior art:** Rust declarative (`macro_rules!`) vs procedural (syntax-tree) macros; Lisp
  macros; template/AST-rewrite systems.
- **Foolish-native option:** a macro as a **brane that transforms branes** — match statements via
  the existing predicates (name/value/characterization + FOOP-14 all-results), produce rewritten
  statements. Explore sufficiency and hygiene.
- **Expansion layer:** parse-time (AST), compile-time (`Astn`→FIR), or a compute-time rewriting
  FIR. This is the decision the FOOP must make.

## FIR Impact

TBD by the research. Possibly none (if expansion is pre-FIR) or a rewriting FIR (if compute-time).

## UBC Step Impact

TBD.

## Test Plan

TBD by the design. Likely: a small set of illustrative macros (repetition, boilerplate-generator)
expanded and evaluated, with approval snapshots.

## Rejected Alternatives

### A. No macros

**Rejected** (eventually) — a high-value abstraction the matching substrate naturally enables. But
this FOOP is last; deferral until the substrate (FOOP-14 all-results, characterizations/FOOP-63)
exists is intentional.

### B. A C-style textual preprocessor

Pure text substitution before parsing. **Rejected** (tentatively): ignores Foolish's structural
matching; a brane-transforms-brane model is more in keeping. To be argued in the research memo.

## Open Questions

Essentially everything — hygiene, expansion phase, recursion within macros, whether a macro is a
first-class brane, surface syntax. The least-defined FOOP; kept last on purpose.

## Plan (lean)

- [ ] Research memo: Rust declarative vs procedural macros, Lisp, AST-rewrite; syntax-tree search.
- [ ] Evaluate the "brane transforms brane" Foolish-native model against the matching substrate
      (searches + FOOP-14 all-results + characterizations/FOOP-63).
- [ ] Decide the expansion layer (parse / compile / compute-time FIR).
- [ ] Spec + prototype a minimal macro; approval examples.
- [ ] Worktree lifecycle per `foop.md`.

## Appendix — notes toward the full spec

- **Benefits from FOOP-14** (all-results — "find every statement matching a pattern") and
  **FOOP-63** (characterizations as type/tag inputs). Not a hard dependency for the research, but
  the design will reference them.
- Because Foolish evaluation is itself a rewrite-to-constanic process, a compute-time rewriting
  FIR might align naturally with the model — worth serious exploration. Cf. FOOP-73 booleans as
  table searches: the "define it in Foolish, not the FVM" instinct may extend to macros.

## References

- Prior: FOOP-14 (all-results), FOOP-63 (characterizations), FOOP-33 (characterizations origin),
  FOOP-73 (Foolish-native definition instinct).
- External: Rust `macro_rules!` / proc-macros; Lisp macros.
- Notes: `NOTES-creation-lineage-and-search-family.md` §3 + Engineering guidance.

## Last Updated

**Date**: 2026-07-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Renumbered from FOOP-04 to FOOP-44 (impl-order reorg — last). Added a **"For the
agent" parallel-work handoff** header. Macros for Foolish; central question is the expansion layer
and whether a macro is a brane-that-transforms-branes; leans on FOOP-14 + characterizations.
