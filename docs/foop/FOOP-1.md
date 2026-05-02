---
foop: 1
title: FOOP Purpose, Process, and Format
author: hc <hc.busy@gmail.com>
status: Final
type: Process
created: 2026-05-01
phase: meta
supersedes: []
---

# FOOP-1: FOOP Purpose, Process, and Format

## Abstract

This document defines the **Foolish Optimization Process** (FOOP), the
mechanism for proposing, discussing, and tracking changes to the Foolish
language and its reference implementation. FOOP is to Foolish what PEP is to
Python, JEP to OpenJDK, and SIP to Scala. This first FOOP — the meta-FOOP —
defines the process itself.

## Motivation

Foolish is a small language, currently with one implementer (BDFL: hc), but
the design history is already non-trivial: which features are deferred, why
`if-then-else` is rejected, how concatenation evaluates, what the FIR contract
is. Without a structured record:

- Decisions get re-litigated whenever someone (including future-self) looks
  at the code.
- The "why" of design choices erodes; only the "what" survives in the source.
- New contributors have no entry point for proposing changes.
- The relationship between language design and implementation phase
  (Phase 1, 2, ...) becomes muddled.

A FOOP is a single document that captures one design decision: the
motivation, the chosen design, the alternatives rejected, the impact on FIR
and UBC, and the test plan. Decisions live as immutable historical artifacts.

## Inspirations

| Process | Maturity | Key takeaway for FOOP |
|---------|---------|------------------------|
| **PEP** (Python) | 1000s, 25+ years | Numbered, immutable; metadata header; status state machine; one canonical index |
| **JEP** (OpenJDK) | ~500, vendor-driven | Strong technical bar; explicit "candidate → completed" gates; each JEP targets a release |
| **SIP** (Scala) | ~50, committee-driven | Requires reference implementation before promotion; "shepherd" role |
| **RFC** (IETF) | 9000+, 50+ years | Long-form; alternatives and prior art encouraged |

FOOP borrows from all four but is intentionally **lighter than PEP/JEP** —
the project is small, the implementer is one person, and process overhead
must not exceed implementation effort.

## Specification

### 1. FOOP Numbering

FOOPs are numbered with **little-endian decimal**: `FOOP-1`, `FOOP-2`,
`FOOP-9`, `FOOP-01`, `FOOP-11`, `FOOP-21`. Numbers are assigned sequentially
in order of submission (not order of acceptance). Numbers are never reused.

**Collation rule**: FOOP-`abcd` sorts by numerical value `dcba`.

That is, read the digits in reverse and sort by the resulting number.
There is no automated tool — sorting is done by hand when needed.

Why little-endian? Two reasons. First, it dovetails with Foolish's general
preference for non-conventional notations where the conventional one is
arbitrary (decimal place values are themselves a positional convention).
Second, it produces a natural batching: FOOPs proposed in the same
"hundred-block" cluster together when sorted, which makes time-correlated
changes visible at a glance.

### 2. FOOP Document Format

Each FOOP is a single Markdown file: `FOOP-N.md` where N is the FOOP number.

Files live in `docs/foop/` in the docs branch.

The file MUST begin with a YAML front matter block:

```yaml
---
foop: <number>                        # integer, no zero padding
title: <short title>                  # one line
author: <name> <email>                # at least one author
status: <status>                      # see status state machine below
type: <Standards|Process|Informational>
created: YYYY-MM-DD
phase: <phase tag>                    # which implementation phase this targets
supersedes: [<foop>, ...]             # list of FOOPs this replaces (often empty)
superseded_by: <foop>                 # if status is Superseded, which FOOP replaces this
implementation: <commit-sha or PR>    # added when status reaches Implementing+
---
```

Body sections (use `##` headings, in this order):

1. **Abstract** — one paragraph, what this FOOP proposes
2. **Motivation** — why this matters; the problem being solved
3. **Specification** — the design itself, in detail
4. **FIR Impact** — new variants? state changes? serialization changes? (skip if N/A)
5. **UBC Step Impact** — new step rules? affects coordination? (skip if N/A)
6. **Test Plan** — how this is verified; new test files, new approval cases
7. **Rejected Alternatives** — at least one; designs considered and not chosen
8. **Open Questions** — known unknowns; what's left to decide
9. **References** — links to prior FOOPs, external docs, prior art

A FOOP without **Motivation** and **Rejected Alternatives** is incomplete.
The rejected-alternatives section is the single most valuable historical
artifact a FOOP produces; a future maintainer who only reads "what we chose"
will eventually re-propose the rejected idea.

### 3. FOOP Types

| Type | Purpose |
|------|---------|
| **Standards** | Adds, removes, or changes language semantics or syntax |
| **Process** | Changes how the project itself operates (FOOP-1 is one) |
| **Informational** | Documents a decision or convention without normative force |

### 4. Status State Machine

```
                         (BDFL accepts)
   Draft ────────────► Brewing ───────────► Implementing ───────► Final
     │                    │                       │
     │                    │                       │
     ▼                    ▼                       ▼
  Withdrawn            Rejected              (rare) Withdrawn
```

| Status | Meaning |
|--------|---------|
| **Draft** | Authored, not yet submitted for review. Editable freely. |
| **Brewing** | Submitted; being actively designed. May still change substantially. |
| **Implementing** | BDFL has accepted the design. Implementation in progress. The FOOP may still be edited to clarify, but the core design is frozen. |
| **Final** | Implementation merged, tests pass, FOOP is closed. Editable only for typo / link fixes — never for content changes. |
| **Withdrawn** | Author retracted before acceptance. |
| **Rejected** | BDFL declined. The FOOP stays in the index as a historical record. |
| **Superseded** | A later FOOP replaces this one. The superseder's number goes in `superseded_by`. |

