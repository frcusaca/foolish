---
foop: 71
title: FOOP attachments — structured side documents with frontmatter
author: Claude Code / claude-opus-5 (directed by the human)
status: Draft
type: Process
created: 2026-09-24
phase: meta
supersedes: []
begun: [ ]
---

# FOOP-17: FOOP attachments — structured side documents with frontmatter

## Abstract

A FOOP is two files today: `FOOP-#.md` (spec) and `FOOP-#.plan.md` (plan). In practice agents
and humans keep writing a **third** kind — bug lists, review feedback, tests captured during
design, addenda — as `FOOP-#.<something>.md`. Eight such files exist, using **seven different
ad-hoc suffixes**, and **none carries frontmatter**, so no tooling can see them: they do not
appear in `foop_check.py`, they are invisible to status scans, and a listing that filters on
`status:` reports them as malformed.

This FOOP makes that third kind a **first-class, structured attachment**: a declared set of
kinds, required frontmatter that binds an attachment to its parent FOOP, and tooling that lists
and validates them.

## Motivation

### §1 The pattern already exists and is substantial

Measured on `jia` at `0df1865b`:

| File | Lines | What it is |
|---|---|---|
| `FOOP-16.addendum.md` | 144 | a specification addendum |
| `FOOP-32.bugs.md` | 212 | a snapshot bug repair list |
| `FOOP-52.bugs.md` | 466 | a snapshot bug repair list |
| `FOOP-62.deepseek-feedback.md` | 334 | external model review |
| `FOOP-62.feedback-synthesis.md` | — | synthesis of that review |
| `FOOP-62.mimo-feedback.md` | — | external model review |
| `FOOP-64.einmo-hardening.md` | 184 | a sub-plan |
| `FOOP-75.tests.md` | 407 | tests written during design |

These are not scratch notes — several exceed their parent's plan in length. The content is
worth keeping; what is missing is structure.

### §2 What goes wrong without it

- **Invisible to tooling.** `foop_check.py` does not know they exist. A status scan filtering on
  `status:` classifies all eight as unparseable, inflating any "open FOOPs" count (observed
  2026-09-24: a listing reported 48 open, of which 9 were these files plus the template).
- **No stated relationship.** Nothing says whether `FOOP-32.bugs.md` is in force, historical, or
  superseded — or whether its bugs were fixed. A reader must infer from prose.
- **Suffix drift.** Seven suffixes for eight files. `deepseek-feedback` and `mimo-feedback` are
  the same kind under different names; `einmo-hardening` is a plan, not a kind at all.
- **No lifecycle.** A FOOP's own status has a defined progression; an attachment has none, so a
  completed bug list looks exactly like an open one.

## Specification

### §3.1 Naming

`FOOP-<NUMBER>.<kind>.md`, where `<kind>` is one of the declared kinds in §3.2. The `<NUMBER>`
is the parent's, in the same little-endian form as the parent filename. `FOOP-#.plan.md` is
**not** an attachment — the plan stays the second of the two core files.

### §3.2 Declared kinds

| Kind | For |
|---|---|
| `addendum` | specification material added after the spec was written |
| `bugs` | a defect list discovered while executing the FOOP |
| `feedback` | review of the FOOP by a human or another model |
| `tests` | tests written during design, before implementation |
| `notes` | working material that does not fit the above |

A kind not on this list is a validation error. `feedback` **subsumes** today's
`deepseek-feedback` / `mimo-feedback` / `feedback-synthesis`: the reviewer's identity belongs in
frontmatter (`source:`), not in the filename. Several feedback attachments on one FOOP are
distinguished by `source:` plus `created:`.

### §3.3 Required frontmatter

```yaml
---
foop: <parent's sort key>
attachment: <kind>            # one of §3.2
parent: FOOP-<NUMBER>         # the parent's filename stem
title: <one line>
author: <who wrote it>
status: <Draft | Active | Resolved | Superseded>
created: <YYYY-MM-DD>
source: <optional — for feedback: who reviewed>
---
```

`status` for an attachment means:

- **Draft** — being written.
- **Active** — in force; a reader must take it into account.
- **Resolved** — its content has been acted on (bugs fixed, feedback incorporated). Kept as
  history.
- **Superseded** — replaced; `supersedes:`/`superseded_by:` name the other document.

An attachment does **not** carry `begun:` — that belongs to a plan, and attachments are not
executed.

### §3.4 Tooling

