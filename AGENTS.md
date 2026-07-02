# AI Agent Development Guide

This document provides instructions for AI agents (including Claude Code, GitHub Copilot, Cursor, and other AI coding assistants) working on the Foolish project.

## Use Common Sense
Apply industry standard best practices liberally. Use colloquial rust to most correctly, efficiently and readably implement the system. Colloquial rust tend to be most supported and most optimized.
Documentation is organized under docs/ in subdirectories: howto/ (tutorials), why/ (philosophy), how/ (engineering), todo/ (project tracking), and vintage_legacy/ (legacy documents being reorganized).


## Development process
Due to the nature of human-driven development, AI should always write the tests first. Approval tests and unit tests, write the tests with most important features, and unclear corner cases written as tests to not only check behavior, but also to document what it looks like.
Ask permission before coding new features or repairing bugs in languages other than Rust or Foolish.

## Overview

Foolish is a revolutionary programming language implemented in Rust. This guide helps AI agents navigate the unique build requirements and environment-specific setup needed for development.

**Multiple AI agents collaborate on this project.** This document serves as the shared knowledge base for all AI coding assistants (Claude Code, GitHub Copilot, Cursor, and others) to ensure consistent understanding of the project structure, build processes, testing workflows, and coding conventions.

## Build Requirements

- **Rust**: current stable toolchain

## Project Segmentation
Software projects May be large or small. Their complexity and diffiulty may also vary. Generally speaking we use these terms for disjoint components of softare:
  - Major
    - This is a noun, That "specification file is for a major", or an adjective "that is a major specification"
    - This is a very large feature, that may break many existing functionalities while implementing
    - Some extensive exchange with human may be required.
    - Some multi-modal analysis, including web-searches, prototyping, analysis, etc.
    - aka Major Feature, Major release, Major upgrade, etc.
    - Example: "Centralize and fully sepcification of CLI interface by gathering features from all the existing implementations. Resolve any conflicts or redundancies. Then update all implementation to follow new specification."
    - Example: "DHT for discovering peers for different purposes: mutual attestation, calendar replication, capability-matching, etc."
  - Phase
    - a Major feature may be implemented in many phases
    - Example: Research, Discuss and Q&A with Human, Design and implement tests, Implementation feature, Code Review, Security Review, Fresh-eye review, merge to alpha, etc.
  - Stage
    - each phase may contain many stages
    - Example for Research: Analyze code, web search, pose research questions, combination and synthesis, etc.
  - Step
    - Each stage may be several steps.
    - Example: Search Arxiv, Search Google Schollar, Search wikipedia, Search reddit, Search Google Groups,
    - Example: Change the entire project name from "Fortias" to "Foretias".
  - Task
    - Each step may be several tasks.
    - Tasks are smaller very well defined jobs, typically using tool or simple updates.
    - Example: Alter spelling of "Fortias" to "Foretias" in all file names
    - Example: Alter spelling of "Fortias" to "Foretias" in C11 code.
    - Example: Alter spelling of "Fortias" to "Foretias" in rs code.
It is very important, given a request from user that correspond to a feature request or software change, to set a scope size. After scoping, perhaps the new request may be placed into an existing larger sized poject, or cause a split of existing project to form similar sized projects. Ultimately correctness and implementation efficiency is the goal achieved through organization, consideration and communication.

When request is small, you may combine Major/Phase/Stage into a single unit.

## Development Organization

### FOOP (Foolish Optimization Process)

FOOP documents are the Foolish equivalent of Python's PEP or Rust's RFC. They
propose, discuss, and track changes to the Foolish language and its reference
implementations.

> ## ⛔ STOP — READ `foop.md` BEFORE TOUCHING ANY FOOP's ⛔
> **WHEN you need to read or write foop, you MUST first read `foop.md` (in the
> repository root, next to this file) in full — every line. That document is the
> authoritative reference for the FOOP process, its philosophy, the numbering
> system, the file layout, plan construction, project/branch management, and the
> full checkbox lifecycle. The summary below covers only the few common,
> every-day uses; for anything beyond them, `foop.md` governs.

