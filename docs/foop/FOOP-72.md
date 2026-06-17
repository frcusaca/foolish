---
foop: 27
title: Foolish Numbering System (FNS) and Snapshot Test Organization
author: Sisyphus <agent>
status: Draft
type: Standards
created: 2026-06-17
phase: phase-0
supersedes: []
---

# FOOP-72: Foolish Numbering System (FNS) and Snapshot Test Organization

> **WORKTREE.** This FOOP is implemented in its own worktree:
>
> ```
> WORKTREE_ORIGIN_BRANCH=alpha
> WORKTREE_ORIGIN_PATH=$(pwd)
> WORKTREE_BRANCH_NAME=foop-72-fns-snapshot
> WORKTREE_FULL_FS_PATH=${HOME}/tmp/foolish-worktrees/foop-72-fns-snapshot
> ```

## Abstract

The little-endian numbering convention currently lives inside the FOOP process
(`foop.md`, `foop_check.py`, scattered references in `AGENTS.md`). This FOOP
promotes it to a **general-purpose Foolish Numbering System (FNS)** applicable
to any ordered artifact — snapshots, FOOPs, approval files, or future
numbered entities — and consolidates snapshot testing documentation into a
single authoritative document.

A new `AGENTS/` directory collects documents referenced exclusively by
`AGENTS.md` — agent-targeted reference material that humans rarely need
to browse directly. `AGENTS.md` points into it; agents read it.

| Document | Purpose |
|----------|---------|
| `AGENTS/fns.md` | Foolish Numbering System — the little-endian convention, rules, script usage, and where it applies |
| `AGENTS/snapshot.md` | Snapshot testing — insta workflow, approval process, signature verification, `.snap` format, AI agent constraints |

Correspondingly, `foop_check.py` is renamed to `fns_check.py` and generalized
to accept any directory and prefix, not just `FOOP-` files.

## Motivation

### Numbering is scattered

The little-endian convention is documented in three places:
- `foop.md` — the full description, embedded as a FOOP-specific concern
- `foolish/AGENTS.md` — a summary under "Working with FOOPs"
- `docs/foop/scripts/foop_check.py` — the docstring repeats the convention

Each copy is slightly different. Adding a new numbered artifact (snapshots)
means either extending the FOOP document with unrelated content or creating
yet another partial copy of the convention.

### Numbering is general-purpose

The little-endian convention is not inherently about FOOPs. It is a
Foolish-project convention for any sequentially-numbered artifact where:
- The identifier is written front-to-back in the filename
- Chronological order is recovered by reversing the digits
- Consecutive numbering is enforced (no gaps)

Snapshot tests are the second consumer. `Fir__check_parent__09283.foo` uses
the same convention — the trailing digits are the little-endian identifier,
reversed to get chronological sort order.

### Snapshot documentation is scattered

Snapshot testing rules appear in:
- `README.md` — command reference, AI agent warning
- `AGENTS.md` — approval workflow, signature verification, snapshot format
- `foolish-core/` and `foolish-ubcb/` — inline comments in test harness code

There is no single place that answers "how do snapshots work end-to-end."

## Terminology

- **FNS** — Foolish Numbering System. The little-endian numbering convention.
- **Sort key** — The chronological position (1, 2, 3, ...) obtained by reversing
  the filename digits and reading as decimal.
- **Identifier digits** — The little-endian digits in the filename (e.g., `09283`
  in `Fir__check_parent__09283.foo`). These ARE the identifier, not the sort key.
- **FNS prefix** — The non-numeric part of a filename before the identifier digits
  (e.g., `FOOP-`, `Fir__check_parent__`).

## The Foolish Numbering System

### Convention

An FNS-numbered filename has the form:

```
<prefix><digits>.<extension>
```

Where:
- `<prefix>` is a domain-specific string (e.g., `FOOP-`, `Fir__check_parent__`)
- `<digits>` are the identifier digits, written little-endian
- `<extension>` is the file type (`.md`, `.foo`, `.snap`, ...)

