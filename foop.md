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
into concrete, trackable tasks using checkboxes.

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

### Worktree Branch Tracking

If a worktree branch is used for implementation, the plan **must** document
the lifecycle of that worktree as explicit, separate checkbox tasks placed
at appropriate points in the plan. The workpath shall always be:

```
WORKTREE_ORIGIN_BRANCH=alpha
WORKTREE_ORIGIN_PATH=$(pwd)
WORKTREE_BRANCH_NAME=foop-<NUMBER>-short_description
WORKTREE_FULL_FS_PATH=${HOME}/tmp/foolish-worktrees/foop-<NUMBER>-short_description

## The branch is created this way from the ${WORKTREE_ORIGIN_BRANCH} branch and path
# cd $WORKTREE_ORIGIN_PATH ## User normally starts in this directory
# git checkout $WORKTREE_ORIGIN_BRANCH ## Again, user normally already has this branch checked out.
git worktree add -b "$WORKTREE_BRANCH_NAME" "$WORKTREE_FULL_FS_PATH"
cd "$WORKTREE_FULL_FS_PATH"
# Now commence work here.
```

The short_description in the path should be generated as part of the
.plan.md generation. It is possible because the specification is already
made and a short description should be possible. the "foop-<NUBER>" suffix
should match the name of the foop file as well as the plan file. Once set,
this path name

Agent with permission to work on the main foolish directory also has
permission to work on a worktree added from the foretias directory. If asking
for permission, ask once for the entire worktree branch:
"${WORKTREE_FULL_FS_PATH}" not a subdirectory.

```markdown
- [ ] Create worktree at ${HOME}/tmp/foolish-worktrees/constanic-clone-foop-7 with branch `foop/foop-7-constanic-clone`
...
  (implementation tasks here)
...
- [ ] Verify all work is complete in ${HOME}/tmp/foolish-worktrees/3841-foop-7 and committed to `foop/foop-7-constanic-clone`
- [ ] Merge `foop/foop-7-constanic-clone` to `alpha` #Btw, These branch names and paths reflect expanded $HOME, $WORKTREE_BRANCH_NAME and $WORKTREE_ORIGIN_BRANCH, which should be known and specified by the time of _PLAN.md's completion. Fillers such as the literal '$WORKTREE_ORIGIN_BRANCH' should be replaced with real values before starting work on the plan. "${HOME}" should be the full path to "${HOME}" when the plan is generated.
```

### Sub-Tasks

If a task proves larger than expected and splits into multiple sub-tasks,
indent them under the parent. Use completed sub-tasks to justify why the
split occurred:

```markdown
- [ ] Merge ${WORKTREE_BRANCH_NAME} to ${WORKTREE_ORIGIN_BRANCH} # <-- this checkbox is the last to be checked after all the work is done.
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
merge conflicts on `alpha` may trigger follow-up repair work. NB: The environmental
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

---

## Last Updated

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
