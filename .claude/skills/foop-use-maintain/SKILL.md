---
name: foop-use-maintain
description: "MUST USE when FINDING, EXECUTING, UPDATING, BACKBURNERING, or CANCELLING existing FOOPs (Foolish Optimization Process). Covers: listing/finding FOOPs in chronological order (little-endian ls|rev|sort -V|rev, foop_check.py list/get_last/check), the two-file system (must read both spec and plan before executing), status lifecycle (Draft→Brewing→Final→Implementing→complete), plan execution flow (begin→worktree→all-work-in-worktree→commit-regularly→merge), checkbox lifecycle (completing with timestamp, backburnering with [x] backburnered, cancelling with [x] Canceled + [-] per-item), worktree execution (create from jia, work only in worktree, merge to jia, cleanup), sub-task execution patterns (parent not checked until children done, the merge STOP pattern, conflict repair because Foolish uses merge not rebase), comprehensive test verification, human communication protocol (PTAL reminder with FOOP number and worktree path), and safety invariants. Gives exact copy-pasteable commands with <NUMBER> and <SHORT_DESCRIPTION> placeholders. Triggers: 'find foop', 'list foop', 'execute foop', 'foop status', 'foop execution', 'check foop checkbox', 'backburner foop', 'cancel foop', 'deprecate foop', 'foop merge', 'foop cleanup', 'foop worktree', 'foop begun', 'foop progress', 'resume foop', 'foop PTAL'."
---

# FOOP — Using and Maintaining

This skill covers **finding, executing, updating, backburnering, and cancelling** existing FOOPs. For creating a new FOOP spec or writing a plan, use the `foop-write-plan` skill.

> **Authoritative source**: `foop.md` at the repository root. When this skill and `foop.md` appear to disagree, `foop.md` wins. Read `foop.md` before executing or maintaining any FOOP.

---

## The Two Files of a FOOP (Read Both)

Every FOOP is expressed as (up to) two separate files:

| File | Purpose |
|------|---------|
| `FOOP-<NUMBER>.md` | **Specification** — the *what* and *why*. |
| `FOOP-<NUMBER>.plan.md` | **Plan** — the *how* and *in-what-order* (lowercase `.plan.md`). |

**Executing a FOOP requires reading BOTH files.** The plan assumes the context of the specification; do not act on `FOOP-<NUMBER>.plan.md` without first reading `FOOP-<NUMBER>.md`. The plan is meant to be executed sequentially from top to bottom.

---

## FOOP Numbering — Little-Endian (for finding)

FOOP numbering is **little-endian**: the filename digits ARE the identifier, but they sort in reverse. Chronological order (oldest → newest):

```
FOOP-1, FOOP-2, ... FOOP-9, FOOP-01, FOOP-11, FOOP-21, FOOP-31, FOOP-41, FOOP-51, FOOP-61, ...
```

FOOP-9 is the one **before** FOOP-01. The `foop:` frontmatter field is a separate sort key (digits reversed) — do NOT use it as the identifier in prose.

| Context | Form | Example |
|---------|------|---------|
| Filename, code, formal citation | `FOOP-<NUMBER>` (dash) | `FOOP-01.md` |
| Prose / sentences | `FOOP <NUMBER>` (space) | "FOOP 01 and FOOP 11 are pre-teen FOOPs." |

---

## Task: Find and List FOOPs

### List all FOOPs in chronological order

**Always** use this command to establish ordering. Do not `ls` naively — little-endian breaks alphabetical sort.

```bash
ls docs/foop | rev | sort -V | rev
```

Or use the helper script (gives identifiers + sort keys):

```bash
python3 docs/foop/scripts/foop_check.py list
```

### Find the most recent FOOP

```bash
python3 docs/foop/scripts/foop_check.py get_last
```

Output: `FOOP-<LAST_NUMBER>\tFOOP-<LAST_NUMBER>.md\t(sort key <N>)`

### Check numbering integrity

Run periodically to catch drift (gaps, duplicates):