A few common every-day uses (full detail lives in `foop.md`):

- **Numbering & the helper utility.** FOOP numbering is *little-endian*: the
  filename digits ARE the identifier, while the `foop:` frontmatter is a
  separate sort key (the digits reversed). Manage numbering with the helper
  script — always run `gen_next` before creating a new FOOP:

  ```bash
  python3 docs/foop/scripts/foop_check.py check     # verify consecutive numbering
  python3 docs/foop/scripts/foop_check.py get_last  # most recent FOOP
  python3 docs/foop/scripts/foop_check.py gen_next  # filename for next FOOP
  python3 docs/foop/scripts/foop_check.py list      # all FOOPs in chronological order
  ```

- **The two files of a FOOP.** `FOOP-#.md` is the **specification** (the what, why,
  and how). `FOOP-#.plan.md` is a **checkboxed plan** meant to be executed
  sequentially from top to bottom(this is the how to get there roadmap). **Executing
  a FOOP plan requires reading BOTH `FOOP-#.md` and `FOOP-#.plan.md`** — the plan
  assumes the specification's context.

- **Constructing the plan.** How to decompose a specification into an ordered
  `FOOP-#.plan.md` (concrete sequential tasks, worktree lifecycle checkboxes,
  sub-task splits) is described in `foop.md`.

- **Checkbox lifecycle.** How to use checkboxes — completing them with a
  timestamp on the indented next line, delaying them with `[x]
  backburnered`, and cancelling them with `[x] Canceled` plus per-item `[-]`
  markers — is described in `foop.md`.

Whenever the task involves reading or writing FOOPs (creating, planning,
implementing, checking boxes, or merging), **read `foop.md` first** and follow
it exactly.



## Development tools
Please use plugins and mcp's for performing disk operations, file searches and file edits. Use fully specified regular expressions (covering various cases), through mcp or using `sed` directly. These means of editing are much faster than regenerating the entire document. Each time regexp is used to for updates, please reread updated document before replacing original document.  Use Github mcp to perform git related actions.

When commiting to Git, always state project segment and software version and model version:
```git
Major: Refactor CLI, Phase: Discussion with Human--complete
opencode 1.14.39, Qwen3.6-27B-AWQ-BF16-INT4
```

## Development Rules
**NEVER** start file changes for project Phase or larger WHEN any tests are broken.
**NEVER** start large project segment work WHEN ANY tests are broken even if there're notes indicating those breakage are known. The test has to be manually disabled by human OR repaired and committed.

**Exception:** Snapshots that have been reviewed by the human and contain `@agent` comments
(i.e. `.snap.new.check` files) are permitted to remain non-conformant. These represent known
issues that the human has inspected and accepted as work-in-progress. They will be fixed as
part of the impending work tracked in the plan.

## How To Write Rust Code

> ## ⛔ STOP — READ `rust_instructions.md` BEFORE TOUCHING ANY RUST ⛔
>
> **EVERY** coding agent — Claude Code, Copilot, Cursor, or any other — **MUST**
> read [`rust_instructions.md`](rust_instructions.md) at the repository root
> **before reading or writing a single line of Rust in this repository**, and
> **MUST** follow it. This is not optional and not negotiable.
>
> `rust_instructions.md` is the **single authoritative source** for how Rust is
> written here. It contains the full guidance — optimization priorities,
> ownership and borrowing, encapsulation, enum dispatch, error handling, the
> Foolish/Foretias project-specific rules (FIR semantics, compiler phases,
> cryptography, FFI, bindings), testing requirements, and the hard tooling gates
> (`cargo fmt`, `cargo clippy -D warnings`, tests). All of it formerly lived in
> this section and now lives there.
>
> If you are about to edit Rust and have not read `rust_instructions.md` this
> session, **stop and read it now.**

