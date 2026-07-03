---
foop: 30
title: Repository Cleanup — Remove Dead Code, Flatten Workspace, Establish UBCa as Reference Implementation, Rename Main to jia
author: Sisyphus / mimo-v2.5-pro
status: Implementing
type: Standards
created: 2026-07-01
phase: meta
supersedes: []
begun: [x]
---

# FOOP-03: Repository Cleanup — Remove Dead Code, Flatten Workspace, Establish UBCa as Reference Implementation, Rename Main to jia

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

## Abstract

This FOOP performs a comprehensive cleanup of the Foolish repository:

1. **Remove all JVM traces** — Java, Scala, Maven files, CI workflows, badges,
   and documentation references.
2. **Remove dead Rust crates** — `foolish-web`, `foolish-ubcb`,
   `foolish-ubcb-cli`. Only `foolish-core` (UBCa), `foolish-parser`, and
   `foolish-cli` survive.
3. **Flatten the workspace** — move `foolish/*` contents to the repository root,
   eliminating the unnecessary nesting directory.
4. **Update all documentation** — rewrite AGENTS.md, README.md, and other docs
   to describe `foolish-core` as the sole UBCa reference implementation, with
   accurate key terminology (FIR, Nyes states, constanic, etc.) and correct
   paths throughout.
5. **Rename main branch to `jia`** — on GitHub, with contributor instructions.

After this FOOP, the repository is a clean, idiomatic Rust workspace at its
root with a single FVM implementation (UBCa) and documentation that accurately
reflects the current state of the project.

## Motivation

### Dead Code Creates Confusion

The repository currently contains:
- **JVM implementation** (Java/Scala via Maven) — abandoned, tests don't pass,
  creates broken CI badges on the README
- **foolish-web** — a web server crate nobody maintains
- **foolish-ubcb** / **foolish-ubcb-cli** — an alternative FVM implementation
  that duplicates foolish-core's functionality

A newcomer seeing six crates, Java badges, and Maven commands would reasonably
ask "which one do I use?" The answer is always "foolish-core" — so let's make
that obvious by removing everything else.

### Non-Standard Layout

All Rust code lives under `foolish/` rather than at the repo root. This forces
every build command to include `cd foolish` and every documentation reference
to carry the `foolish/` prefix. Idiomatic Rust workspaces put `Cargo.toml` at
the repository root.

### Documentation Is Stale

AGENTS.md and README.md reference Java 25, Scala 3.3.7, ANTLR 4.13.2, Maven
build commands, cross-validation between implementations, and a multi-crate
architecture that no longer exists. The terminology section doesn't describe
UBCa's FIR state machine or Nyes states accurately.

### Branch Naming

`main` is generic. `jia` (家, "home") is the project's identity. Renaming
during a major cleanup minimizes disruption — contributors expect changes.

## Specification

### 1. Remove JVM Artifacts

#### 1a. Delete Files

| File/Directory | Action |
|----------------|--------|
| `.github/workflows/java-tests.yml` | Delete |
| `.github/workflows/scala-tests.yml` | Delete |
| `docs/ubc1/todo/scala-mvp/` | Delete entire directory |
| `samples/` | Delete if only contains Java/Scala samples; keep if contains `.foo` files |

#### 1b. Update `.github/workflows/tests.yml`

Remove any JVM cross-validation references. Keep Rust-only CI.

#### 1c. Update `.gitignore`

Remove JVM-specific entries that don't apply to Rust:
- `*.class`, `*.jar`, `*.war`, `*.ear`
- `pom.xml.tag`, `pom.xml.releaseBackup`, etc.
- `ivy-cache/`, `ivy.xml.original`, `ivy-report.xml`

Keep general IDE entries (`.idea/`, `.vscode/`, `.DS_Store`).

### 2. Remove Dead Rust Crates

#### 2a. Delete Crate Directories

| Directory | Action |
|-----------|--------|
| `foolish/foolish-web/` | Delete |
| `foolish/foolish-ubcb/` | Delete |
| `foolish/foolish-ubcb-cli/` | Delete |

#### 2b. Surviving Crates

After cleanup, the workspace contains exactly three crates:

| Crate | Role |
|-------|------|
| `foolish-core` | UBCa — the Unicellular Brane Computer (reference implementation) |
| `foolish-parser` | Parser — lexes and parses `.foo` source into AST/FIR |
| `foolish-cli` | CLI — `run`, `step`, `repl` commands for executing Foolish programs |

Dependency chain: `foolish-parser` ← `foolish-core` ← `foolish-cli`

### 3. Flatten Workspace

Move the contents of `foolish/` to the repository root.

#### 3a. Files to Move

| From | To |
|------|----|
| `foolish/Cargo.toml` | `Cargo.toml` |
| `foolish/Cargo.lock` | `Cargo.lock` |
| `foolish/foolish-core/` | `foolish-core/` |
| `foolish/foolish-parser/` | `foolish-parser/` |
| `foolish/foolish-cli/` | `foolish-cli/` |