```bash
python3 docs/foop/scripts/foop_check.py check
```

### Find a specific FOOP's files

```bash
ls docs/foop/FOOP-<NUMBER>*.md
# Shows: docs/foop/FOOP-<NUMBER>.md  and  docs/foop/FOOP-<NUMBER>.plan.md (if it exists)
```

### Find FOOPs by status

FOOPs do not have a built-in status filter in the helper script. To find FOOPs at a specific status, grep the frontmatter:

```bash
# Find all FOOPs in "Implementing" status:
grep -l '^status: Implementing' docs/foop/FOOP-*.md

# Find all FOOPs in "Draft" status:
grep -l '^status: Draft' docs/foop/FOOP-*.md

# Find all FOOPs that have begun (begun: [x]):
grep -l '^begun: \[x\]' docs/foop/FOOP-*.md
```

### Find backburnered plans

Backburnered plans are excluded from normal "ready/pending/active" queries. They can **only** be found by explicitly searching for the backburner marker:

```bash
grep -l 'backburnered' docs/foop/FOOP-*.plan.md
```

---

## FOOP Status Lifecycle

A FOOP progresses through statuses:

```
Draft → Brewing → Final → Implementing → complete
```

| Status | Meaning |
|--------|---------|
| `Draft` | Initial state. Being written, not yet ready for review. |
| `Brewing` | Ready for BDFL review. The spec is complete enough for discussion. |
| `Final` | Accepted. The design is frozen. Ready for implementation planning. |
| `Implementing` | Active coding. The plan is being executed. Open Questions section should be empty (design frozen). |
| `complete` | All work done, merged, worktree cleaned up. |

To change status, edit the `status:` field in the FOOP's frontmatter:

```yaml
status: Implementing
```

The `begun:` field tracks whether work has started:

```yaml
begun: [ ]   # not yet started
begun: [x]   # work has begun
```

---

## Task: Execute a FOOP Plan

### Execution Flow (step by step)

1. **Read both files.** Read `FOOP-<NUMBER>.md` (the spec) first, then `FOOP-<NUMBER>.plan.md` (the plan). Do not act on the plan without the spec's context.

2. **Begin work** — in the origin directory:
   - Check the `begun: [x]` box in the FOOP's frontmatter.
   - Commit the FOOP file stating that work has commenced on this FOOP.

3. **Create the worktree** (if the plan calls for one):

   ```bash
   # Variables (should already be expanded to literals in the plan):
   # WORKTREE_ORIGIN_BRANCH=jia
   # WORKTREE_ORIGIN_PATH=/home/<USER>/foolish-rust
   # WORKTREE_BRANCH_NAME=foop-<NUMBER>-<SHORT_DESCRIPTION>
   # WORKTREE_FULL_FS_PATH=$(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>

   cd /home/<USER>/foolish-rust
   git worktree add -b "foop-<NUMBER>-<SHORT_DESCRIPTION>" \
       "$(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>"
   cd "$(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>"
   # Now commence work here.
   ```

   > **Branch naming (no prefix):** the branch is `foop-<NUMBER>-<SHORT_DESCRIPTION>` — bare, no
   > `foop/` prefix — and identical to `WORKTREE_BRANCH_NAME` and to the worktree directory's
   > basename. One name, used everywhere. If a plan you are executing mixes a `foop/`-prefixed
   > form with a bare form (older plans do), **stop and reconcile it before creating the
   > worktree**: otherwise the create step makes one branch and the merge step names another that
   > does not exist.

4. **All subsequent work happens in the worktree** — including updates to the FOOP spec or the plan itself. Changes to `docs/foop/` go **ONLY** to the worktree until merge time. This is non-negotiable.

5. **Commit regularly** as progress is made. Good progress should be committed frequently.

6. **Execute checkboxes top-to-bottom.** Each task is executed one after another. Parent tasks are not checked off until all children are complete.

7. **Upon completion** (or at request of user), merge the branch according to the stated plan.

### When asking the human questions

Always remind them of context:

> Above message comes from FOOP-<NUMBER> working to <brief description>; the worktree is at $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>. PTAL

### Running the sub-section test subset

Each sub-section (or undivided phase) starts with an **"Establish relevant tests"** checkbox
naming the sub-section's test subset and linking to `README.md` §"Running specific tests".
When you reach it:

1. Follow the linked README section to build the run commands for the NAMED cases (unit-test
   name filters; einmo `evaluate --filter` + `compare` for the einmo cases).
2. Run the subset FREQUENTLY while implementing the sub-section — after each feature increment
   and each time a new test lands — and analyze the results before moving on. Add each new
   test to the subset's list as it is written.
3. When the sub-section is complete, run ALL tests (`cargo test --workspace` and
   `cargo test -p foolish-ubca --lib -- einmo_gate_checked`) — even if the phase test-gate
   checkbox comes later.

**Use subagents for test runs whenever the environment provides them** — launch the unit
subset and the einmo subset (and independent filter batches) as parallel subagent tasks, keep
implementing, and collect the results. This is the agent equivalent of a human opening several
terminals; do not serialize long test runs behind typing.

Older plans (pre-2026-08-12) lack the "Establish relevant tests" checkbox. When executing
one, derive the subset yourself from the sub-section's feature and its Test Plan, and apply
the same discipline.

---

## Task: Checkbox Lifecycle

### Completing a task (with timestamp)

When an item is checked off, **always place a timestamp (to the minute) on the next line with indent**:

```markdown
- [x] Task completed
      (2026-07-11 14:32)
```

**Wrong** (no timestamp):
```markdown
- [x] Task completed
```

**Parent tasks are not checked until all children are complete.** This is a hard rule — the parent checkbox is the last to be checked in a block of sub-tasks.

### Backburnering (Delaying)

When a specification is considered VERY important but interfering with current highest priorities, it is marked with `[x] backburnered`. To be revived by removing the `[x] backburnered` marker.

```markdown
- [x] backburnered
      (2026-07-11 14:00)
- [ ] Do this or system will break
- [ ] And fix that bug
- [ ] ...
```

**Exclusion rule:** These plans are to be **excluded** when an agent or human asks for plans that are: ready, pending, iterating, in progress, developing, active, etc. Backburnered plans can **only** be found and addressed directly by using the words "backburnered plan(s)".

**Reviving:** Remove the `[x] backburnered` marker (and its timestamp line) from the plan. The remaining tasks become active again.

### Cancelling (Deprecation)

Canceled features are marked as "not to be done." The procedure:

1. **First** add the canceled checkbox at the top of the plan.
2. **Then** mark **all** todo items with per-item cancellation `[-]`.
3. The deprecation can have elaboration regarding the reasons and context on the same line after the initial `[x] Canceled.` text.

```markdown
- [x] Canceled. Optionally explain — see FOOP-<NEW_NUMBER>
      (2026-07-11 14:00)
- [-] Do this or system will break
- [-] And fix that bug
- [-] ...
```

**Entirely deprecated plan:** Has a `[x] Canceled` box at the top, and every todo is marked `[-]`.

**Per-item cancellation:** Use `[-]` (not `[ ]` or `[x]`) for each cancelled task. This distinguishes "not done because cancelled" from "not done yet" (`[ ]`) and "done" (`[x]`).

---

## Task: Worktree Execution

### Permission scope

An agent with permission to work on the main foolish directory also has permission to work on a worktree added from the foretias directory. If asking for permission, ask once for the **entire worktree path** (`$(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>`), not a subdirectory.

### All work goes in the worktree

Once work begins, all updates — including to the foop folder (FOOP spec edits, plan edits) — MUST be written ONLY to the worktree. This continues until merge time. The origin directory's FOOP files are not touched again until the merge brings the worktree's changes back.

### Commit regularly

Good progress should be committed regularly. Do not batch all work into a single commit at the end — commit as logical units complete.

---

## Task: Merge to Alpha and Cleanup

### Pre-merge verification

Before merging, verify (from the worktree):