<!-- The detailed Rust guidance previously inlined here has been moved verbatim
     (and integrated with a cited general-Rust ruleset) into rust_instructions.md.
     Do not re-add Rust style content to AGENTS.md — keep it in one place. -->

## Environment Detection

## Important Safety Guide Rails
Agents shall **NEVER** take restricted actions. For example 'chmod a+rw file' is not permitted. The most an agent can do in those respects is to suggest user to perform the action and give the
command sequence with the first word in all caps: 'CHMOD a+rw file'. This ensures that even the user cannot copy and paste it blindly. Every line of a multi-line suggestion shall have first word
case inverted. So, for example if agent suggests running program "Agent --reset-context", it shall recommend to the user to type "aGENT --reset-context".

Restricted actions are:

 * Changing permissions on any file. For example: 'chmod a+rw filename'
 * Altering maven, git and other softare configuration files, these include, not exclusively, ".gitigore", ".git", '.claude', ...
 * Never alter any approved approval files matching pattern "*.approved.foo"
 * Never alter any approved approval Foolish files matching pattern "*.approved.foo" Even if it is to change the number of steps taken.
 * Never move, rename, or alter `.snap` files (approved snapshots).
 * Never move, rename, or alter `.snap.new.approved` files (human-approved, awaiting signing).
 * Never run `cargo insta accept` or `INSTA_UPDATE=always`.

For requesting restricted file changes, agents may suggest diff patch or full text of replacement content.


## Task Management
This project uses the todo skill for all task tracking. All todo files live
in docs/.../todo/ and are exclusively maintained by the skill — do not edit
them directly.

### Default session file
Each AI session writes to its own todo file by default:
docs/todo/AIAGENT-<session-id>.todo.md To switch to a project-specific todo
file, say "use the sprint-3 todo" or invoke /todo-use sprint-3 at any point
in the session.

### When starting any multi-step task
Before executing, read the active todo file and either map the work to
existing open items or add new ones. Write a session started Log entry
summarizing the plan and which IDs will be worked.

### While executing
Log progress on each item before starting it (in progress) and close it
with a meaningful summary when done (/todo-done, /todo-abandon, or
/todo-cancel). If new work is discovered mid-task, add it immediately.

### When finishing or pausing
Write a session ended Log entry listing what was completed, what remains,
and any context the next session needs.  General rule Keep the todo file
synchronized with actual work at all times using the commands of the skill.
It is the record of what happened, not just what is planned.


## Build Commands

All commands below run from the repository root `/home/hcbusy/foolish-rust`.

### Rust Implementation

```bash
cd /home/hcbusy/foolish-rust

cargo check --workspace                          # Quick check (fastest validation)
cargo build --workspace                          # Build everything
cargo build --workspace --release               # Release build (LTO, stripped)
```

Binary after release: `target/release/foolish`

### Unit Tests

```bash
cargo test --workspace                           # All unit tests
cargo test -p foolish-core                       # One crate
cargo test -p foolish-core -- brane_search       # Specific test (substring match)
```

### Approval Tests (insta snapshots)

**foolish-core** snapshot tests use `SnapshotSuite` and `insta`. Snapshots live in
`foolish-core/snapshot_tests/approved/`.

**foolish-ubca** snapshot tests also use `SnapshotSuite`. Snapshots live in
`foolish-ubca/snapshot_tests/approved/`.

Each snapshot file has the format:
```
INPUT:
```foolish
<source>
```
[0] RESULT:
```hssnap
<humanizing-sequencer output>
```
Public key: <hex>
Foolish signature: <base64>
HS signature: <base64>
```
Signatures are Ed25519, derived from a passphrase via Argon2id. The default (empty)
passphrase is the computer/AI-agent key. Human reviewers can re-sign with their own
passphrase using the `verify_signatures --write-verified` tool.

#### Key commands