#### 3b. Files to Delete (not move)

| File | Reason |
|------|--------|
| `foolish/target/` | Build cache; will regenerate |
| `foolish/mcp.log.*` | Log files; not part of project |
| `foolish/.claude/` | Redundant with root `.claude/` |
| `foolish/.omo/` | Redundant with root `.omo/` |

#### 3c. Update Cargo.toml Workspace Members

Before:
```toml
[workspace]
members = [
    "foolish-parser",
    "foolish-core",
    "foolish-cli",
    "foolish-web",
    "foolish-ubcb",
    "foolish-ubcb-cli",
]
```

After:
```toml
[workspace]
members = [
    "foolish-parser",
    "foolish-core",
    "foolish-cli",
]
```

#### 3d. Delete Empty Directory

Remove `foolish/` after all contents are moved.

### 4. Update All Documentation

Every file that references the old structure, old crates, or JVM tooling must
be updated. This is the largest part of the FOOP.

#### 4a. AGENTS.md — Full Rewrite of Key Sections

**Build Requirements** — remove Java, Scala, Maven, ANTLR references. Replace
with:

```
## Build Requirements

- **Rust**: current stable toolchain
```

**Build Commands** — remove all `cd foolish` prefixes. All commands run from
the repository root.

**Project Structure** — replace the multi-crate description with:

```
## Project Structure

The Foolish repository is a Rust workspace with three crates:

| Crate | Description |
|-------|-------------|
| `foolish-core` | UBCa — the Unicellular Brane Computer. The sole reference implementation of the Foolish language. |
| `foolish-parser` | Parser — lexes `.foo` source into AST, compiles to FIR. |
| `foolish-cli` | CLI — `run`, `step`, `repl` commands for executing Foolish programs. |
```

**Key Terminology / Architecture** — add or rewrite a section describing UBCa
as the reference implementation. Content should include:

```markdown
## Architecture

**UBCa** (Unicellular Brane Computer, reference implementation) is the sole
FVM (Foolish Virtual Machine). It lives in `foolish-core/` and implements the
complete evaluation model.

### FIR (Foolish Internal Representation)

FIR objects represent expressions during evaluation. Each FIR carries a **Nyes**
(Not Yet Evaluated State) that tracks its evaluation progress:

- `PREMBRYONIC` — initial state, not yet stepped
- `EMBRYONIC` — stepping begun, task queue built
- `BRANING` — stepping in progress, draining child tasks
- `ECONSTANIC` (ee-con-STAN-nic) — Exactly CONSTANt IN Context. Search
  performed, nothing found. May gain value via recoordination.
- `WOCONSTANIC` — Waiting On CONSTANICs. All searches found, dependencies
  themselves constanic.
- `CONSTANT` — Fully evaluated. A genuine value.
- `INDEPENDENT` — Self-contained constant. No context dependencies.
- `NK` — Not Knowable. Provably unfindable (`???`). Terminal.

**Constanic** (adjective): a FIR in ANY terminal state — ECONSTANIC,
WOCONSTANIC, CONSTANT, INDEPENDENT, or NK. Pre-constanic (nigh) =
PREMBRYONIC, EMBRYONIC, BRANING — more stepping is appropriate.

### Brane Reference Semantics

**Ancestral Brane (AB)** and **Immediate Brane (IB)** are the two context
layers for name resolution:

- **IB**: Current context accumulated so far (lines before current expression)
- **AB**: Parent brane context containing the defining expression and its AB/IB

When a brane is referenced by name, it is **detached** from its original AB/IB
and **recoordinated** with the new context. Previously failed name searches can
now resolve.
```

**Testing** — remove all UBCb snapshot test references. Update snapshot commands
to use `foolish-core` only:

```bash
cargo test -p foolish-core --lib
cargo test -p foolish-core --lib -- approval_all
```

**Cross-validation** — remove references to Java/Scala cross-validation. If
Rust-to-Rust cross-validation exists between foolish-core and foolish-ubcb,
remove it (foolish-ubcb is deleted).

#### 4b. README.md — Full Rewrite

Remove:
- Java/Scala badge lines
- "Quick Start (Java/Scala)" section
- All Maven commands
- References to Java 25, Scala 3.3.7, ANTLR 4.13.2
- Versioned Documentation table (ubc0_1 only existed for Scala MVP)
- "Version Overview" section with ubc0/ubc0_1/ubc1 descriptions

Replace "Running the Rust Implementation" with:

```markdown
## Quick Start

```bash
cargo build --package foolish-cli --release
cargo run --package foolish-cli -- run path/to/program.foo
cargo run --package foolish-cli -- step path/to/program.foo
cargo run --package foolish-cli -- repl
cargo test --workspace
```
```