```bash
cd $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>
git status   # must be clean — all work committed
```

### Merge

```bash
# Switch to origin and merge:
cd /home/<USER>/foolish-rust
git checkout jia
git merge foop-<NUMBER>-<SHORT_DESCRIPTION>
```

**Foolish uses `git merge`, not rebase.** Expect merge conflicts on `jia` — they trigger follow-up repair work (see "Sub-task execution" below).

### Post-merge: repair tests if needed

If the merge brought conflicts or broke tests:

```bash
# Repair ALL tests in jia:
cd /home/<USER>/foolish-rust
cargo test --workspace
# Fix any failures, then complete the merge commit.
```

### Cleanup the worktree

```bash
# After merge is verified and all tests pass:
git worktree remove $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>
```

### Merge sub-task checklist (from the plan)

This is the canonical merge sub-task pattern. The parent checkbox is the last to be checked:

```markdown
- [ ] Merge `foop-<NUMBER>-<SHORT_DESCRIPTION>` to `jia`
  - [ ] Check and make sure current foop has, and passes, a "comprehensive" snaptest that thoroughly tests interaction of current feature with older features. Input name: `input/foop/<NUMBER>/comprehensive.foo` (reserved for this foop). Agent generates and verifies; human gives final signed approval.
  - [x] Detected complex merge situation requiring additional work
        (2026-07-11 14:00)
  - [ ] Update `foop-<NUMBER>-<SHORT_DESCRIPTION>` to follow new coding style
  - [ ] Update `foop-<NUMBER>-<SHORT_DESCRIPTION>` to use new API call convention
  - [x] Merged breaking changes from `jia`
        (2026-07-11 14:31)
  - [ ] Repair ALL tests in `jia` in /home/<USER>/foolish-rust
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present human with the `cd $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>` command and ask them to review snapshots BEFORE checking the parent checkbox.
  - [ ] Cleanup $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>
    - [ ] Check that .plan.md has all but Cleanup checkboxes completed
    - [ ] Remove $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>
    - [ ] This is the last sub-task checkbox to be checked in this block
```

**Key rules from this pattern:**
- The merge checkbox is the **last** to be checked after all work is done.
- The comprehensive snaptest must exist and pass before merge.
- Every `output→checked` promotion runs through the **Promotion Review Gate** (below) — never a bare promote.
- The `STOP!` checkpoint requires human review of snapshots — the agent must **never** continue past this automatically.
- Cleanup is the last sub-task block: verify all-but-cleanup checkboxes are done, then remove the worktree directory.

---

## Task: Promote `output` → `checked` (Promotion Review Gate)

**Promotion is a correctness claim you are personally making, not a bookkeeping step.** Writing `checked/` freezes an expected-output contract that every future change is measured against. Checking a promotion box asserts: *"I read this case's OUTPUT statement by statement, and I can say in my own words why each line is what the specification requires."*

**Being your own FOOP's test makes promotion permissible, not justified.** The justification is the reading. Never run `einmo promote` as a step you arrive at — run it as a step you have *earned*.

### Procedure

1. **Confirm the suite is otherwise green.**
   ```bash
   cargo test -p foolish-ubca --lib -- einmo_gate_checked
   einmo compare output checked foolish-ubca/einmo_suite
   ```
   Any *foreign*-FOOP baseline divergence is a regression you introduced — **fix your code**, do not promote. Any case with a `verified/` twin is frozen — **STOP and ask the human**.

2. **Enumerate the cases you intend to promote, by name.** If the plan's gate block does not already name them one per sub-task, fix the plan first (see `foop-write-plan` §"Promotion Review Gate"). "Promote the FOOP-*N* outputs" is not a reviewable unit of work; `foop/23/value_search_unanchored` is.

3. **Re-read the in-force specification** for each feature under test — the FOOP's own `.md` §Specification (not the plan, and not a superseded revision), plus `README.md` §"The Unknown" for any NK result.