```bash
cargo test -p foolish-core --lib                           # all core snapshot tests
cargo test -p foolish-core --lib -- approval_all           # just the approval suite
cargo test -p foolish-ubca --lib                           # all UBCa snapshot tests
```

#### ⚠️ CRITICAL: NEVER AUTO-ACCEPT SNAPSHOTS ⚠️

**AI AGENTS MUST NEVER RUN `cargo insta accept` OR `INSTA_UPDATE=always`.**

Snapshots are the authoritative record of correct Foolish VM behavior. They are cryptographically
signed to distinguish AI-generated output from human-reviewed output. Auto-accepting snapshots
bypasses this verification and corrupts the approval chain.

**The snapshot review workflow (human-driven):**

1. AI runs `cargo insta test -p <crate> --lib` to generate `.snap.new` files.
2. Human runs `foolish_review.sh <crate>` — opens each `.snap.new` in `vimdiff` against the
   approved `.snap`. The human either:
   - **Approves** (no edits): file is moved to `.snap.new.approved`.
   - **Flags** (adds `@agent <comment>`): file is moved to `.snap.new.check` for AI to address.
3. Human runs `accept_approved.sh <crate>` — signs all `.snap.new.approved` files with their
   passphrase, then moves each to `.snap` (replacing the old approved snapshot).
4. AI addresses any `.snap.new.check` files flagged with `@agent` comments.

**AI is NEVER allowed to:**
- Move, rename, or alter `.snap` files.
- Move, rename, or alter `.snap.new.approved` files.
- Run `cargo insta accept` or `INSTA_UPDATE=always`.
- Silently accept snapshots that change evaluation semantics.

**Remember:** A snapshot change may indicate a VM bug, not a formatting improvement.
Always verify the semantic correctness of output before accepting.

#### When bug fixes are complete — present this review to the human

```bash
# Set this to the module being reviewed:
module_path=foolish-ubca   # e.g. foolish-core, foolish-ubca

cd /home/hcbusy/foolish-rust
cargo clean -p $module_path
cargo insta test -p $module_path --lib
./accept_approved.sh $module_path
```

Human reviews with:
```bash
./foolish_review.sh $module_path
./accept_approved.sh $module_path
```

#### Signature verification

```bash
# Build the tool (once, from the repository root)
cargo build -p foolish-core --bin verify_signatures

# Verify all snapshots with the default computer key
./target/debug/verify_signatures foolish-core/snapshot_tests/approved/

# Verify a single file
./target/debug/verify_signatures path/to/file.foo.snap

# Re-sign files with the computer key (e.g. after regenerating snaps manually)
./target/debug/verify_signatures --write-verified foolish-core/snapshot_tests/approved/

# Re-sign with a human passphrase (reads from stdin)
./target/debug/verify_signatures --stdin-passphrase --write-verified foolish-core/snapshot_tests/approved/
```

Output columns: `key match: yes, foolish: yes, hs: yes  <path>`
Values are `yes`, ` no` (space-padded), or `n/a` (unsigned/old-format file).

### CLI Usage

```bash
cargo run -p foolish-cli -- run path/to/program.foo    # Evaluate a .foo file
cargo run -p foolish-cli -- step path/to/program.foo   # Step-by-step (debug)
cargo run -p foolish-cli -- repl                       # Interactive REPL
```

### Unit Test Redability
Unit tests are required to test correctness of internal state of the FVM. There are some infrastructure built
to help this. Unit test can generate a scafolding of Foolish brane using Foolish language. The unit test can 
then alter the initialized Foolish FIR, adding/subtracting or otherwise mutating it into the desired testing
situation. It is free to use the parser, the FoolishIndex and the root Brane's '.search(...)' method to make
the test itself easier to read to human reviewers of the test.

