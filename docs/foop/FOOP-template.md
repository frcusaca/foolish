---
foop: D<NUMBER>
title: <SHORT TITLE — one line, no trailing period>
author: <Name> <email@example.com>
status: Draft
type: Standards
created: <YYYY-MM-DD>
phase: <phase-1 | phase-2 | phase-3 | phase-4 | phase-5 | phase-6 | phase-7 | meta>
supersedes: []
begun: [ ] 
---

# FOOP-<NUMBER>: <TITLE>
FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.** The one
template-specific note: the `foop:` front-matter field may either match the
filename digits directly:
```markdown
foop: <NUMBER>
```
or give the big-endian decimal value, preceded by `D` (so `foop: D42` is the
same as `foop: 24`, i.e. the file `FOOP-24.md`):
```markdown
foop: D<NUMBER>
```
In all cases, the `FOOP-<NUMBER>.md` file name is ultimately the right
numbering.

## Abstract

One paragraph. What does this FOOP propose? Read this and you should know
whether to read the rest.

## Motivation

Why does this matter? What's the problem being solved? What does the world
look like today, and what does it look like after this FOOP is implemented?

## Specification

The design itself. Be precise. If a feature has syntax, give the grammar
fragment. If it adds an FIR variant, give the Rust struct and its `Fir` enum
arm. If it changes a step rule, give the before/after.

Use code blocks for anything formal:

```rust
// example: FIR variant — a per-variant struct carrying its Nyes state,
// plus the arm added to the `Fir` enum in foolish-core/src/fir.rs.
pub struct FooFir {
    pub(crate) bar: String,
    pub(crate) state: Nyes,
}

// enum Fir { ... Foo(FooFir), ... }
```

## FIR Impact

If this FOOP doesn't touch FIR, write "None." and move on.

Otherwise: list every new FIR variant, every state-machine change, every
serialization implication. Include the YAML/JSON shape for any new variant.

## UBC Step Impact

If this FOOP doesn't touch the evaluator, write "None."

Otherwise: list every new step rule. State the before/after. Note any
interaction with existing step rules (especially around constanic
coordination).

## Test Plan

How is this verified?

- New unit tests in `<file>` covering ...
- New `.foo` approval tests at ...
- Existing tests that need updating ...

If a feature can't be cleanly tested, say so explicitly and explain why.

## Rejected Alternatives

At least one alternative MUST be listed, even if it's just "do nothing" with
an explanation of why doing nothing is worse.

### A. <Alternative name>

Description and reason for rejection.

### B. <Alternative name>

Description and reason for rejection.

## Open Questions

Things still to decide. List them as bullets. As they're resolved, edit the
FOOP body and remove from this section. When this section is empty and the
FOOP is `Implementing`, the design is frozen.

- ?

## References

- Prior FOOPs: ...
- External docs: ...
- Code locations: ...
