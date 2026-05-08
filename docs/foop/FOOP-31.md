---
foop: 31
title: SPA1 — Semi-Privately Available milestone: UBC reference implementation (depth-first)
author: hc <hc.busy@gmail.com>
status: Draft
type: Milestone
created: 2026-05-07
phase: meta
supersedes: []
---

# FOOP-31: SPA1 — Semi-Privately Available milestone (UBC reference)

## Abstract

Defines **SPA1** (Semi-Privately Available), the first alpha-equivalent milestone of the
Foolish project. SPA1 comprises the complete Rust reference implementation — parser,
compiler, depth-first VM, and CLI/REPL — covering all basic language features through
Phase 4 of the Rust MVP plan.

SPA1 is the **UBC** (Unicellular Brane Computer) reference implementation. It establishes
the canonical behavior of the Foolish language. A separate effort, **UBCb**, will be
developed in parallel and incrementally catch up to UBC's capabilities.

## Motivation

Currently, development work is scattered across FOOPs (language design decisions) and
phase documents (implementation plans). There is no single milestone that captures "what
does a working, releasable Foolish system look like?" SPA1 provides that anchor:

- **For the Rust implementation**: a clear target — finish all governing FOOPs through
  Phase 4 and ship a working CLI/REPL.
- **For UBCb**: a clear starting point — UBCb begins as a subset of SPA1 and grows
  toward full UBC parity over time.
- **For language design**: any new FOOPs that modify core semantics can be evaluated
  against "does this affect SPA1?" with a clear yes/no answer.

## SPA1 Scope

SPA1 includes all of Phases 1 through 4 of the Rust MVP plan:

| Phase | Document | Description |
|-------|----------|-------------|
| Phase 1 | [phase1_compiler.md](todo/rust-mvp/phase1_compiler.md) | Parser (antlr4rust), AST, AST→FIR compiler, FIR JSON serialization |
| Phase 2 | [phase2_ubc.md](todo/rust-mvp/phase2_ubc.md) | Depth-first UBC evaluator: search, short-circuiting, constanic cloning, SF/SFF, seeks |
| Phase 3 | [phase3_concatenation.md](todo/rust-mvp/phase3_concatenation.md) | Concatenation operator `A B C ...` with merged brane and recoordination |
| Phase 4 | [phase4_cli.md](todo/rust-mvp/phase4_cli.md) | CLI binary: `compile`, `run`, `step`, `repl` |

### Language Features in SPA1

| Feature | Phase | Governing FOOPs |
|---------|-------|-----------------|
| Branes `{...}` | 1 (compile), 2 (eval) | — |
| Integer literals | 1 | — |
| Identification `name = expr` | 1, 2 | — |
| `???` (NK literal) | 1, 2 | — |
| Arithmetic `+ - * / %` | 1, 2 | FOOP-9 |
| Unary `-` | 1, 2 | FOOP-9 |
| Bare identifiers (unanchored search) | 1, 2 | FOOP-4 |
| Anchored search `.`, `?`, `^`, `$`, `#N` | 1, 2 | FOOP-10 |
| `#-N` unanchored seek | 1, 2 | FOOP-4 |
| SF `<expr>` and SFF `<<expr>>` | 2 | — |
| Concatenation `A B C ...` | 3 | FOOP-3 |
| Comments, shebang | 1 | — |
| if-then-else | **rejected** | FOOP-2 |
| Detachment | **deferred** (Phase 7) | — |
| Forward search `~` | **deferred** (Phase 7) | — |

### Governing FOOPs for SPA1

| FOOP | Phase(s) | Status | Title |
|------|----------|--------|-------|
| FOOP-2 | Phase 1 | Final | Remove if-then-else from the language |
| FOOP-4 | Phase 1 | Final | Bare identifiers compile to anchored regex SearchFirs |
| FOOP-5 | Phase 1 | Final | Compile-time vs evaluation-time work — the FIR contract |
| FOOP-9 | Phase 1 | Brewing | Operators are brane-like FIRs with positional unnamed operands |
| FOOP-12 | Phase 1 | Brewing | Alarms — diagnostic levels emitted by compiler and evaluator |
| FOOP-6 | Phase 2 | Brewing | Phase 2 evaluator is depth-first sequential |
| FOOP-7 | Phase 2 | Brewing | Constanic Clone — recoordination contract |
| FOOP-8 | Phase 2 | Brewing | FIRs are mutable; parent pointers are post-clone |
| FOOP-10 | Phase 2 | Brewing | Anchored search through constanic anchors |
| FOOP-11 | Phase 2 | Brewing | Search stops at NK; search result becomes NK |
| FOOP-3 | Phase 3 | Brewing | Concatenation algorithm |

