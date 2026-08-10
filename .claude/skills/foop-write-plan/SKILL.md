---
name: foop-write-plan
description: "MUST USE when CREATING or PLANNING a FOOP (Foolish Optimization Process) — writing the specification file (FOOP-#.md) and the plan file (FOOP-#.plan.md). Covers: what a FOOP is, the two-file system (spec + plan), little-endian numbering (FOOP-1→FOOP-9→FOOP-01→FOOP-11→FOOP-21...), the foop_check.py helper script (check/get_last/gen_next/list), naming convention (dash in filenames, space in prose), the spec template (frontmatter fields foop/title/status/phase/begun, body sections Abstract/Motivation/Specification/FIR Impact/UBC Step Impact/Test Plan/Rejected Alternatives/Open Questions/References), the D-prefix sort-key rule, plan construction rules (ordered/concrete/trackable, worktree lifecycle checkboxes, sub-tasks, variable expansion), checkbox format (timestamp on next indented line), sub-task splitting, worktree branch tracking (naming, creation, lifecycle checkboxes), comprehensive snapshot tests (input/foop/<NUMBER>/comprehensive.foo), and the minimal plan skeleton. Gives exact copy-pasteable commands with <NUMBER> and <SHORT_DESCRIPTION> placeholders. Triggers: 'create foop', 'new foop', 'write foop', 'foop spec', 'foop plan', 'foop template', 'foop numbering', 'foop frontmatter', 'foop_check gen_next', 'plan a foop', 'foop worktree setup', 'foop comprehensive test'."
---

# FOOP — Writing and Planning

This skill covers **creating** FOOPs: writing the specification and constructing the plan. For finding, executing, backburnering, or cancelling existing FOOPs, use the `foop-use-maintain` skill.

> **Authoritative source**: `foop.md` at the repository root. When this skill and `foop.md` appear to disagree, `foop.md` wins. Read `foop.md` before creating or editing any FOOP.

---

## What a FOOP Is

A FOOP (Foolish Optimization Process) is the Foolish equivalent of Python's PEP or Rust's RFC. It proposes, discusses, and tracks changes to the Foolish language and its reference implementations.

A FOOP progresses through statuses: `Draft` → `Brewing` (ready for BDFL review) → `Final` (accepted) → `Implementing` (active coding) → complete. Each FOOP is assigned to a `phase` (phase-1 through phase-7, or `meta` for process documents).

---

## The Two Files of a FOOP

Every FOOP is expressed as (up to) two separate files that share the same `FOOP-<NUMBER>` stem:

| File | Purpose | Answers |
|------|---------|---------|
| `FOOP-#.md` | **Specification** — the proposal, motivation, design, semantics, discussion. | *What* and *why* |
| `FOOP-#.plan.md` | **Plan** — a checkboxed, sequentially-executed breakdown of the work. (Note the lowercase `.plan.md` extension.) | *How* and *in-what-order* |

**Executing a FOOP requires reading BOTH files.** The plan assumes the context of the specification; do not act on `FOOP-#.plan.md` without first reading `FOOP-#.md`. The plan is meant to be executed sequentially from top to bottom.

---

## FOOP Numbering — Little-Endian (Critical)

FOOP numbering is **little-endian**: the filename digits ARE the identifier, but they sort in reverse. Chronological order (oldest → newest):

```
FOOP-1, FOOP-2, FOOP-3, ... FOOP-9, FOOP-01, FOOP-11, FOOP-21, FOOP-31, FOOP-41, FOOP-51, FOOP-61, ...
```

- FOOP-9 is the one **before** FOOP-01.
- The digits in `FOOP-01` are `0` then `1` — read as "ten" when reversed, so its sort key is `10`.
- FOOP-21 → sort key 12. FOOP-51 → sort key 15.

| Filename | Identifier | Sort key (frontmatter only) |
|----------|------------|-----------------------------|
| `FOOP-9.md` | FOOP-9 | 9 |
| `FOOP-01.md` | FOOP-01 | 10 |
| `FOOP-21.md` | FOOP-21 | 12 |
| `FOOP-51.md` | FOOP-51 | 15 |