#### NYES transition tests (`*_nyes_transitions`)
Every FIR kind has a unit test named `<kind>_nyes_transitions` in
`foolish-ubca/src/fir_kinds.rs` (tests module). Each steps the FIR to settled, records
the per-step NYES sequence, and asserts the progression via the shared `assert_progression`
helper: it must start `PREMBRIONIC`, end constanic, be monotone (no constanic → pre-constanic
regression), and reach the kind's expected terminal state. There are also context tests that
compile real Foolish and check the right thing is found in the right stage (e.g. IB search in
EMBRYONIC vs AB search in BRANING).

**REQUIREMENT:** when you add a NEW FIR kind, or add/change a NYES state or transition, you
MUST add or extend the corresponding `*_nyes_transitions` test(s) so the new progression is
documented and pinned. These are unit tests (NYES is internal FVM state), not approval cases.

### Approval Test
Approval tests demonstrate the behavior of the Foolish VM by writing inputs in '.foo' files, running a special
VM to produce a final result. Sometimes the results could be Constanic other times they could be NK. As long
as it matches the expected output byte for byte, it is correct. The approval test program outputs more than
just the final brane, it outputs alarms generated along the way as well as number of steps it took to execute
the FVM before the input Foolish file became isConstanic.

Separate languages read from the same test input resources directory to produce their own approval output.
A crossvalidation process checks that implementations in different languages are behaving identically.

**Snapshot workflow**: Run a test → if output differs, insta writes `.snap.new` →
**PRESENT TO HUMAN FOR REVIEW** → human runs `cargo insta review` or `cargo insta accept`.
AI agents MUST NEVER accept snapshots. See the ⚠️ CRITICAL section above.

## Clarifications
* When user mentions "path/" first interpret it as relative path from the directory where claude code was invoked. This is normal behavior for most unix apps, for example if I "cat path/file" that path is resolved from the current path.
* Never directly edit `.approved.foo` files

### The Unicellular Brane Computer (UBC)

The **UBC is the reference implementation of Foolish**. It implements a unique evaluation model based on branes (containment structures).

#### FIR (Foolish Internal Representation)

FIR objects represent expressions during evaluation and progress through a multi-stage state machine:

```
## NYES states (Not Yet Evaluated State):
- `PREMBRYONIC`: initial state, not yet stepped
- `EMBRYONIC`: stepping begun, task queue built
- `BRANING`: stepping in progress, draining child tasks
- `ECONSTANIC` (say "ee-con-STAN-nic"): Exactly CONSTANt IN Context — search performed, nothing found. May gain value via recoordination.
- `WOCONSTANIC`: Waiting On CONSTANICs — all searches found, dependencies themselves constanic.
- `CONSTANT`: Fully evaluated — a genuine value.
- `INDEPENDENT`: Self-contained constant — no context dependencies.
- `NK`: Not Knowable — provably unfindable (`???`). Terminal.

**Constanic** (adjective): a FIR in ANY terminal state — ECONSTANIC, WOCONSTANIC, CONSTANT,
INDEPENDENT, or NK. Pre-constanic (nigh) = PREMBRYONIC, EMBRYONIC, BRANING — more stepping
is appropriate. See FOOP-62 §Terminology for the authoritative UBCa definition.
```

#### Brane Reference Semantics: AB and IB

**Ancestral Brane (AB)** and **Immediate Brane (IB)** are critical context for name resolution:
- **IB**: Current context accumulated so far (lines before current expression)
- **AB**: Parent brane context containing the defining expression and its AB/IB

**Detachment and Coordination**: When a brane is referenced by name:
1. The brane was already partially resolved in its original AB/IB context
2. A clone is **detached** from its original AB/IB
3. The clone is **recoordinated** with new AB (the containing brane) and new IB (preceding lines)
4. Previously failed name searches can now resolve in the new context

In UBC implementation, this means creating a modified clone with new context. See `docs/vintage_legacy/ECOSYSTEM.md` for detailed semantics.

### Test Infrastructure

**Two-Tier Testing:**