The **sort key** (chronological order) is `int(reversed(digits))`.

Examples:

| Filename | Identifier Digits | Sort Key |
|----------|-------------------|----------|
| `FOOP-1.md` | `1` | 1 |
| `FOOP-9.md` | `9` | 9 |
| `FOOP-01.md` | `01` | 10 |
| `FOOP-62.md` | `62` | 26 |
| `FOOP-72.md` | `72` | 27 |
| `Fir__check_parent__001.foo` | `001` | 100 |
| `Fir__check_parent__100.foo` | `100` | 001 |

### Rules

1. **Consecutive sort keys.** No gaps. Sort key N+1 follows sort key N.
2. **Identifier digits are the identifier.** In prose, reference the digits
   as they appear in the filename, never the sort key.
3. **Zero-padded to fixed width within a domain.** FOOPs use variable-width
   (the little-endian convention inherently pads). Snapshot tests within a
   component prefix use a fixed width (e.g., 5 digits: `00001` through `99999`).
4. **Monotonically increasing sort keys.** New artifacts always get the next
   sort key. Never reuse or skip.

### Command: `fns`

The current `foop_check.py` is replaced by a short command `fns` — a thin
executable wrapper at `docs/scripts/fns` (or a shell alias) that invokes the
generalized Python script. From the workspace root:

```bash
# Check consecutive numbering in a directory
fns check --dir docs/foop/ --prefix FOOP-

# Get the last identifier
fns get_last --dir docs/foop/ --prefix FOOP-

# Generate the next identifier
fns gen_next --dir docs/foop/ --prefix FOOP-

# List all in chronological order
fns list --dir docs/foop/ --prefix FOOP-

# Output a markdown table (grep-filterable)
fns markdown --dir docs/foop/ --prefix FOOP-

# Snapshot tests — different directory, different prefix pattern
fns check --dir foolish/foolish-ubca/snapshot_tests/input/ --pattern '__(\d+)\.foo$'
```

The Python implementation lives at `docs/scripts/fns_check.py`. The wrapper
(`docs/scripts/fns`) is a one-liner with a `#!/usr/bin/env bash` shebang:

```bash
#!/usr/bin/env bash
exec python3 "$(dirname "$0")/fns_check.py" "$@"
```

Both files move from `docs/foop/scripts/` to `docs/scripts/`.

#### `fns markdown` — Machine-readable summary table

The `markdown` subcommand reads frontmatter and plan files to produce a
grep-filterable markdown table. By default it outputs **all FOOPs**. An
optional `#-N` argument shows the N largest (most recent) FOOP numbers:

```bash
# All FOOPs
fns markdown --dir docs/foop/ --prefix FOOP-

# Largest 10 FOOP numbers (most recent 10)
fns markdown --dir docs/foop/ --prefix FOOP- #-10

# Largest 5
fns markdown --dir docs/foop/ --prefix FOOP- #-5
```

Output (all FOOPs):

| ID | Sort | Title | Status | Deps | Worktree | Branch |
|----|------|-------|--------|------|----------|--------|
| FOOP-1 | 1 | FOOP Process Itself | merged | — | no | — |
| … | … | … | … | … | … | … |
| FOOP-62 | 26 | UBCa Two-Store ProtoBrane | plan_coded | FOOP-1 | yes | foop-62-ubca-mimo |
| FOOP-72 | 27 | FNS and Snapshot Organization | draft | — | no | — |

**Columns:**

| Column | Source | Values |
|--------|--------|--------|
| **ID** | Filename digits | `FOOP-62`, `FOOP-72`, … |
| **Sort** | Reversed digits (chronological) | `26`, `27`, … |
| **Title** | Frontmatter `title:` field | Truncated to 60 chars |
| **Status** | Derived: spec status → plan execution → completion | See lifecycle below |
| **Deps** | Frontmatter `dependencies:` field | Comma-separated FOOP IDs or `—` |
| **Worktree** | Exists? (checks `WORKTREE_FULL_FS_PATH` on disk) | `yes`, `no` |
| **Branch** | `WORKTREE_BRANCH_NAME` from spec | Branch name or `—` |