`docs/foop/scripts/foop_check.py` gains:

- `attachments [<NUMBER>]` — list attachments, optionally for one parent.
- Validation in `check`: every `FOOP-*.*.md` that is not `.plan.md` must carry §3.3's
  frontmatter, its `parent:` must exist, and its `attachment:` must be a declared kind.
- The existing `check` must **stop counting attachments as FOOPs** — the current sort-key
  consecutiveness check reads only `FOOP-<N>.md`, so this is a guard against future drift
  rather than a fix to a present miscount.

### §3.5 Migrating the eight existing files

Each gains frontmatter; **none is renamed except the three feedback files**, whose kind moves
into `source:`:

| Today | Becomes | `source:` |
|---|---|---|
| `FOOP-62.deepseek-feedback.md` | `FOOP-62.feedback.md` | `deepseek` |
| `FOOP-62.mimo-feedback.md` | `FOOP-62.feedback.md` *(collision — see below)* | `mimo` |
| `FOOP-62.feedback-synthesis.md` | `FOOP-62.notes.md` | — |

**The collision is real and must be resolved in the plan**, not hand-waved here: two `feedback`
attachments on FOOP-62 cannot share a filename. Either the filename admits a discriminator
(`FOOP-62.feedback.deepseek.md`) or the two are merged into one document with two sections. The
plan decides, with the human.

`FOOP-64.einmo-hardening.md` describes itself as "(plan)". Whether it is genuinely a plan for a
FOOP-64 that has no `.plan.md`, or an attachment of kind `notes`, must be checked against
FOOP-64 before assigning a kind.

## FIR Impact

None — a documentation and tooling change.

## UBC Step Impact

None.

## Test Plan

- `foop_check.py check` fails on an attachment missing frontmatter, with an unknown kind, or
  with a `parent:` that does not exist.
- `foop_check.py attachments` lists all eight migrated files, and `attachments 62` lists only
  FOOP-62's.
- `foop_check.py check` still reports consecutive sort keys after migration — attachments must
  not perturb the FOOP numbering.
- A status scan over `docs/foop/` reports zero unparseable documents.

## Plan of Execution for Plan

**Scheduled immediately after FOOP-26** (the human, 2026-09-24). It is independent of the
evaluator work, so it does not block Track 6; placing it after 26 means the migration happens
once the FOOPs most likely to *acquire* attachments (26, 46) have finished generating them.

Mechanical throughout — adding frontmatter to eight existing files and extending a Python
helper — so it suits a smaller agent, with two exceptions that need judgement and the human:
the FOOP-62 feedback-filename collision (§3.5) and classifying `FOOP-64.einmo-hardening.md`.

## Rejected Alternatives

- **Fold attachments into the spec or plan.** Rejected: a 466-line bug list inside a spec buries
  the specification, and the plan is a checkbox roadmap, not a place for external review.
- **Leave them unstructured.** They are already numerous and long enough that tooling cannot
  ignore them without miscounting, and a reader cannot tell an in-force addendum from a
  historical one.
- **One free-form `notes` kind only.** Rejected: `bugs` and `feedback` have real lifecycles
  (resolved / incorporated) that a generic kind cannot express.

## Open Questions

- The FOOP-62 feedback filename collision (§3.5) — discriminator in the filename, or merge?
- Is `FOOP-64.einmo-hardening.md` a missing `.plan.md` or an attachment?
- Should an attachment's `status: Active` block its parent from reaching `Complete`? Probably
  yes for `bugs`, probably no for `feedback` — worth deciding rather than leaving implicit.

## References

- `foop.md` — the authoritative FOOP process; this FOOP extends its file-layout section.
- `docs/foop/scripts/foop_check.py` — the helper gaining `attachments`.
- AGENTS.md §"FOOP (Foolish Optimization Process)" — states the two-file system this amends.

## Last Updated

**Date**: 2026-09-24
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created. Makes `FOOP-#.<kind>.md` side documents first-class: five declared kinds
(`addendum`, `bugs`, `feedback`, `tests`, `notes`), required frontmatter binding an attachment to
its parent with its own four-state status, and `foop_check.py` support for listing and validating
them. Motivated by eight existing such files using seven ad-hoc suffixes with no frontmatter,
invisible to tooling and inflating open-FOOP counts. Records two migration problems that need the
human rather than a default: two FOOP-62 feedback documents would collide on one filename, and
`FOOP-64.einmo-hardening.md` may be a missing plan rather than an attachment.