4. **Review each case, statement by statement.** For every OUTPUT line, read the INPUT statement that produced it and state *why that value is what the spec mandates*:
   - **"The evaluator emitted this" is never a justification** — it is the thing being checked.
   - **Be skeptical of `NK`.** NK is the narrow, exceptional outcome. Name which legitimate case applies (anchored miss ⇒ NK; unanchored miss ⇒ ECONSTANIC). If you cannot name it, trace it with the `foolish-debugging` skill before promoting.
   - **Statement names are specification.** `hit = ?…` claims the search finds its target; `miss = ?…` claims it does not. A `hit` yielding NK is the test contradicting itself — resolve it (fix the predicted bug, or rename with an explanatory comment) rather than promoting past it.
   - **Check coherence, not only conformance.** Does an analogous existing feature behave the same way? Would a Foolisher reading only the spec predict this output? A locally defensible result that is inconsistent with a sibling feature is a design bug to raise, not a baseline to freeze.
   - **Step counts and alarms are part of the contract too.** An unexplained jump in steps for a feature whose cost should not have changed is a signal to investigate.

5. **Write the justification down** — in the plan under the gate, or in the commit message. One or two sentences per case: what it demonstrates, why its result is spec-correct. If you cannot write it, you have not reviewed it.

6. **Promote, then re-verify.**
   ```bash
   einmo promote output to checked foolish-ubca/einmo_suite
   cargo test -p foolish-ubca --lib -- einmo_gate_checked   # must exit 0
   ```

### Reasonable effort, and what to do with a doubt

Justifying a line is not proving it from first principles. Where a result plainly follows from the spec, note it and move on; concentrate effort where a result is surprising, where the spec is ambiguous, or where a value contradicts its statement's own name.

**When you doubt something, write it down and keep going.** Do not halt mid-review to ask about a single case, and do not send concerns one at a time. Record the case, the line, what you expected, what you saw, and which spec or sibling behavior makes you doubt it — then continue. **At the end of the pass, present all accumulated concerns in one statement** (with the PTAL reminder — FOOP number and worktree path).

Uncertainty is never grounds to promote unread; it is grounds to finish reviewing and report. Non-blocking doubts: promote and report alongside. Any blocking doubt: promote nothing, report the full set, wait.

**If any case fails review, promote none of them.** Fix the code — or revise the test's input/statement names, which is a reviewable change in its own right — and re-run the gate.

---

## Task: Comprehensive Test Verification

Every FOOP should have a comprehensive snapshot test (generated during implementation, see `foop-write-plan` skill). During execution and before merge, verify it:

```bash
# Run the approval test suite:
cargo test -p foolish-ubca --lib -- approval_all

# Review the .snap.new output (present to human — NEVER auto-accept):
# Human runs:  ./foolish_review.sh foolish-ubca
# Human runs:  ./accept_approved.sh foolish-ubca
```

> **NEVER** run `cargo insta accept` or `INSTA_UPDATE=always`. See `AGENTS.md` snapshot safety rules. The agent generates and verifies; **human gives final signed approval**.

The comprehensive test input file is at:
```
foolish-ubca/einmo_suite/input/foop/<NUMBER>/comprehensive.foo
```

This name is reserved for this FOOP alone.

---

## Task: Resume a FOOP (after interruption)

If a FOOP was in progress and work was interrupted:

1. **Find the FOOP**: `python3 docs/foop/scripts/foop_check.py list` or look for `begun: [x]` in frontmatter.
2. **Read both files**: `FOOP-<NUMBER>.md` and `FOOP-<NUMBER>.plan.md`.
3. **Check the plan for completed checkboxes** — they have timestamps, so you can see where work stopped.
4. **Check if the worktree still exists**:
   ```bash
   git worktree list
   ```
   If it exists: `cd` into it and continue.
   If it was removed but the branch exists: recreate the worktree:
   ```bash
   git worktree add $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION> foop-<NUMBER>-<SHORT_DESCRIPTION>
   ```
5. **Continue from the next unchecked checkbox.**

### Finding backburnered plans to revive

Backburnered plans are excluded from normal queries. To find them:

```bash
grep -l 'backburnered' docs/foop/FOOP-*.plan.md
```

To revive: remove the `[x] backburnered` marker (and its timestamp line) from the plan.

---

## Quick Reference — All Execution Commands

```bash
# ── Finding ──
python3 docs/foop/scripts/foop_check.py list       # all FOOPs, chronological
python3 docs/foop/scripts/foop_check.py get_last   # most recent FOOP
python3 docs/foop/scripts/foop_check.py check      # verify consecutive numbering
ls docs/foop | rev | sort -V | rev                 # chronological ls
grep -l '^status: Implementing' docs/foop/FOOP-*.md  # find by status
grep -l 'backburnered' docs/foop/FOOP-*.plan.md      # find backburnered

# ── Worktree ──
git worktree list                                  # check existing worktrees
git worktree add -b foop-<NUMBER>-<SHORT_DESCRIPTION> \
    $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>
cd $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>

# ── Comprehensive test ──
cargo test -p foolish-ubca --lib -- approval_all

# ── Merge & cleanup ──
cd ${HOME}/foolish-rust && git checkout jia
git merge foop-<NUMBER>-<SHORT_DESCRIPTION>
cargo test --workspace                              # verify after merge
git worktree remove $(pwd)/../foolish_worktrees/foop-<NUMBER>-<SHORT_DESCRIPTION>
```

---

## Safety Invariants

1. **Read `foop.md` before executing or maintaining any FOOP.** This skill is a cookbook; `foop.md` is the authority.
2. **Read BOTH files (spec + plan) before acting.** The plan assumes the spec's context.
3. **Once work begins, all updates go to the worktree ONLY** — including FOOP spec/plan edits — until merge time.
4. **Execute checkboxes top-to-bottom.** Parent tasks are not checked until all children are complete.
5. **Every checkbox completion gets a timestamp** on the next indented line (to the minute).
6. **Backburnered plans are excluded** from "ready/pending/active" queries. Only found by explicitly saying "backburnered."
7. **Cancelled plans** have `[x] Canceled` at top + `[-]` on every todo item.
8. **Never auto-accept snapshots.** Do not run `cargo insta accept` / `INSTA_UPDATE=always`. Human review required.
9. **Never start Phase+ work when tests are broken.** Fix or disable (with human permission) first.
10. **Foolish uses `git merge`, not rebase.** Expect conflict-repair sub-tasks.
11. **Never continue past a `STOP!` checkpoint automatically.** Human must check that box.
12. **Never commit from inside this skill** unless the user explicitly asks.
13. **Never promote `output→checked` without reading every case, statement by statement.** Promotion is a correctness claim, not bookkeeping; "the evaluator emitted this" is the thing being checked, not a justification for it. Run the **Promotion Review Gate** (see the task section above) — green suite, no `verified/` twin, spec re-read, per-case justification written down — before any `einmo promote`. A promotion box checked faster than the case could be read is a false record of work.
14. **Never promote over a foreign FOOP's divergent baseline.** A failing einmo test is broken code, not a stale baseline — fix your code. If the divergent baseline has a `verified/` twin, it is frozen without a human reviewer's key. See `rust_instructions.md` §"Phase-by-phase testing discipline."

---

## Last Updated

**Date**: 2026-08-12
**Updated By**: Sisyphus / oqwen/qwen/qwen3.8-max
**Changes**: Added **§"Running the sub-section test subset"** under Task: Execute a FOOP Plan:
each sub-section's "Establish relevant tests" checkbox (installed by the `foop-write-plan`
skill per its rule 11) names the sub-section's small test subset and links to `README.md`
§"Running specific tests"; the implementer builds the commands from that central reference,
runs the subset frequently during the sub-section, runs ALL tests when the sub-section
completes, and runs tests through parallel subagents where available (the agent equivalent of
several terminals). Includes the fallback for pre-2026-08-12 plans that lack the checkbox.
Mirrors the new §"Sub-Section Test Subsets" in `foop.md`.