1. **Unit Tests** — focused component tests in Rust (`cargo test`)
2. **Approval Tests** — insta snapshot-based integration tests (see Build Commands above)
### Foolish Terminology (from STYLES.md)

- **Foolisher** - developer/user of Foolish
- **Nye** (say "nigh") - Not Yet Evaluated
- **NYES** (say "nice") - Not Yet Evaluated State
- **No-no** - The `???` unknown value
- **Constanic** (say "cons-TAN-nic") - Constant in Context. Any terminal NYES state:
  ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, or NK. Pre-constanic (nigh) = needs more stepping.
- **Ordinate** - a name associated with a brane
- **Coordinate** - brane member names used for relational access
- **Lexed** - feature parses to AST
- **Interpreted** - feature fully implemented in VM

### Code Style

- Tabs for depth markers (reduces storage)
- 108 character width for documents
- `.foo` extension for Foolish programs
- Full-width space (＿) in approval tests shows indentation precisely
- Variable names follow power-law distribution (mean 3.5 chars short, 5 chars long)
- Use diverse Unicode: Latin, Greek, Cyrillic, Hebrew, Arabic, Chinese, Sanskrit

### Commit Message Format

Include AI agent and model information in commit messages:
```
Summary of changes

Detailed description of what was changed and why...

[AI Agent Name] [Version] / [Model ID]
```

Examples:
```
Add RegExp search to brane operations

Implemented pattern-based search using RegexpSearchFiroe.
Added tests and updated documentation.

Claude Code v1.0.0 / claude-sonnet-4-5-20250929
```

```
Fix type inference bug in FIR resolution

GitHub Copilot / gpt-4
```

### Computational Tools Preference
When it is available, prefer to use python repl to perform math calculations, complex string manipulations, or even to perform regular expression substitutions.

## Documentation

### Directory Structure

- **`docs/ubc1/how`** - UBC1 engineering documentation - operational semantics, implementation details, reference
- **`docs/ubc1/todo`** - UBC1 project tracking - active development roadmap
- **`docs/ubc0_1/how`** - UBC0_1 engineering documentation
- **`docs/ubc0_1/todo`** - UBC0_1 project tracking
- **`docs/howto`** - "How to Express it in Foolish" - literate programming tutorials as .foo files
- **`docs/why`** - "Philosophy of Foolish" - origins, inspirations, design philosophy
- **`docs/vintage_legacy`** - Legacy documentation (being reorganized into the above directories)
- **`docs/todo`** - todo lists

### Additional Resources

For complete details on:
- **How to write Rust (REQUIRED before any Rust work) → See `rust_instructions.md`**
- Language features and semantics → See `README.md`
- Terminology and conventions → See `docs/vintage_legacy/STYLES.md`
- UBC architecture → See `docs/vintage_legacy/ECOSYSTEM.md`
- Name resolution and search → See `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md`
- Claude-specific guidance → See `.claude/CLAUDE.md`

## Quick Reference

## Markdown File Update Protocol

**IMPORTANT**: Whenever ANY AI agent modifies a `*.md` file in this repository, the agent MUST update the "## Last Updated" section at the end of that file with:

1. **Current timestamp** (YYYY-MM-DD format)
2. **Agent identifier** (as specific as possible, including model name/version)
3. **Brief summary** of what was changed

Example format:
```markdown
## Last Updated

**Date**: 2026-01-15
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-5-20250929
**Changes**: Added detailed UBC architecture documentation and test infrastructure workflows
```

This ensures all AI agents can track who modified documentation and when, maintaining clear collaboration history.

## Maintenance Instructions

**Weekly Check**: After one week past the day of last update to AGENTS.md (either by git timestamp or the Last Updated section below), please review this file for accuracy:

1. Verify that project structure, build commands, and setup instructions are still accurate
2. Check if new project conventions or workflows need documentation
3. Ensure UBC architecture details match current implementation
4. Confirm test infrastructure documentation reflects actual test structure
5. Verify that all AI agents have access to necessary information
6. Check that environment detection and CCW setup instructions are current
7. Propose updates to the user if discrepancies are found
8. Update the Last Updated section below--even if user makes no changes

