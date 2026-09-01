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
- Every sub-section (and every phase that is not subdivided) STARTS with the
  "Establish relevant tests" checkbox naming that sub-section's test subset
  (see "Sub-Section Test Subsets" below).

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
  - [ ] Check and make sure current foop has, and passes, a "comprehensive" snaptest that thoroughly tests interaction of current feature with older features. it would have the unique input name 'input/foop/<NUMBER>/comprehensive.foo', which is a name reserved for this foop. This test may be slightly repetitative of previous tests preferring coverage of high value features and checking odd edge cases. Note, generating and running the test and verifying is agent's job, but final approval for new tests requires human operator review and formal signed approval.
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

### Sub-Section Test Subsets (frequent-run discipline)

Every sub-section of the plan — and every phase that is not subdivided — **starts** with one
checkbox that establishes the SMALL set of tests relevant to that sub-section: the old unit
tests and einmo cases its work must not break, plus the new tests written for it. The checkbox
names the cases and links to the central test-running documentation; the planner fills in REAL
case names (expand every placeholder, same rule as worktree variables):

```markdown
- [ ] Establish relevant tests for this sub-section. Use [these instructions](../../README.md#running-specific-tests) to run einmo tests: <case_1>, foop/<NUMBER>/<case_2>, <case_3>; run unit tests: <crate>::<test_a>, <crate>::<test_b>.
```

The list is alive: as the sub-section writes new tests, each one is added to its checkbox's
list.

**During development** the implementer runs this subset frequently — after each feature
increment and each time a new test lands — and analyzes the results before moving on. **When
the sub-section is complete**, ALL tests run (`cargo test --workspace` and
`cargo test -p foolish-ubca --lib -- einmo_gate_checked`) — do not wait for the phase boundary
if the sub-section ends earlier.

**Run tests through subagents whenever the environment provides them.** Parallel subagent test
runs are the agent equivalent of a human opening several terminals: launch the unit subset and
the einmo subset (and independent filter batches) as separate subagent tasks, keep
implementing, and collect the results. Do not serialize long test runs behind typing when a
subagent could be running them.

The command forms live ONLY in `README.md` §"Running specific tests" — the plan names CASES,
the central document owns the COMMANDS. When the einmo CLI evolves, only that README section
changes; existing plans keep working because they reference cases by name.

---

## Comprehensive FOOP Tests

Every FOOP has the right — and the obligation — to generate a **comprehensive snapshot test**
that thoroughly exercises the new feature interacting with existing features. This test:

- **Input file**: `foolish-ubca/einmo_suite/input/foop/<NUMBER>/comprehensive.foo`
  (e.g. `input/foop/23/comprehensive.foo`). The path is reserved for this FOOP alone.
  Its einmo case name is therefore `foop/<NUMBER>/comprehensive`.
- **Purpose**: coverage of high-value feature combinations and edge cases that the
  per-phase approval tests may not reach. Slight repetition of earlier tests is acceptable
  if it serves coverage.
- **Scope**: mix new features with old — value search inside nested branes, contexted
  operators chained with dot access, expression patterns referencing ancestral names,
  combined name+value with head/tail, etc. The test should be large enough to exercise
  at least one path through every new operator or predicate variant.
- **Process**: the agent generates the `.foo` input, runs it through the approval test
  suite, and reviews the output through the **Promotion Review Gate** (below) before
  promoting. Final approval requires human review and formal signed acceptance.
- **Placement in plan**: a checkbox task "Write and verify
  `input/foop/<NUMBER>/comprehensive.foo`" should appear in the plan, after all implementation
  phases and before the merge STOP, followed by its own Promotion Review Gate.

---

## Promotion Review Gate (`output` → `checked`)

`einmo promote output to checked` writes the **frozen expected-output contract** that every
future change is measured against. It is a *correctness claim made by the agent*, never a
bookkeeping step and never a way to make a red suite green.

**Checking a promotion checkbox asserts:** *"I read this case's OUTPUT statement by statement,
and I can say in my own words why each line is what the specification requires."*

Being your own FOOP's test makes promotion **permissible**, not **justified**. The justification
is the reading. An agent that promotes without inspecting each case has not done the work the
checkbox records — it has falsified the record.

### The gate is a checkbox block, never a one-liner

A plan **must not** contain a bare `- [ ] einmo promote output to checked`. Wherever a phase
produces new or changed einmo output, the plan installs this block, expanded with the **real
case names, one sub-task each**:

```markdown
- [ ] Review and promote `output` → `checked` for FOOP-<NUMBER>'s einmo cases
  - [ ] Confirm the rest of the suite is green — no foreign-FOOP baseline diverges
  - [ ] Confirm no case below has a `verified/` twin (if one does: STOP, ask the human)
  - [ ] Re-read the in-force specification for each feature under test
  - [ ] Review `foop/<NUMBER>/<case_1>` — every OUTPUT statement justified
  - [ ] Review `foop/<NUMBER>/<case_2>` — every OUTPUT statement justified
        (…one checkbox per case; name them all, never "…and the rest")
  - [ ] Write the justification summary into the plan or commit message
  - [ ] Report ALL accumulated doubts to the human in ONE statement — or record "no doubts"
  - [ ] Run `einmo promote output to checked foolish-ubca/einmo_suite`
  - [ ] Re-run `cargo test -p foolish-ubca --lib -- einmo_gate_checked` — must exit 0
```