Once a FOOP is `Final`, `Rejected`, or `Superseded`, the file is immutable.
Corrections happen via a new FOOP that supersedes it.

### 5. Approval

The current BDFL is **hc** (`hc.busy@gmail.com`). All FOOP transitions from
`Brewing → Implementing` and `Implementing → Final` require BDFL approval.

When the project grows beyond one implementer, this section will be
superseded by a new FOOP defining a committee. Until then, BDFL is one
person.

### 6. The Index

`docs/foop/INDEX.md` is the single canonical list of FOOPs. It MUST be kept
in sync with the actual files. It is regenerated by listing all `FOOP-*.md`
files using the little-endian collation rule (see §1).

The index has columns: number, title, status, phase, created, author.

### 7. Phase Targeting

Foolish development proceeds in numbered phases (Phase 1, 2, 3, 4, 5, 6;
see `scala-mvp/foolish-scala/docs/01_phases_overview.md`). Every Standards
FOOP MUST list the phase its implementation targets. Process and
Informational FOOPs may use `phase: meta`.

A FOOP that targets a phase already complete is suspicious: either it
should target a later phase, or it documents a retroactive decision (which
is fine — see §9).

### 8. Lifecycle Workflow

1. **Author creates** `FOOP-N.md` with status `Draft`. The next available N
   is found by scanning `INDEX.md`.
2. **Author submits** by changing status to `Brewing` and committing the
   file plus an `INDEX.md` update.
3. **Discussion happens** in commit messages, issue comments, or directly
   on the FOOP file via further commits. The FOOP body evolves.
4. **BDFL accepts** by changing status to `Implementing` and recording the
   PR or commit that begins the work.
5. **Implementation lands**; status changes to `Final`. The
   `implementation` field gets the merge commit sha.
6. **Or BDFL rejects**; status changes to `Rejected`. A `## Rejection`
   section is appended explaining why.

### 9. Retroactive FOOPs

Decisions made before this process existed may be backfilled as FOOPs to
preserve the record. Retroactive FOOPs:

- Use `created:` set to the date the decision was actually made (best
  guess if unknown), not the date the FOOP was written.
- Note "Retroactive: documents a decision made on <date>" as the first
  line of the Abstract.
- Skip directly to status `Final` if the implementation already exists.

The first batch of retroactive FOOPs (FOOP-2 through FOOP-5) documents
decisions made during the Phase 1 design.

### 10. What FOOPs Are NOT For

- **Bug fixes**: a one-line change to a regex doesn't need a FOOP. Just fix
  it.
- **Refactoring**: moving a class around doesn't need a FOOP.
- **Test additions** that don't change semantics: just add the test.
- **Documentation typos**: just fix them.

Rule of thumb: if you can't articulate the decision in one sentence in the
abstract, it's probably not FOOP-worthy. Conversely, if there are at least
two reasonable options and choosing one over the other has lasting
consequences, write a FOOP.

## FIR Impact

None. FOOP-1 is a process FOOP and has no impact on the implementation.

## UBC Step Impact

None.

## Test Plan

The FOOP process itself is "tested" by usage. Specifically:

- The retroactive FOOPs (2–5) demonstrate that the format can capture
  existing decisions cleanly.
- An `INDEX.md` exists and lists all FOOPs in correct collation order.
- Every Phase-targeted FOOP either matches its phase document's scope or
  the phase document is updated to cite the FOOP.

## Rejected Alternatives

### A. No process at all

Just write code, leave decisions undocumented. **Rejected**: prior
implementations of Foolish (UBC0, UBC1) accumulated tribal knowledge that
got lost between implementation attempts. The "why" of `if-then-else
removal" was hard-won and would have been lost without explicit
documentation. Even one implementer benefits from forcing the rationale
into prose.

### B. Big-endian numbering (FOOP-001, FOOP-002, ..., FOOP-100)

The conventional choice. **Rejected**: little-endian is more in keeping
with Foolish's notational eccentricity, and the natural batching of
"FOOPs from the same era" when correctly sorted is a useful affordance.
The cost — having to write a custom sort — is one shell line.

### C. Markdown-only, no YAML front matter

Simpler. **Rejected**: structured metadata is cheap to write and
machine-readable for index generation. The cost of YAML is one block at
the top of each file.

### D. PEP-style heavyweight process (multiple authors, mailing list,
pre-acceptance review period)

**Rejected**: scale mismatch. Foolish has one implementer. The process
overhead would exceed implementation effort. When the project grows, FOOP
itself can be amended (this is a Process FOOP — see §4 status machine).

### E. Inline FOOPs in the source code (Rust-style RFC pointer comments)

Comments like `// FOOP-3: this is the SearchFir pattern format`.
**Rejected for the document format, recommended as a complement**: the
canonical FOOP lives in `docs/foop/`, but source code MAY reference FOOP
numbers in comments where the connection isn't obvious from context.

## Open Questions

None at time of submission. This FOOP defines the steady-state process; the
natural way to refine it is via subsequent Process FOOPs that supersede it
in part.

## References

- [PEP 1 — PEP Purpose and Guidelines](https://peps.python.org/pep-0001/)
- [JEP 1 — JDK Enhancement-Proposal & Roadmap Process](https://openjdk.org/jeps/1)
- [Scala Improvement Process](https://docs.scala-lang.org/sips/)
- [IETF RFC Editor Style Guide](https://www.rfc-editor.org/styleguide/)