The **filename digits ARE the identifier**. The `foop:` frontmatter field is a separate numeric sort key (the digits reversed). Do NOT use the sort-key value as the identifier in prose.

---

## Naming Convention

| Context | Form | Example |
|---------|------|---------|
| Filename, code, formal citation | `FOOP-<NUMBER>` (dash) | `FOOP-01.md` |
| Prose / sentences | `FOOP <NUMBER>` (space) | "FOOP 01 and FOOP 11 are pre-teen FOOPs." |

The space form in prose reduces digit-reversal errors: writing "FOOP 01" makes it harder to accidentally type "FOOP 10".

---

## File Locations

| What | Path |
|------|------|
| FOOP specs & plans | `docs/foop/FOOP-<NUMBER>.md` and `docs/foop/FOOP-<NUMBER>.plan.md` |
| Index | `docs/foop/INDEX.md` (canonical list, sorted by number) |
| Template | `docs/foop/FOOP-template.md` |
| Helper script | `docs/foop/scripts/foop_check.py` |
| Meta-FOOP (defines the process itself) | `docs/foop/FOOP-1.md` |

---

## The Numbering Helper Script

Use `docs/foop/scripts/foop_check.py` to manage FOOP numbering. Run it before creating a new FOOP and periodically to catch drift.

```bash
python3 docs/foop/scripts/foop_check.py check     # verify consecutive numbering
python3 docs/foop/scripts/foop_check.py get_last  # most recent FOOP
python3 docs/foop/scripts/foop_check.py gen_next  # filename for next FOOP
python3 docs/foop/scripts/foop_check.py list      # all FOOPs in chronological order
```

**When creating a new FOOP, ALWAYS run `gen_next` first** to get the correct filename and identifier. The script handles the little-endian encoding for you.

---

## Task: Create a New FOOP Specification

### Step 1 — Get the next FOOP number

```bash
python3 docs/foop/scripts/foop_check.py gen_next
```

Output: `FOOP-<NUMBER>\tFOOP-<NUMBER>.md\t(sort key <N>)`

Remember the `<NUMBER>` — you will substitute it everywhere below.

### Step 2 — Copy the template

```bash
cp docs/foop/FOOP-template.md docs/foop/FOOP-<NUMBER>.md
```

### Step 3 — Fill in the frontmatter

Edit `docs/foop/FOOP-<NUMBER>.md`. The frontmatter must be:

```yaml
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
```

**Frontmatter field rules:**

| Field | Rule |
|-------|------|
| `foop` | The sort key. Two accepted forms (see "The `foop:` field" below). |
| `title` | Short title, one line, no trailing period. |
| `status` | Start as `Draft`. Lifecycle: `Draft` → `Brewing` → `Final` → `Implementing` → complete. |
| `type` | Typically `Standards`. |
| `created` | Date in `YYYY-MM-DD` format. |
| `phase` | One of `phase-1` through `phase-7`, or `meta` for process documents. |
| `supersedes` | List of FOOP identifiers this one replaces. Empty list `[]` if none. |
| `begun` | `[ ]` (not yet started). Changed to `[x]` when work begins (see `foop-use-maintain` skill). |

**The `foop:` field — two accepted forms:**

1. **`D` prefix (big-endian decimal):** `foop: D<NUMBER>` — the `D` means "the filename digits reversed as a big-endian decimal." So `foop: D42` = sort key 42 = file `FOOP-24.md`.
2. **Direct value:** `foop: <NUMBER>` (no `D`) — the literal sort-key value directly.

In all cases, the **filename** (`FOOP-<NUMBER>.md`) is the ultimate identifier and the right numbering. The `foop:` field is only a sort key for tooling.

### Step 4 — Fill in the body

The body follows this structure (from the template):

```markdown
# FOOP-<NUMBER>: <TITLE>

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

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

Use code blocks for anything formal.

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
```

---

## Task: Create the Plan File

A FOOP is not actionable without a plan. Write `docs/foop/FOOP-<NUMBER>.plan.md` (lowercase `.plan.md`). **Read `FOOP-<NUMBER>.md` first** — the plan is derived from the specification; the spec exists before the plan, so you can name a concrete `short_description` and decompose the spec into ordered tasks.