Per the plan-execution rule above, a parent checkbox is not checked until its children are — so
the per-case boxes cannot be collapsed into a single tick.

### What "every OUTPUT statement justified" requires

- **Statement by statement.** Read each OUTPUT line against the INPUT statement that produced
  it, and state why that value is what the specification mandates. **"The evaluator emitted
  this" is not a justification** — it is the thing being checked.
- **Be skeptical of `NK`.** A search settling NK is the narrow, exceptional outcome, not a
  default. Name which legitimate case applies (anchored miss ⇒ NK; unanchored miss ⇒
  ECONSTANIC — see `README.md` §"The Unknown" and FOOP-23 §Specification). If you cannot name
  it, trace it (see the `foolish-debugging` skill); do not promote it.
- **Statement names are specification, not decoration.** `hit = ?…` asserts the search finds
  its target; `miss = ?…` asserts it does not. A `hit` yielding NK is the test contradicting
  itself — resolve it before promoting, never by promoting past it.
- **Coherence, not just conformance.** Does an analogous existing feature behave the same way?
  Would a Foolisher reading only the spec predict this output? A result that is locally
  defensible but inconsistent with a sibling feature is a design bug to raise, not a baseline
  to freeze.
- **Step counts and alarms are part of the contract.** An unexplained jump in step count for a
  feature whose cost should not have changed is a signal to investigate, not noise to accept.

### Reasonable effort, and what to do with a doubt

Justifying a line does not mean proving it from first principles. Where a result plainly follows
from the spec, note it and move on; concentrate the effort where a result is surprising, where the
spec is ambiguous, or where a value contradicts its statement's own name. Aim for what a careful
colleague would check.

**When you doubt something, write it down and keep going.** Do not halt the review to ask about
one case, and do not send concerns one at a time. Record — case, line, what you expected, what you
saw, and which specification or sibling behavior makes you doubt it — then continue to the next
case. **At the end of the pass, present all accumulated concerns to the human in a single
statement.**

This is the sanctioned exit from an impasse. Uncertainty is never a reason to promote unread; it
is a reason to finish reviewing and report. If the doubts are non-blocking, promote and report
them alongside. If any doubt blocks, promote nothing and report the full set.

**If any case fails review, promote none of them.** Fix the code — or revise the test's input or
statement names, which is a reviewable change in its own right — and re-run the gate.

The full three-stage contract (`output` throwaway / `checked` frozen / `verified` human-attested),
the "a failing einmo test is broken code, not a stale baseline" rule, and the foreign-baseline
prohibition live in `rust_instructions.md` §"Phase-by-phase testing discipline."

### Verified-tier tests are never `#[ignore]`d in the code

An `einmo_gate_verified` test (or any test asserting a `checked/`↔`verified/` correspondence)
must never carry a source-level `#[ignore]` attribute added by an agent. `verified/` is the
human-signed, highest-trust tier of the three-stage contract; an `#[ignore]` baked into the code
silently and permanently removes that gate from every future `cargo test` run, for every agent
and every human, with no visible trace beyond a line in a diff nobody is looking at.

If a Verified gate is noisy during ordinary work — e.g. its `verified/` is genuinely still empty
for a crate mid-migration, and its expected, unresolved failure would otherwise clutter every
`cargo test` run — an agent may skip it for a given invocation via a **command-line filter**
(`cargo test --workspace -- --skip einmo_gate_verified`, or the equivalent per-crate form), never
by adding `#[ignore]` to the source. A command-line skip is scoped to one invocation and leaves
the test itself live, undisguised, and immediately visible to the next `cargo test --workspace`
that does not pass the filter; a code-level `#[ignore]` is a standing, silent, and easily
forgotten exemption. See AGENTS.md's Verified-tier rule for the full statement, including that
this applies retroactively to any existing such `#[ignore]` an agent already added.

---

## Last Updated

**Date**: 2026-09-01
**Updated By**: Claude Code / claude-sonnet-5
**Changes**: Added **§"Verified-tier tests are never `#[ignore]`d in the code"** under the
Promotion Review Gate: an agent may quiet a noisy, expected-to-fail `einmo_gate_verified` for
one `cargo test` invocation via a command-line `--skip` filter, but must never add `#[ignore]`
to the test's source — that is a standing, silent exemption an agent must not grant itself.
Mirrors the rule added to AGENTS.md the same day, prompted by finding exactly this on
`foolish-ubca2`'s `einmo_gate_verified` without a distinct human sign-off (FOOP-16).

This log keeps only the single newest entry — see `git log foop.md` for full history.