When proposing updates, explain what has changed and why the documentation needs adjustment. After user review, update the "Last Updated" date below whether changes are accepted or the user confirms current state is acceptable.

## Last Updated

**Date**: 2026-06-22
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Added "NYES transition tests (`*_nyes_transitions`)" subsection under Unit Test
Redability: every FIR kind has a `<kind>_nyes_transitions` unit test (assert_progression);
new FIR kinds / NYES states/transitions MUST extend these tests. (FOOP-62 #16.)

**Date**: 2026-06-11
**Updated By**: Sisyphus / mimo-v2.5-pro
**Changes**: Updated NYES state section with complete UBCa states (PREMBRYONIC through NK).
Added "Constanic" to Foolish Terminology. Corrected pronunciation: "cons-TAN-nic" not
"CON-STAN-NICK".

**Date**: 2026-06-10
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8

### MISC

#### Embedded Communications

If any file, other than this example in the AGENTS.md, contain a parenthetical comment, anywhere, it is a request for agent to comment based on the context surrounding that comment.
```markdown
Blah blah, some texxt (@Agent, do you think that word is mispelled?)
```

or
```python
def fibonacii(x):
	# @agents, errrr, terminal case? spelling? did you even run this?
	return fibonacii(x-1) + fibonacii(x-2)
```

or even not in a comment
```python
def add (x):
@AGENT, this is just plain wrong!
	return x+y;
```
The expectation is for agent to consider, discuss, and resolve the concern
that follows various capitalizations of `@agent` or `@agents`. Resolution, once achieved, also means the parenthetical comment can be completely removed.

If this form of embedded communication is discussed while performing another task, determin if it is relevant or interferes with current task. In some cases, this causes an immediately actionable response, other times, the encounterance results in an extra '[ ] TODO:human concern at file FILENAME line LINE_NUMBER' added to current task list to investigate. In some cases, if it is clear that the situation is too complex or require too much context, it may become a "[ ] TODO: write a specification and plan to address human concern at file FILENAME line LINE_NUMBER"



#### Uncertainty and Other Utterances in Conversing with Human

Expressions of uncertainty and hypotheticals, such as "perhaps", "maybe", "possible", "what if", "in case". These words does not mean a firm directive from human to either pause work, or make large changes. It means human wants a todo task enqueued, perhaps to be done immediately, to explore options regarding the statement. In the last sentence, the perhaps suggests an option that can be explored, and it also highlight the possibility of the task not at the top of the todo list. More than anything else, the statement suggests human is thinking about the issue and you can help that thinking process.

"Wait!" is almost always typed when humans are reading the previous output and found something objectionable. "Wait!" meant stop that, something was wrong. This also implies whatever they ask about, it is highly unlikely they read through the reast of the response. Good or bad, that is human nature, please accomodate this behavior as a supportive agent. After addressing the concern following "wait!", the you can summarize what you meant to say after the output that the human said "Wait!" to--where it is is inferred based on the question or comment after "WAit!", when in doubt, summarize the whole response in the context of having addressed the human's concern.

"Continue." is uttered when the humans sees output on the screen that they think is incomplete. The best course of action, irrespective of actual status, is to summarize the progress made in the most recent few turns of conversation. If indeed the progress was ended or blocked by nonresponsive sub-agents, then take approrpiate action. If the short term task is truely complete, still output the summary, but also present outstanding todo items as well as other possible next steps for human to decide. Human may decide previous task is not complete and needs more work, or they may agree previous task was complete and move on to one of the options for next steps.


#### When in Doubt

When uncertain, choose the design that is easiest to prove correct, easiest to test, and easiest for the next human to understand.

Correctness first. Then readability and maintainability. Then efficiency. Then principles and asethetics.

