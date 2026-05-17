---
foop: 19
title: Rename all_terminal to all_constanic in UBCb
author: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
status: Draft
type: Nitpick
created: 2026-05-17
phase: phase-3
supersedes: []
---

# FOOP-91: Rename all_terminal to all_constanic in UBCb

## Abstract

Rename the method `all_terminal` to `all_constanic` in the UBCb (Rust)
implementation. The term "terminal" is not used elsewhere in the Foolish
language — "constanic" is the canonical terminology for FIRs that have
reached their final evaluated state. Mixing terminology creates confusion.

## Motivation

The UBCb codebase uses `all_terminal` as a method name. The Foolish language
and all other implementations (Java, Scala) use "constanic" to describe FIRs
that have fully evaluated. This inconsistency is unnecessary noise — a
single term should be used across all implementations.

## Specification

Rename `all_terminal` to `all_constanic` in the UBCb Rust implementation.

**Location:** `foolish/foolish-ubcb/src/engine.rs`

**Changes:**

1. Line 231: Rename the method definition
   ```rust
   // Before:
   fn all_terminal(&self) -> bool {
   // After:
   fn all_constanic(&self) -> bool {
   ```

2. Line 124: Update the call site
   ```rust
   // Before:
   if self.all_terminal() {
   // After:
   if self.all_constanic() {
   ```

This is a pure rename — no logic, type signature, or behavior changes.

## FIR Impact

None.

## UBC Step Impact

None. The method name changes but the evaluation logic is identical.

## Test Plan

No new tests needed. The existing test suite covers `all_constanic`
(previously `all_terminal`) indirectly through all evaluation tests:

- `cargo test --workspace` must pass after the rename.

## Rejected Alternatives

### A. Keep both names (add alias)

Adding a `#[deprecated]` alias would prolong the inconsistency and add
maintenance burden for no benefit. This is a single-crate rename with one
call site.

### B. Rename to `all_constant`

"Constant" is a distinct Foolish term (the final evaluated value), whereas
"constanic" means "constant in context." The method checks whether all FIRs
in a brane are constanic, so `all_constanic` is the precise term.

## Open Questions

None.

## References

- Code location: `foolish/foolish-ubcb/src/engine.rs`
- Related terminology: `docs/vintage_legacy/STYLES.md` — "Nye", "Constanic",
  "Constant" definitions