Update badge URLs to reference `jia` branch (after rename).

#### 4c. rust_instructions.md

Update any path references from `foolish/` prefix to root-relative paths.

#### 4d. docs/DOC_AGENTS.md

Update path references. Remove any JVM or multi-crate references.

#### 4e. docs/styleguide.md

Update path references if any exist.

#### 4f. docs/foop/ Files

Search all `FOOP-*.md` and `FOOP-*.plan.md` files for references to:
- `foolish/` as a directory prefix
- `foolish-web`, `foolish-ubcb`, `foolish-ubcb-cli`
- Java, Scala, Maven
- `cd foolish`

Update or add notes indicating these are historical references.

#### 4g. docs/ubc1/ and docs/ubc0_1/

Update engineering docs to remove references to dead crates. Keep historical
content in `docs/vintage_legacy/` as-is.

### 5. Rename Main Branch to jia

#### 5a. GitHub Steps (Manual — Repository Admin Required)

```bash
# Option 1: GitHub Web UI
# Go to: https://github.com/frcusaca/foolish/settings/branches
# Under "Default branch", click pencil icon, rename "main" to "jia"

# Option 2: gh CLI
gh api repos/frcusaca/foolish/branches/main/rename -f new_name=jia
```

#### 5b. Local Clone Updates (All Contributors)

```bash
git branch -m main jia
git fetch origin
git branch -u origin/jia jia
git remote set-head origin -a
```

#### 5c. Update Branch References in Files

After rename, update:
- `.github/workflows/tests.yml` — branch trigger filters
- `README.md` — badge URLs (`?branch=main` → `?branch=jia`)
- `AGENTS.md` — any references to `main` as the primary branch
- `docs/foop/` — any references to merging to `main`

#### 5d. Branch Protection

Transfer any branch protection rules from `main` to `jia`:
- Go to: https://github.com/frcusaca/foolish/settings/branches
- Add protection rule for `jia` matching previous `main` rules

## FIR Impact

None. Repository structure cleanup only.

## UBC Step Impact

None. No evaluator changes.

## Test Plan

After all changes:
1. `cargo build --workspace` from repo root — succeeds
2. `cargo test --workspace` — passes
3. `cargo clippy --workspace` — clean
4. `cargo fmt --check` — clean
5. `grep -r "foolish/" --include="*.md"` — no matches except historical
   references in `docs/vintage_legacy/`
6. `grep -r "java\|scala\|maven\|pom\.xml" --include="*.md"` — no matches
   outside `docs/vintage_legacy/`
7. `grep -r "foolish-web\|foolish-ubcb" --include="*.md"` — no matches
8. `ls foolish/` — directory does not exist
9. GitHub shows `jia` as default branch
10. CI workflows trigger on `jia`

## Rejected Alternatives

### A. Keep Dead Crates in Archive Branch

The code is in git history. An archive branch creates maintenance burden and
signals the code might return. Clean deletion is clearer.

### B. Keep `foolish/` Subdirectory

Would avoid a large diff but perpetuate a non-standard layout. Every new
contributor learns the `cd foolish` convention. Every doc snippet carries the
prefix. Cost compounds.

### C. Use `crates/` Subdirectory

Common in large Rust projects (e.g., `rust-lang/rust`). Foolish is not that
large — three crates at root is clear and idiomatic.

### D. Keep `main` as Branch Name

Renaming during cleanup (when contributors expect disruption) is the ideal
time. `jia` is short, meaningful, distinctive.

## Open Questions

- Should `docs/ubc0_1/` be preserved or merged into `docs/vintage_legacy/`?
  (ubc0_1 existed partly for the Scala MVP; with JVM gone, its value is
  historical only.)
- Should the GitHub repository name change from `frcusaca/foolish` to
  something else? (Current spec assumes keeping it.)

## References

- Rust workspace docs: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
- GitHub branch rename: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-branches-in-your-repository/renaming-a-branch

## Last Updated

**Date**: 2026-07-02
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Executed Phases 1-5 and Phase 6a (merge to `alpha`, commit
e55ddab0) per `FOOP-03.plan.md`. `foolish-ubca` was kept as a fourth
surviving crate throughout (this spec's original crate table only named
three because it predated awareness of `foolish-ubca`'s in-progress
FOOP-62 work — establishing it as the sole reference implementation and
retiring `foolish-core` remains blocked on FOOP-62's unresolved
human-gated retirement question). Status moved Draft → Implementing.
Phase 6b (branch rename to `jia`) is deliberately deferred as a separate,
explicitly-authorized action — not done as part of this update. See
`FOOP-03.plan.md` for the full phase-by-phase execution record.

**Date**: 2026-07-01
**Updated By**: Sisyphus / mimo-v2.5-pro
**Changes**: Initial draft.
