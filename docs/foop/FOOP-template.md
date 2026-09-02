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

## Plan of Execution for Plan

**How this FOOP's plan gets executed, and by whom.** Plan the execution based on
complexity: match each phase to an agent with sufficient capability for it, rather
than sizing the whole FOOP to one model. Sizing everything to the hardest phase
wastes capability on mechanical work; sizing everything to the easiest puts
judgment calls in the wrong hands.

Fill in the table for THIS FOOP's phases. Harness-specific model names as of
writing: Claude — Opus / Sonnet for judgment, Sonnata for execution; Codex —
GPT-terra; local — Qwen3.8-27B.

| Phase | Character | Needs |
|-------|-----------|-------|
| <N> — <name> | <what kind of work it is> | <larger model / smaller model, and why> |

**Judgment phases** (a larger model) are those where the deliverable IS a
decision: resolving an open question whose answer changes the design, predicting
expected output from the specification, and every `output` → `checked` promotion
review.

**Execution phases** (a smaller model) are those with a fixed target to hit: code
given in the plan, a hand-written expectation to match, a mechanical diff to
verify. An agent that cannot judge "is this right?" can still answer "does this
match the thing a human wrote?"

State what makes the small-model phases safe — inline facts rather than references,
a fixed target per phase, and named stop conditions ("if X, STOP and report") so
trouble is matched against a stated condition rather than recognized unaided.

**What must not be delegated**, regardless of model size (AGENTS.md §"The agent is
responsible for correctness"): every `output` → `checked` promotion; any
hand-written expectation the FOOP's design depends on; any decision to change a
crate the FOOP promised not to touch; marking any Verified-tier test `#[ignore]`.

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
