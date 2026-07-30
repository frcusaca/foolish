# FOOP — Foolish Optimization Process (Full Reference)

> **Read every line of this file before you read or write any FOOP.**
> `AGENTS.md` carries only a short summary of the common, every-day FOOP
> operations and points here for everything else. This document is the
> authoritative description of the FOOP process, its philosophy, the
> numbering system, the file layout, plan construction, and the checkbox
> lifecycle. When `AGENTS.md` and this file appear to disagree about FOOP
> mechanics, this file is the more detailed source — reconcile in favor of
> the explicit rules written here.

---

## What a FOOP Is

FOOP documents are the Foolish equivalent of Python's PEP or Rust's RFC.
They propose, discuss, and track changes to the Foolish language and its
reference implementations.

- **Location**: `docs/foop/FOOP-###.md`
- **Index**: `docs/foop/INDEX.md` (canonical list, sorted by number)
- **Template**: `docs/foop/FOOP-template.md`
- **Meta-FOOP**: [FOOP-1](docs/foop/FOOP-1.md) defines the process itself

A FOOP progresses through statuses: `Draft` → `Brewing` (ready for BDFL
review) → `Final` (accepted) → `Implementing` (active coding) → complete.
Each FOOP is assigned to a `phase` (phase-1 through phase-7, or `meta` for
process documents).

---

## The Two Files of a FOOP

Every FOOP is expressed as (up to) two separate files that share the same
`foop-<NUMBER>` stem:

- **`FOOP-#.md`** — the **specification** and related information: the
  proposal, motivation, design, semantics, and discussion. This is the
  *what* and the *why*.
- **`FOOP-#.plan.md`** — the **plan**: a checkboxed, sequentially-executed
  breakdown of the work needed to implement the specification. This is the
  *how* and the *in-what-order*. (Note the lowercase `.plan.md` extension.)

**Executing a FOOP requires reading BOTH files.** The plan assumes the
context of the specification; do not act on `FOOP-#.plan.md` without first
reading `FOOP-#.md`. The plan is meant to be executed sequentially from top
to bottom.

---

## FOOP Numbering is Little Endian

FOOP-1 is before FOOP-2, FOOP-9 is the one before FOOP-01, and so on and so
forth. To list the directory in order of oldest to newest, use this command:

```bash
ls docs/foop|rev|sort -V|rev
```

*always* use this command to list the FOOPs to establish ordering.

---

## FOOP Naming Convention (Critical)

The identifier `FOOP-01` uniquely identifies an optimization step. In free
text, use "FOOP 01" (no dash, space instead). This convention reduces the
risk of digit reversal: writing "FOOP 01" in prose makes it harder to
accidentally type "FOOP 10". In sentences, use the space form: "FOOP's 01,
11, 21 are the only pre-teen foops we will implement." Reserve the dash form
`FOOP-01` for filenames, code references, and formal citations only.

The **filename digits ARE the identifier**. The `foop:` frontmatter field is
a separate numeric sort key, equal to the digits reversed. Do NOT use the
sort-key value as the identifier in prose. Examples:

| Filename     | Identifier (use this) | Sort key (frontmatter only) |
|--------------|-----------------------|-----------------------------|
| `FOOP-9.md`  | FOOP-9                | 9                           |
| `FOOP-01.md` | FOOP-01               | 10                          |
| `FOOP-21.md` | FOOP-21               | 12                          |
| `FOOP-51.md` | FOOP-51               | 15                          |

---

## FOOP Numbering Helper Script

Use `docs/foop/scripts/foop_check.py` to manage FOOP numbering. Run it
before creating a new FOOP and periodically to catch drift:

```bash
python3 docs/foop/scripts/foop_check.py check     # verify consecutive numbering
python3 docs/foop/scripts/foop_check.py get_last  # most recent FOOP
python3 docs/foop/scripts/foop_check.py gen_next  # filename for next FOOP
python3 docs/foop/scripts/foop_check.py list      # all FOOPs in chronological order
```

When creating a new FOOP, **always** run `gen_next` first to get the correct
filename and identifier. The script handles the little-endian encoding for
you.

---

## Plan Files for FOOP Implementation