### Plan Construction Rules

Build the plan so that:

1. **Tasks are listed in the order they must be executed** (top to bottom).
2. **Each task is concrete and trackable** on its own.
3. **Worktree lifecycle tasks** (create / verify / merge / cleanup) appear as **explicit checkboxes** at the appropriate points (see "Worktree Branch Tracking" below).
4. **Tasks that prove larger than expected split into indented sub-tasks** (see "Sub-Tasks" below).
5. **All RHS variables should be expanded and literally placed** into the plan file as the plan is being created. No `${WORKTREE_*}` placeholders remain when work begins — fill them with real paths.
6. If the spec has research/experimentation (web search, historic docs, prototyping), those should be **clearly documented in the FOOP file**, and the plan steps shall, where needed, contain **section or sub-section header pointers** into the FOOP file. A large todo with sub-tasks may have several "read section X of FOOP-<NUMBER>.md" as its first few checkboxes.
7. **Sanity-check sub-tasks** may be installed where ambiguity exists — e.g. "[ ] sub-agent please consult with primary agent or human regarding the current approach to..." These can be installed or removed by the planning agent as specification, clarification, design, and planning progresses.
8. **Once work begins on a FOOP, all updates — including to the foop folder — MUST be written ONLY to the worktree.** This continues until merge time.
9. **Every phase ends with a test-gate checkbox.** The last checkbox of every implementation phase (and a final one right before the merge STOP) is, verbatim:
   ```
   - [ ] Run all tests — old and new — and make sure they all pass correctly.
   ```
   A phase is not complete until this box is checked. The contract behind it (a failing einmo test is broken code, not a stale baseline; never `promote` over a foreign FOOP's divergent baseline; the three `output`/`checked`/`verified` stages are a contract) lives in `rust_instructions.md` §"Phase-by-phase testing discipline" — the plan installs the checkbox; `rust_instructions.md` explains why.
10. **Every `output→checked` promotion gets its own review-gate block.** Promotion is a *correctness claim made by the agent*, never a bookkeeping step, and it is **never** installed as a bare one-line checkbox. Wherever a phase produces new or changed einmo output, the plan installs the full **Promotion Review Gate** block below (see "Promotion Review Gate"), with **one `[ ]` sub-task per einmo case** to be promoted, named individually. A block that names no cases is not a gate; a gate whose per-case boxes are checked faster than the cases could be read is a falsified record.

### Promotion Review Gate

`einmo promote output to checked` writes the **frozen expected-output contract**. Checking a promotion box asserts: *"I read this case's OUTPUT statement by statement, and I can say in my own words why each line is what the specification requires."* It does not assert "the suite is green" and it does not assert "the evaluator produced this."

Install this block — expanded with the **real case names**, one sub-task each — before any `einmo promote` in the plan:

```markdown
- [ ] Review and promote `output` → `checked` for FOOP-<NUMBER>'s einmo cases
  - [ ] Confirm the rest of the suite is green — no foreign-FOOP baseline diverges
        (a foreign divergence is a regression I introduced: fix the code, do not promote)
  - [ ] Confirm no case below has a `verified/` twin (if one does: STOP, ask the human)
  - [ ] Re-read the in-force specification for each feature under test:
        FOOP-<NUMBER>.md §Specification, plus README.md §"The Unknown" for any NK result
  - [ ] Review `foop/<NUMBER>/<case_1>` — every OUTPUT statement justified
  - [ ] Review `foop/<NUMBER>/<case_2>` — every OUTPUT statement justified
        (…one checkbox per case; name them all, never "…and the rest")
  - [ ] Write the justification summary into the plan or commit message:
        for each case, what it demonstrates and why its result is spec-correct
  - [ ] Report ALL accumulated doubts to the human in ONE statement — or record
        "no doubts". Blocking doubts stop here; non-blocking ones are reported alongside.
  - [ ] Run `einmo promote output to checked foolish-ubca/einmo_suite`
  - [ ] Re-run `cargo test -p foolish-ubca --lib -- einmo_gate_checked` — must exit 0
```

**What "every OUTPUT statement justified" requires**, per case — this is the work, not a formality:

- **Statement by statement.** Read each OUTPUT line against the INPUT statement that produced it. State *why that value is what the specification mandates*. "The evaluator emitted it" is the thing being tested, not a justification for it.
- **Be skeptical of `NK`.** A search settling NK is the narrow, exceptional outcome. Name which legitimate case applies (see `README.md` §"The Unknown" and FOOP-23 §Specification — anchored miss ⇒ NK, unanchored miss ⇒ ECONSTANIC). If you cannot name it, trace it with the `foolish-debugging` skill; do not promote.
- **Statement names are specification.** `hit = ?…` asserts the search finds its target; `miss = ?…` asserts it does not. A `hit` that yields NK is a contradiction between the test's own claim and its result — resolve it (fix the bug the name predicted, or rename with a comment explaining why) before promoting, never by promoting past it.
- **Coherence, not just conformance.** Ask whether the result fits the rest of Foolish: does an analogous existing case behave the same way? Would a Foolisher reading only the spec predict this output? A result that is locally defensible but inconsistent with a sibling feature is a design bug worth raising, not a baseline to freeze.
- **Step counts and alarms count too.** They are part of the OUTPUT contract. A step count that jumped sharply for a feature that should not have changed cost is a signal to investigate.

If any case fails review, **do not promote any of them**. Fix the code (or revise the test's input/statement names, which is its own reviewable change) and re-run the gate.

### Checkbox Format

When an item is checked off, **always place a timestamp (to the minute) on the next line with indent into the bulleted list**:

```markdown
- [ ] Task not yet done
- [x] Task completed                    ← bad (no timestamp)
- [x] Task completed                    ← good
      (2026-05-06 13:11)                ← timestamped properly
```

This gives both agents and humans a clear view of how work is progressing over time.

### Sub-Tasks

If a task proves larger than expected and splits into multiple sub-tasks, indent them under the parent. Use completed sub-tasks to justify why the split occurred:

```markdown
- [ ] Merge `foop-<NUMBER>-<SHORT_DESCRIPTION>` to `jia`
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] Check and make sure current foop has, and passes, a "comprehensive" snaptest. Input name: `input/foop/<NUMBER>/comprehensive.foo` (reserved for this foop). Agent generates and verifies; human gives final signed approval.
  - [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [x] Detected complex merge situation requiring additional work
        (2026-05-06 14:00)
  - [ ] Update `foop-<NUMBER>-<SHORT_DESCRIPTION>` to follow new coding style
  - [ ] Update `foop-<NUMBER>-<SHORT_DESCRIPTION>` to use new API call convention
  - [x] Merged breaking changes from `jia`
        (2026-05-06 14:31)
  - [ ] Repair ALL tests in `jia` in /home/<USER>/foolish-rust
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present human with the `cd $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>` command and ask them to review snapshots BEFORE checking the parent checkbox.
  - [ ] Cleanup $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>
    - [ ] Check that .plan.md has all but Cleanup checkboxes completed
    - [ ] Remove $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>
    - [ ] This is the last sub-task checkbox to be checked in this block
```

This pattern is common because Foolish uses `git merge` (not rebase), so merge conflicts on `jia` may trigger follow-up repair work.

### Worktree Branch Tracking

If a worktree branch is used for implementation, the plan **must** document the lifecycle of that worktree as explicit, separate checkbox tasks placed at appropriate points in the plan.

**Variables** (expand to literals before the plan is finalized):

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=$(pwd)
WORKTREE_BRANCH_NAME=foop-<NUMBER>-<SHORT_DESCRIPTION>
WORKTREE_FULL_FS_PATH=$(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>
```

**The `short_description`** in the path should be generated as part of the `.plan.md` generation. It is possible because the specification is already made. The `foop-<NUMBER>` suffix should match the name of the foop file as well as the plan file. Once set, this path name is fixed.

**Worktree creation command** (from the `${WORKTREE_ORIGIN_BRANCH}` branch and path):

```bash
# cd $WORKTREE_ORIGIN_PATH    ## User normally starts in this directory
# git checkout $WORKTREE_ORIGIN_BRANCH  ## Already on this branch normally
git worktree add -b "$WORKTREE_BRANCH_NAME" "$WORKTREE_FULL_FS_PATH"
cd "$WORKTREE_FULL_FS_PATH"
# Now commence work here.
```

**Permission scope**: An agent with permission to work on the main foolish directory also has permission to work on a worktree added from the foretias directory. If asking for permission, ask once for the **entire worktree branch path** (`$WORKTREE_FULL_FS_PATH`), not a subdirectory.

**Worktree checkboxes in the plan** (all variables expanded to literals):

```markdown
- [ ] Create worktree at $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION> with branch `foop-<NUMBER>-<SHORT_DESCRIPTION>`
...
  (implementation tasks here)
...
- [ ] Verify all work is complete in $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION> and committed to `foop-<NUMBER>-<SHORT_DESCRIPTION>`
- [ ] Merge `foop-<NUMBER>-<SHORT_DESCRIPTION>` to `jia`
```

> **Branch naming (no prefix)**: The branch name is `foop-<NUMBER>-<SHORT_DESCRIPTION>` — **bare, with no `foop/` prefix** — and must be **identical** to `WORKTREE_BRANCH_NAME` and to the worktree directory's basename. The `foop-<NUMBER>` stem must match the FOOP filename.
>
> Current practice (FOOP-13, FOOP-23, FOOP-33, FOOP-54, FOOP-84): `foop-23-value-search`,
> `foop-54-einmo`. A `foop/`-prefixed form appears in older pre-2026-06 plans (`foop/12-alarms`)
> and in some `foop.md` examples — **do not copy it**. Mixing the two within one plan is a real
> hazard: the create checkbox makes one branch while the merge checkbox names another that does
> not exist.

> **Variable expansion**: The branch names and paths in the plan should reflect **expanded** `$HOME`, `$WORKTREE_BRANCH_NAME` and `$WORKTREE_ORIGIN_BRANCH`. Fillers such as the literal `$WORKTREE_ORIGIN_BRANCH` should be replaced with real values before starting work on the plan. `${HOME}` should be the full path when the plan is generated.

### Comprehensive FOOP Test

Every FOOP has the right — and the obligation — to generate a **comprehensive snapshot test** that thoroughly exercises the new feature interacting with existing features.

| Attribute | Value |
|-----------|-------|
| **Input file** | `foolish-ubca/einmo_suite/input/foop/<NUMBER>/comprehensive.foo` |
| **Name** | Reserved for this FOOP alone |
| **Purpose** | Coverage of high-value feature combinations and edge cases that per-phase approval tests may not reach. Slight repetition of earlier tests is acceptable if it serves coverage. |
| **Scope** | Mix new features with old — value search inside nested branes, contexted operators chained with dot access, expression patterns referencing ancestral names, combined name+value with head/tail, etc. The test should be large enough to exercise at least one path through every new operator or predicate variant. |
| **Process** | The agent generates the `.foo` input, runs it through the approval test suite, and **reviews the output statement by statement via the Promotion Review Gate** before promoting. Final approval requires human review and formal signed acceptance. |
| **Placement in plan** | A checkbox task "Write and verify `input/foop/<NUMBER>/comprehensive.foo`" should appear in the plan, **after all implementation phases and before the merge STOP**. |

### Minimal Plan Skeleton

```markdown
# FOOP-<NUMBER>.plan — <SHORT_DESCRIPTION>

- [ ] Begin work: commit FOOP-<NUMBER>.md and FOOP-<NUMBER>.plan.md to origin, check `begun: [x]` in frontmatter
      (YYYY-MM-DD HH:MM)
- [ ] Create worktree at $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION> with branch `foop-<NUMBER>-<SHORT_DESCRIPTION>`
- [ ] (read §<SECTION> of FOOP-<NUMBER>.md)
- [ ] <implementation task 1>
- [ ] <implementation task 2>
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] Review and promote `output` → `checked` for FOOP-<NUMBER>'s einmo cases
      (expand the full Promotion Review Gate block, one sub-task per named case)
- [ ] Write and verify `foolish-ubca/einmo_suite/input/foop/<NUMBER>/comprehensive.foo`
- [ ] Run all tests — old and new — and make sure they all pass correctly.
- [ ] Review and promote `output` → `checked` for `foop/<NUMBER>/comprehensive`
      (expand the full Promotion Review Gate block)
- [ ] Verify all work is complete in $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION> and committed to `foop-<NUMBER>-<SHORT_DESCRIPTION>`
- [ ] Merge `foop-<NUMBER>-<SHORT_DESCRIPTION>` to `jia`
- [ ] Cleanup worktree at $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>
```

---

## Quick Reference — All Creation Commands

```bash
# ── Numbering ──
python3 docs/foop/scripts/foop_check.py gen_next   # get next FOOP number
python3 docs/foop/scripts/foop_check.py check      # verify no gaps

# ── Create spec ──
cp docs/foop/FOOP-template.md docs/foop/FOOP-<NUMBER>.md
$EDITOR docs/foop/FOOP-<NUMBER>.md                  # fill frontmatter + body

# ── Create plan ──
$EDITOR docs/foop/FOOP-<NUMBER>.plan.md             # write from spec, expand all variables

# ── Worktree setup (for plan construction reference) ──
git worktree add -b foop-<NUMBER>-<SHORT_DESCRIPTION> \
    $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>
```

---

## Safety Invariants

1. **Read `foop.md` before creating or editing any FOOP.** This skill is a cookbook; `foop.md` is the authority.
2. **Always run `gen_next` before creating a new FOOP.** Never guess the next number.
3. **All variables must be expanded to literals** in a finalized plan. No `${WORKTREE_*}` placeholders remain when work begins.
4. **At least one Rejected Alternative must be listed** in the spec, even if it's "do nothing."
5. **Never auto-accept snapshots.** Do not run `cargo insta accept` / `INSTA_UPDATE=always`. Human review required for all snapshot tests.
6. **Never start Phase+ work when tests are broken.** Fix or disable (with human permission) first.
7. **Never commit from inside this skill** unless the user explicitly asks.
8. **Never promote over a foreign FOOP's divergent einmo baseline.** A failing einmo test (output≠checked) is broken code, not a stale baseline — fix your code so the pre-existing baseline passes again. `einmo promote output→checked` is only for your own FOOP's new tests, after the rest of the suite is green; never a remedy for a failing test. If the divergent baseline has a `verified/` twin, it is frozen — do not touch it without a human reviewer's key. See `rust_instructions.md` §"Phase-by-phase testing discipline."
9. **Never promote a case you have not read.** "It is my own FOOP's test" makes promotion *permissible*, not *justified* — the justification is a statement-by-statement reading against the in-force specification. Every plan that promotes installs the **Promotion Review Gate** with one named sub-task per case; a plan with a bare `- [ ] einmo promote …` line is malformed, and a gate whose per-case boxes were checked without the cases being read is a false record of work.

---

## Last Updated

**Date**: 2026-08-09
**Updated By**: Claude Code / claude-opus-5
**Changes**: Added the **Promotion Review Gate** — a full checkbox block (one named sub-task per
einmo case, plus green-suite / `verified`-twin / spec-reread / justification-summary steps) that
every plan must install before any `einmo promote`, replacing bare one-line promote tasks. Added
**plan-construction rule 10** and **safety invariant 9** (never promote a case you have not read;
"it is my own FOOP's test" makes promotion permissible, not justified), and spelled out what
"every OUTPUT statement justified" requires — statement-by-statement reading, skepticism toward
NK, statement names as specification, coherence with sibling features, step counts and alarms —
tempered by reasonable effort, with doubts accumulated and reported to the human in ONE statement
at the end of the pass rather than halting mid-review.
Wired the gate into the minimal plan skeleton and the comprehensive-test process row. Motivated by
observed agent behavior of promoting `output→checked` with zero inspection. Earlier: rule 9's
per-phase test-gate checkbox and invariant 8 (never promote over a foreign divergent baseline),
from the FOOP-33 Phase-3 `default_equal` regression.