### What SPA1 Excludes

- **Phase 5** — Breadth-first evaluation (deferred, UBC-specific)
- **Phase 6** — Web brane browser (application layer, post-SPA1)
- **Phase 7** — Detachment branes, partial application, forward search liberation
- **UBCb** — The message-passing variant (governed by FOOP-14)
- **Scala/Java** — Parallel language implementations (post-SPA1 cross-validation)

## UBC as Reference Implementation

The Rust implementation (UBC) is the **reference implementation** of Foolish. Its semantics
are the canonical behavior. UBCb — and any future implementations in other languages —
must match UBC's behavior on all approval tests.

The approval test suite (`test-resources/`) is the behavioral contract. UBC's approval
tests are the ground truth.

## SPA1 Exit Criteria

- All governing FOOPs (listed above) are in status `Final` or `Implementing`.
- All Phase 1 unit tests pass (AST, AST→FIR, roundtrip).
- All Phase 2 approval tests pass (60+ `.foo` files).
- All Phase 3 concatenation tests pass.
- `foolish run <file.foo>` produces correct output for the full test suite.
- `foolish repl` handles multiline input, parse errors, and ECONSTANIC display.
- `foolish step <file.foo>` emits useful debugging output.

## UBCb Relationship

UBCb (governed by FOOP-14) is a **separate development track** that:

1. Starts with a subset of SPA1 capabilities.
2. Uses a message-passing architecture (per the UBC2 design docs in `docs/ubc1/how/`).
3. Grows its capability incrementally to catch up with UBC.
4. Must match UBC's approval test results at each parity checkpoint.

UBCb is NOT part of SPA1. SPA1 is UBC only.

## Test Plan

SPA1 is validated by the existing test infrastructure:

- Phase 1: three test layers (AST, AST→FIR, roundtrip) per [phase1_compiler.md]
- Phase 2: 60+ `.foo` approval tests per [phase2_ubc.md]
- Phase 3: concatenation-specific approval tests per [phase3_concatenation.md]
- Phase 4: CLI functional tests and REPL session tests per [phase4_cli.md]

## Rejected Alternatives

### A. Include UBCb in SPA1

Ship both UBC and UBCb at the same time. **Rejected**: UBCb has fundamentally different
architecture (message-passing vs depth-first function-call stepping). Shipping both
simultaneously doubles the implementation burden and muddies the reference semantics.
UBC establishes the ground truth first.

### B. Make SPA1 only Phase 1+2 (no concatenation or CLI)

**Rejected**: concatenation is a core language feature, not optional. A CLI without
concatenation is significantly less useful than one with it. Phase 3 is a natural
part of the SPA1 milestone.

### C. Include Phase 5 (breadth-first) in SPA1

**Rejected**: Phase 5 is architecturally complex (wake-up queues, dependency maps,
cooperative scheduling). Depth-first is sufficient for a functional alpha release.
Phase 5 is a UBC-specific enhancement, not a language requirement.

## Open Questions

- What is the initial feature set for UBCb's first parity checkpoint? (Deferred to
  FOOP-14.)
- How many approval tests must UBCb pass before its first public demonstration?
  (Deferred to FOOP-14.)
- Should SPA1 include a version string or release tag in the CLI? Implementation
  detail.

## References

- [phase1_compiler.md](todo/rust-mvp/phase1_compiler.md) — Rust Phase 1 spec
- [phase2_ubc.md](todo/rust-mvp/phase2_ubc.md) — Rust Phase 2 spec
- [phase3_concatenation.md](todo/rust-mvp/phase3_concatenation.md) — Rust Phase 3 spec
- [phase4_cli.md](todo/rust-mvp/phase4_cli.md) — Rust Phase 4 spec
- [01_phases_overview.md](todo/rust-mvp/01_phases_overview.md) — Phase roster and FOOP mapping
- [ubc2_design.md](ubc1/how/ubc2_design.md) — UBC2 design specification (UBCb reference)
- [ubc2_message_protocol.md](ubc1/how/ubc2_message_protocol.md) — UBCb message protocol