When implementing a FOOP, write a detailed plan to
`docs/foop/FOOP-###.plan.md` (lowercase extension). The plan breaks the FOOP
into concrete, trackable tasks using checkboxes. The plan file should have a
level of detail so as for coding to be immediately commenceable. If research
was done on the web, through historic foolish doc's, or experimentation performed
to establish a correct usage pattern, those should be clearly documented in
the foop file, the plan steps shall, where needed, contain section or sub-section
header pointer into the foop file, a large todo with sub-tasks may have several
read such-and-such section of the foop as first few checkboxes.

The plan sub-tasks can also be sanity check markers for implementing agent. For
example, if it is clear that the foop and plan left some ambiguity (perhaps at
demand of human saying "we can figure that out when we get there.") In particular
if a major coding decision needs to be made, or if research and experimentation
is expected. The sanity check instruction subtask could say "[ ] sub-agent please
consult with primary agent or human regarding the current approach to..." During
review of foop/plan, the plannign agent may install these or remove these as
it progresses with specification, clarification, design and planning for the
project.

### Constructing the Plan

The plan is derived from the already-written specification (`FOOP-#.md`).
Because the specification exists before the plan, you can name a concrete
`short_description` for the work and decompose the specification into an
ordered list of checkbox tasks. Build the plan so that:

- Tasks are listed in the order they must be executed (top to bottom).
- Each task is concrete and trackable on its own.
- Worktree lifecycle tasks (create / verify / merge / cleanup) appear as
  explicit checkboxes at the appropriate points (see "Worktree Branch
  Tracking" below).
- Tasks that prove larger than expected split into indented sub-tasks (see
  "Sub-Tasks" below).
- All RHS variables should be expanded and literally placed into the plan
  file as the plan is being created.
- Once work begins on a foop, all updates, including to the foop folder
  *MUST* be written *ONLY* to the worktree. This continues until merge time.

### Checkbox Format

Checkboxes in a plan file track progress. When an item is checked off,
**always place a timestamp (to the minute) on the next line with indent into
the bulleted list**:

```markdown
- [ ] Task not yet done
- [x] Task completed                    ← bad (no timestamp)
- [x] Task completed                    ← good it is
      (2026-05-06 13:11)                ← timestamped properly
```

This gives both agents and humans a clear view of how work is progressing
over time.

### Backburnering (Delay)

When a specification is considered VERY important but interfering with
current highest priorities, it is marked with `[x] backburnered`. To be
revived by removing the `[x] backburnered` marker. These plans are to be
excluded when agent or human asks for plans that are: ready, pending,
iterating, in progress, developing, active, etc. backburnered plans can only
be found and addressed directly by using the words "backburnered plan(s)".

```markdown
- [x] backburnered
      (2026-05-06 14:00)
- [ ] Do this or system will break
- [ ] And fix that bug
- [ ] ...
```

### Cancelling (Deprecation)

Canceled features shall be marked as "not to be done" using the marker `[-]
don't do this`. An entirely deprecated plan shall have a `[x] canceled` box
at the top. The agent should first add the canceled check item, then mark
all todo's with per-item cancelation `[-] each one`. The deprecation can
have elaboration regarding the reasons and context on the same line after
the initial `[x] Canceled.` text. Here is the example of a properly canceled
spec:

```markdown
- [x] Canceled. Optionally explain there's a new spec see FOOP-####
      (2026-05-06 14:00)
- [-] Do this or system will break
- [-] And fix that bug
- [-] ...
```

### The main branch is `jia`

**In the Foolish project, the main branch is named `jia`.** It fills the role
other projects give to `master`, `main`, or `trunk`: it is the trunk of
development, the branch worktrees are created from, and the branch completed
FOOP work is merged back into.

There is no `master`, `main`, or `trunk` branch. Older documents and plans may
refer to an `alpha` branch as the merge target; that name is historical —
**read `alpha` as `jia`** wherever it appears in an in-force instruction.
Completed plan files are left as written, as a historical record.

### Worktree Branch Tracking

If a worktree branch is used for implementation, the plan **must** document
the lifecycle of that worktree as explicit, separate checkbox tasks placed
at appropriate points in the plan. The workpath shall always be:

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=$(pwd)
WORKTREE_BRANCH_NAME=foop-<NUMBER>-short_description
WORKTREE_FULL_FS_PATH=$(pwd)/../foolish_worktrees/foop-<NUMBER>-short_description

## The branch is created this way from the ${WORKTREE_ORIGIN_BRANCH} branch and path
# cd $WORKTREE_ORIGIN_PATH ## User normally starts in this directory
# git checkout $WORKTREE_ORIGIN_BRANCH ## Again, user normally already has this branch checked out.
git worktree add -b "$WORKTREE_BRANCH_NAME" "$WORKTREE_FULL_FS_PATH"
cd "$WORKTREE_FULL_FS_PATH"
# Now commence work here.
```

The worktree path is **relative to the project root** (`../foolish_worktrees/`).
This keeps worktrees close to the project, avoids polluting `$HOME/tmp/`, and
is path-independent of the user's home directory. For a project at `/yolo/src`,
this resolves to `/yolo/foolish_worktrees/<branch-name>`.

The short_description in the path should be generated as part of the
.plan.md generation. It is possible because the specification is already
made and a short description should be possible. the "foop-<NUBER>" suffix
should match the name of the foop file as well as the plan file. Once set,
this path name

Agent with permission to work on the main foolish directory also has
permission to work on a worktree added from the foretias directory. If asking
for permission, ask once for the entire worktree branch:
"${WORKTREE_FULL_FS_PATH}" not a subdirectory.

**Branch naming — one name, no prefix.** The branch is named
`foop-<NUMBER>-<short_description>`, **bare, with no `foop/` prefix**, and it
must be **identical** to `WORKTREE_BRANCH_NAME` and to the basename of
`WORKTREE_FULL_FS_PATH`. One name, used everywhere in the plan.

Current practice (FOOP-13, FOOP-23, FOOP-33, FOOP-54, FOOP-84):
`foop-23-value-search`, `foop-54-einmo`. A `foop/`-prefixed form and
number-prefixed directory names (`3841-foop-7`) appear in older pre-2026-06
plans; **do not copy them.** Mixing forms within one plan is a real hazard — the
create checkbox makes one branch while the merge checkbox names another that
does not exist.

```markdown
- [ ] Create worktree at $(pwd)/../foolish_worktrees/foop-7-constanic-clone with branch `foop-7-constanic-clone`
...
  (implementation tasks here)
...
- [ ] Verify all work is complete in $(pwd)/../foolish_worktrees/foop-7-constanic-clone and committed to `foop-7-constanic-clone`
- [ ] Merge `foop-7-constanic-clone` to `jia` #Btw, These branch names and paths reflect expanded $WORKTREE_BRANCH_NAME and $WORKTREE_ORIGIN_BRANCH, which should be known and specified by the time of _PLAN.md's completion. Fillers such as the literal '$WORKTREE_ORIGIN_BRANCH' should be replaced with real values before starting work on the plan. The worktree directory is relative to the project root.
```

### Sub-Tasks

If a task proves larger than expected and splits into multiple sub-tasks,
indent them under the parent. Use completed sub-tasks to justify why the
split occurred:

```markdown
- [ ] Merge ${WORKTREE_BRANCH_NAME} to ${WORKTREE_ORIGIN_BRANCH} # <-- this checkbox is the last to be checked after all the work is done.
  - [ ] Check and make sure current foop has, and passes, a "comprehensive" snaptest that thoroughly tests interaction of current feature with older features. it would have the unique input name 'foop_<NUMBER>_comprehensive.foo', which is a name reserved for this foop. This test may be slightly repetitative of previous tests preferring coverage of high value features and checking odd edge cases. Note, generating and running the test and verifying is agent's job, but final approval for new tests requires human operator review and formal signed approval.
  - [x] Detected complex merge situation requiring additional work
        (2026-05-06 14:00)
  - [ ] Update ${WORKTREE_BRANCH_NAME} to follow new coding style
  - [ ] Update ${WORKTREE_BRANCH_NAME} to use new API call convention
  - [x] Merged breaking changes from ${WORKTREE_ORIGIN_BRANCH}
        (2026-05-06 14:31)
  - [ ] Repair ALL tests in ${WORKTREE_ORIGIN_BRANCH} in ${WORKTREE_ORIGIN_PATH}
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present human with the the "cd ${WORKTREE_FULL_FS_PATH}" command and ask them to review snapshots BEFORE checking the parent checkbox.
  - [ ] Cleanup ${WORKTREE_FULL_FS_PATH}
    - [ ] Check that _PLAN.md has all but Cleanup checkboxes completed
    - [ ] Remove "${WORKTREE_FULL_FS_PATH}"
    - [ ] This is the last sub-task checkbox to be checked in this block of subtasks
```

This pattern is common because Foolish uses `git merge` (not rebase), so
merge conflicts on `jia` may trigger follow-up repair work. NB: The environmental
variable exprssions within "${}" should be expanded to actual names and full
paths by the time each plan file is finalized.

### Plan execution

Expect to executed each task one after another. parent tasks should not be checked off
until children are complete. One project starts, the 'begun [ ]' checkbox is checked in
the origin directory. The foop file is committed stating that work has commenced on such
and such foop. The worktree/branches are created. From that point onward, all work is to
be completed in the worktree directory. This may include updates to the FOOP file or the
plan itself, these changes are to occur ONLY in the worktree. Good progress should be
commited regularly. Upon completion or at request of user, the branc is merged according
to the stated plan.

When asking human questions, always remind them: "Above message comes from FOOP-<NUMBER>
working to...briefe description...; the worktree is at ${WORKTREE_FULL_FS_PATH}. PTAL"

---

## Comprehensive FOOP Tests

Every FOOP has the right — and the obligation — to generate a **comprehensive snapshot test**
that thoroughly exercises the new feature interacting with existing features. This test:

- **Input file**: `foolish-ubca/snapshot_tests/input/foop_<NUMBER>_comprehensive.foo`
  (e.g. `foop_23_comprehensive.foo`). The name is reserved for this FOOP alone.
- **Purpose**: coverage of high-value feature combinations and edge cases that the
  per-phase approval tests may not reach. Slight repetition of earlier tests is acceptable
  if it serves coverage.
- **Scope**: mix new features with old — value search inside nested branes, contexted
  operators chained with dot access, expression patterns referencing ancestral names,
  combined name+value with head/tail, etc. The test should be large enough to exercise
  at least one path through every new operator or predicate variant.
- **Process**: the agent generates the `.foo` input, runs it through the approval test
  suite, and verifies the `.snap.new` output. Final approval requires human review and
  formal signed acceptance.
- **Placement in plan**: a checkbox task "Write and verify `foop_<NUMBER>_comprehensive.foo`"
  should appear in the plan, after all implementation phases and before the merge STOP.

---

## Last Updated

**Date**: 2026-07-29 (2)
**Updated By**: Claude Code (Opus 5)
**Changes**: Stated the **branch-naming rule** explicitly: the branch is
`foop-<NUMBER>-<short_description>`, bare with **no `foop/` prefix**, identical to
`WORKTREE_BRANCH_NAME` and to the worktree directory's basename — one name, used everywhere in the
plan. The worked example had been triply inconsistent (directory `constanic-clone-foop-7`, then
`3841-foop-7`, with branch `foop/foop-7-constanic-clone`), which is what let a real defect through
in FOOP-84's plan: its create checkbox made one branch while its merge checkbox named another that
did not exist. Example rewritten to use one consistent name. Both FOOP skills updated to match.

**Date**: 2026-07-29
**Updated By**: Claude Code (Opus 5)
**Changes**: Added a "The main branch is `jia`" section stating that `jia` fills the role other
projects give to `master`/`main`/`trunk`, and that `alpha` — appearing in older documents as the
merge target — is historical and should be read as `jia` in any in-force instruction. Completed
plan files are left as written, as a historical record. Updated `WORKTREE_ORIGIN_BRANCH` to `jia`
and the merge/conflict-repair examples to match. AGENTS.md and both FOOP skills updated in the
same pass.

**Date**: 2026-07-06
**Updated By**: Hephaestus / xiaomi/mimo-v2.5-pro
**Changes**: Added "Comprehensive FOOP Tests" section — every FOOP has the right and obligation
to generate a `foop_<NUMBER>_comprehensive.foo` snapshot test exercising new features
interacting with existing ones.

**Date**: 2026-06-10
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Created foop.md as the authoritative full FOOP reference.
Migrated the detailed FOOP content out of AGENTS.md: process philosophy,
little-endian numbering, naming convention, numbering helper script, the
two-file (`FOOP-#.md` spec / `FOOP-#.plan.md` plan) layout, plan
construction, checkbox format, backburnering, cancellation, worktree branch
tracking, and sub-tasks. AGENTS.md now retains only a short summary of
common every-day uses plus an instruction to read this file before reading
or writing any FOOP.