**Unified status continuum.** A FOOP has one status that flows from
specification through plan execution to completion. The `fns` command
derives it by reading the spec frontmatter, checking for a plan file,
evaluating plan checkboxes, and verifying worktree existence:

| Status | Condition |
|--------|-----------|
| `draft` | Spec `status: Draft` |
| `brewing` | Spec `status: Brewing` |
| `final` | Spec `status: Final`, no plan checkboxes checked yet |
| `plan_started` | Plan exists, at least one checkbox is `[x]` |
| `plan_coded` | Plan exists, ≥50% of non-DECIDED checkboxes are `[x]` |
| `plan_completed` | Plan exists, all actionable checkboxes are `[x]` or `[-]` (canceled) |
| `merged` | Spec `status: Complete` or all phases done and merged to alpha |

The continuum is linear: `draft` → `brewing` → `final` → `plan_started` →
`plan_coded` → `plan_completed` → `merged`. A FOOP cannot skip stages —
it must pass through `final` before reaching any `plan_*` status, and
`plan_started` before `plan_coded`.

**Worktree column is separate.** The worktree existence (`yes`/`no`) is
its own column, not folded into status. A FOOP at `final` status may or
may not have a worktree yet. A FOOP at `plan_started` should have one
(if the spec declares a worktree), but the column reports actual disk
state, not expected state.

#### FOOP Dependencies

A FOOP can declare dependencies on other FOOPs via the frontmatter field
`dependencies:`. This is a comma-separated list of FOOP identifiers:

```yaml
---
foop: 27
title: Foolish Numbering System (FNS) and Snapshot Test Organization
author: Sisyphus <agent>
status: Draft
type: Standards
created: 2026-06-17
phase: phase-0
supersedes: []
dependencies: FOOP-1, FOOP-26
---
```

When `dependencies:` is present, the FOOP body **must** include a
`## Dependencies` section that explains each dependency:

```markdown
## Dependencies

- **FOOP-1** (FOOP Process) — This FOOP extends the FOOP process itself
  by adding the FNS convention and the `fns` command. FOOP-1 defines the
  base process that this FOOP builds upon.
- **FOOP-26** (UBCa Two-Store ProtoBrane) — This FOOP promotes the
  numbering convention used by FOOP-26 (UBCa) to a general-purpose system.
  UBCa's snapshot tests are the primary consumer of the new FNS snapshot
  naming convention.
```

The `fns markdown` command shows dependencies in the **Deps** column as
a comma-separated list of FOOP IDs, or `—` when none are declared.

**Dependency rules:**
1. Dependencies must reference existing FOOPs (validated by `fns check`).
2. A FOOP cannot depend on a later-numbered FOOP (no forward dependencies).
3. Circular dependencies are rejected by `fns check`.
4. The `## Dependencies` body section is mandatory when `dependencies:` is
   non-empty; `fns check` flags its absence.

**Grep-friendly design.** Each column is pipe-delimited so downstream
systems can filter:

```bash
# All Brewing FOOPs
fns markdown --dir docs/foop/ --prefix FOOP- | grep '| brewing |'

# FOOPs with a worktree but plan not completed
fns markdown --dir docs/foop/ --prefix FOOP- | grep '| yes |' | grep -v '| plan_completed |'

# All FOOPs actively being coded
fns markdown --dir docs/foop/ --prefix FOOP- | grep '| plan_coded |'

# FOOPs that are final but no worktree yet
fns markdown --dir docs/foop/ --prefix FOOP- | grep '| final |' | grep '| no |'
```

#### AI Agent Consumption Patterns

Research across AI-native task tools (Markplane, mdpm, Postal, ai-todo,
claude-plan-bridge, AgentFlow, PANDA, AGENA, OpenAI Symphony) reveals
consistent patterns in what agents actually consume from project management:

| Pattern | Description | FNS Preemption |
|---------|-------------|----------------|
| **Grep-filterable tables** | Agents pipe `fns markdown \| grep` to find relevant items | ✅ `fns markdown` with pipe-delimited columns |
| **Context summaries** | ~1000 token compressed summaries of project state | `fns summary` — condensed view of active FOOPs |
| **INDEX routing** | Scan an INDEX (~200 tokens), load only needed items | `fns list` — lightweight index before `fns markdown` |
| **Session rehydration** | New session restores task list from disk | `fns rehydrate` — emit agent-readable task list |
| **Status as directory** | `ls tasks/active/` tells state instantly | `fns status` — quick status check without full table |
| **Dependency tracking** | `blocks`/`depends_on` between tasks | ✅ FOOP `dependencies:` frontmatter + `## Dependencies` body section |
| **Work logs** | Append-only history of what was tried | FOOP plan checkboxes with timestamps serve this role |
| **Standup summaries** | Auto-generated "what's happening" | `fns standup` — one-line summary per active FOOP |

**Immediate features** (in scope for this FOOP):
- `fns markdown` — grep-filterable table ✅ (specified above)
- `fns list` — lightweight index (already exists)
- `fns summary` — condensed view of active FOOPs (future subcommand)

**Future features** (out of scope, but anticipated):
- `fns rehydrate` — emit agent-readable task list for session startup
- `fns standup` — one-line summary per active FOOP

### Applying FNS to Snapshot Tests

Snapshot test files use the naming convention:

```
<Component>__<behavior>__<digits>.foo
```

Where:
- `<Component>` is the FIR kind or subsystem being tested
- `<behavior>` describes what is verified
- `<digits>` are the FNS identifier digits (fixed 5-digit width)

Examples:
- `ConstantInt__literal_evaluation__00001.foo`
- `Fir__check_parent_linkage__00042.foo`
- `Search__anchored_forward__00103.foo`
- `Integration__full_program_all_features__00500.foo`

The trailing digits provide chronological ordering. The prefix provides
component grouping. Together they give both timeline and categorization.

## Snapshot Testing Documentation

### New document: `AGENTS/snapshot.md`

Consolidates all snapshot testing knowledge into one file:

1. **Snapshot format** — `.snap` file structure (INPUT, RESULT, signatures)
2. **Workflow** — run test → `.snap.new` → human review → accept
3. **AI agent constraints** — never auto-accept, never `INSTA_UPDATE=always`
4. **Signature verification** — `verify_signatures` tool, key types, re-signing
5. **Test harness** — `SnapshotSuite`, runner, cross-validation
6. **Commands reference** — all `cargo insta` and `verify_signatures` commands
7. **FNS numbering for snapshots** — naming convention, fixed-width digits

Content is extracted from:
- `AGENTS.md` — approval workflow, signature section, AI agent warnings
- `README.md` — snapshot test commands
- Inline comments in `snapshot_suite.rs` across crates

After consolidation, `AGENTS.md` and `README.md` reference `AGENTS/snapshot.md`
with a single pointer each.

## FNS Skill for AI Agents

A skill file enables AI agents (Claude Code, OpenCode, Cursor, etc.) to use the
Foolish Numbering System without memorizing the convention or hunting for the
script. The skill is loaded on-demand whenever an agent needs to create, check,
or list FNS-numbered artifacts.

### Skill file: `docs/skills/fns/SKILL.md`

The skill lives at `docs/skills/fns/SKILL.md` — a standard skill location
discoverable by both Claude (via `.claude/commands/` or inline skill references)
and OpenCode (via the skills registry).

**Trigger phrases** (load this skill when the user says any of):
- "create a new snapshot test"
- "what's the next FNS number"
- "check FNS numbering"
- "list snapshot tests"
- "rename snapshots to FNS"
- "Foolish numbering"
- "fns"

### Skill content

The skill provides:

1. **Convention summary** — little-endian digits, sort key reversal, fixed width
2. **Command location** — `fns` (wrapper at `docs/scripts/fns`) with usage examples
3. **Domain registry** — which directories use which prefix patterns:

| Domain | Directory | Pattern | Width |
|--------|-----------|---------|-------|
| FOOPs | `docs/foop/` | `FOOP-(\d+)\.md` | variable |
| UBCa snapshots | `foolish/foolish-ubca/snapshot_tests/input/` | `.*__(\d+)\.foo$` | 5 |
| UBCb snapshots | `foolish/foolish-ubcb/snapshot_tests/input/` | `.*__(\d+)\.foo$` | 5 |
| Core snapshots | `foolish/foolish-core/snapshot_tests/input/` | `.*__(\d+)\.foo$` | 5 |

4. **Actionable commands** — exact `fns` invocations for each domain
5. **Naming guide** — how to construct a snapshot filename:
   - Pick component: `Fir`, `Search`, `Brane`, `Scope`, `Operator`, ...
   - Describe behavior: `check_parent_linkage`, `anchored_forward`, ...
   - Get next number: `fns gen_next --dir ... --pattern ...`
   - Assemble: `<Component>__<behavior>__<digits>.foo`

### Integration

**OpenCode:** Register the skill in the project's skill list so it appears in
the available skills catalog. The skill name is `fns`.

**Claude:** Add a slash command or skill reference in `.claude/` that points to
`docs/skills/fns/SKILL.md`. The command name is `/fns`.

**Cursor/other:** The `SKILL.md` is a self-contained markdown file that any
agent can read directly. Place it in a well-known location and reference it
in project instructions.

### Example skill invocation

```
User: "create a new snapshot test for Search anchored forward resolution"

Agent (with fns skill loaded):
  1. Run: fns gen_next --dir foolish/foolish-ubca/snapshot_tests/input/ --pattern '__(\d+)\.foo$'
  2. Get: next digits = "00104" (sort key 4100)
  3. Construct: Search__anchored_forward_resolution__00104.foo
  4. Create the .foo file with test content
  5. Verify: fns check --dir ... --pattern ...
```

## Document Changes

### New files
- `AGENTS/fns.md` — Foolish Numbering System specification
- `AGENTS/snapshot.md` — Snapshot testing documentation
- `docs/scripts/fns` — Shell wrapper (the `fns` command)
- `docs/scripts/fns_check.py` — Generalized numbering script (renamed from `foop_check.py`)
- `docs/skills/fns/SKILL.md` — FNS skill for AI agents

### Modified files
- `foop.md` — Replace FNS section with a pointer to `AGENTS/fns.md`; retain
  FOOP-specific process (numbering, two-file layout, plan construction,
  checkbox lifecycle) which remains FOOP-specific
- `AGENTS.md` — Replace snapshot testing sections with pointers to
  `AGENTS/snapshot.md`; replace FOOP numbering summary with pointer to
  `AGENTS/fns.md`
- `README.md` — Replace snapshot commands section with pointer to
  `AGENTS/snapshot.md`

### Deleted files
- `docs/foop/scripts/foop_check.py` — relocated to `docs/scripts/fns_check.py`

## Scope

This FOOP is **documentation and tooling only**. No Rust code changes, no
behavior changes, no snapshot file renames (those are a separate FOOP if
desired). The goal is to establish the FNS as a general convention and
consolidate snapshot documentation.

## Verification

- `AGENTS/fns.md` exists and covers the little-endian convention comprehensively
- `AGENTS/snapshot.md` exists and covers the full snapshot workflow
- `fns` command works for both FOOP and snapshot directories
- `fns markdown` produces a correct, grep-filterable table with all columns including Deps
- `fns markdown #-N` correctly filters to the N largest FOOP numbers
- `fns check` validates `dependencies:` — no forward deps, no circular deps, no missing FOOPs
- FOOPs with `dependencies:` have a corresponding `## Dependencies` body section
- `foop.md`, `AGENTS.md`, `README.md` reference the new documents
- No scattered copies of the numbering convention remain

## Last Updated

**Date**: 2026-06-17
**Updated By**: Sisyphus / Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial draft.
