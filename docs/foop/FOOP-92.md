---
foop: 29
title: Einmo — directory-based signed-snapshot testing with staged promotion
author: Sisyphus <agent>
status: Draft
type: Standards
created: 2026-06-26
phase: meta
supersedes: []
---

# FOOP-92: Einmo — directory-based signed-snapshot testing with staged promotion
FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

This FOOP specifies **Einmo**, a directory-based signed-snapshot testing
library that replaces insta for the Foolish project. Einmo's defining
requirement — the feature no surveyed snapshot/approval-testing framework
provides — is that **promotion is a staged pipeline** (output → checked →
verified), each stage is a directory, and **every generated output is
timestamped and signed by the test runner**. Promotion between stages is a
separate, cryptographically attributable act performed via CLI (by human or
AI agent). The library supports **comparing any two stages** so that
different processes can enforce different correspondences (e.g. "code output
matches checked" or "checked matches verified").

The body is organised as: Abstract → Motivation → Specification → standard
FOOP sections → Appendices A–E carrying the supporting research.

## Abstract

**Einmo** is a workspace crate (`einmo`) providing directory-based
signed-snapshot testing with a three-stage promotion pipeline: **output** →
**checked** → **verified**. Each test is configured with a work directory
containing `input/` (test triggers) and stage directories (`output/`,
`checked/`, `verified/`) holding `.einmo` files. Generated outputs are
timestamped and signed by the test runner. Promotion from output → checked
is a CLI operation available to AI agents or humans. Promotion from checked
→ verified requires a human-specified signing phrase. The library supports
comparing any two stages, enabling processes like "output matches checked"
or "checked matches verified." The crate replaces insta for Foolish
snapshot testing and is designed for reuse by other projects.

## Motivation

### What we (the FOOP project) want

The Foolish language implementation uses snapshot/approval tests as the
**hard acceptance gate** for VM behaviour (FOOP-62): a `.foo` input is
evaluated, the humanizing-sequencer output is captured, and an `.einmo` file
is the approved behavioural contract. Because the project is developed
collaboratively by humans **and AI agents**, we need:

1. **Staged promotion, not a single accept step.** Insta collapses "human
   looked at the diff" and "human promoted the baseline" into one `accept`
   step. We want three stages: output (generated), checked (reviewed),
   verified (human-signed). Each stage is a directory, making the pipeline
   inspectable by CI and by other tools.

2. **Every output is timestamped and signed.** The test runner always signs
   its output with a computer/AI key, embedding the generation timestamp.
   This attestation ("a machine produced this output at this time") is
   permanent and survives promotion.

3. **Compare any two stages.** Different processes can enforce different
   correspondences: "output matches checked" (CI gate), "checked matches
   verified" (release gate), or "output matches verified" (direct promotion
   path). The comparison API is stage-agnostic.

4. **CLI-driven promotion.** Promotion is a command-line operation that can
   be invoked by AI agents (output → checked) or humans (checked → verified,
   requiring a signing phrase). The CLI determines where to get the signing
   key — from keyboard console, from an external API, or from test code.

### The world after this FOOP

- A reusable crate `einmo` with directory-based test configuration.
- Each test's work directory: `input/` → `output/*.einmo` → `checked/*.einmo`
  → `verified/*.einmo`.
- Generated outputs carry a `test` signer entry with timestamp.
- `einmo promote output→checked` (AI/human, no passphrase required).
- `einmo promote checked→verified` (human, passphrase required).
- `einmo compare <stage-a> <stage-b>` — stage-agnostic comparison.
- CI gates on stage correspondence, not just existence.

## Specification

### 1. Crate structure

The library is a workspace member **`einmo`** (path `foolish/einmo/`),
depended on by `foolish-core` and `foolish-ubca` as a dev-dependency.

```
foolish/einmo/
├── Cargo.toml
└── src/
    ├── lib.rs              # re-exports
    ├── config.rs           # TestConfig (work dir, stage dirs)
    ├── stage.rs            # Stage enum + directory operations
    ├── compare.rs          # stage-to-stage comparison
    ├── signature.rs        # moved from foolish-core/src/signature.rs
    ├── format.rs           # .einmo file parse/serialize
    └── bin/
        ├── einmo.rs        # CLI: promote, compare, verify
        └── verify_signatures.rs  # moved from foolish-core/src/bin/
```

### 2. Directory-based test configuration

Each test is configured with a **work directory** containing:

```
test_name/
├── input/                  # test trigger files (*.foo or inline)
│   └── program.foo
├── output/                 # generated outputs (test-runner signed)
│   └── program.foo.einmo
├── checked/                # reviewed outputs (AI or human promoted)
│   └── program.foo.einmo
└── verified/               # human-signed outputs (passphrase required)
    └── program.foo.einmo
```

**Configuration:**

```rust
pub struct TestConfig {
    /// Root work directory for this test suite.
    pub work_dir: PathBuf,
    /// Input directory name (default: "input").
    pub input_dir: String,
    /// Stage directory names.
    pub stages: StageDirs,
}

pub struct StageDirs {
    pub output: String,    // default: "output"
    pub checked: String,   // default: "checked"
    pub verified: String,  // default: "verified"
}
```

### 3. The three-stage lifecycle

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Generated by test runner. Timestamped + computer-signed.
    Output,
    /// Reviewed — promoted from output by AI agent or human.
    /// No passphrase required; the CLI actor is recorded.
    Checked,
    /// Verified — promoted from checked by human with signing phrase.
    /// Passphrase required; human key appended.
    Verified,
}
```

**Stage → directory mapping:**

| Stage | Directory | Signer entries | Who can produce | Passphrase |
|---|---|---|---|---|
| Output | `output/` | `test` (computer key + timestamp) | test runner | none (empty) |
| Checked | `checked/` | `test` (preserved) | AI agent or human via CLI | none |
| Verified | `verified/` | `test` (preserved) + `util` (human key) | human via CLI | required |

**Critical invariant:** the `test` entry (with timestamp) is created once at
generation and **never modified or removed** by any promotion. Only
verification appends a `util` entry.

### 4. Every output is timestamped and signed

The test runner always embeds a generation timestamp in the `.einmo` file's
COMMENTS block and signs the entire output:

```
COMMENTS:
```markdown
test_name
generated: 2026-07-01T15:30:45.123456789Z
```
SIGNATURES:
  * Signed by test: <hex_pk_computer>
    * Foolish: <sig over canon_input>
    * HFS: <sig over canon_input + canon_hs>
    * Comments: <sig over canon_input + canon_hs + canon_comments>
```

The timestamp is **inside the signed content** — it is tamper-evident. The
`test` signer entry uses the empty-passphrase computer key (configurable via
cascade, see §7).

### 5. Promotion CLI

```bash
# Promote output → checked (AI agent or human, no passphrase)
einmo promote output→checked <work_dir> [--filter <pattern>]

# Promote checked → verified (human, passphrase required)
einmo promote checked→verified <work_dir> [--stdin-passphrase]

# Promote output → verified directly (skipping checked, human only)
einmo promote output→verified <work_dir> [--stdin-passphrase]
```

The CLI determines the signing key source:
- **Keyboard console** (default for `--stdin-passphrase`): reads passphrase
  from terminal.
- **External API** (configurable): calls a key-service endpoint.
- **Test code** (for automated re-signing): uses the cascade in §7.

### 6. Stage comparison

The library supports comparing any two stages, enabling different processes
to enforce different correspondences:

```rust
pub struct ComparisonResult {
    /// Files present in stage_a but missing in stage_b.
    pub only_in_a: Vec<PathBuf>,
    /// Files present in stage_b but missing in stage_a.
    pub only_in_b: Vec<PathBuf>,
    /// Files present in both but with different content.
    pub differing: Vec<DiffEntry>,
    /// Files present in both with identical content.
    pub matching: Vec<PathBuf>,
}

pub struct DiffEntry {
    pub path: PathBuf,
    pub stage_a_content: String,
    pub stage_b_content: String,
}
```

```bash
# Compare output with checked (CI gate: "did we review everything?")
einmo compare output checked <work_dir>

# Compare checked with verified (release gate: "did we verify everything?")
einmo compare checked verified <work_dir>

# Compare output with verified (direct path)
einmo compare output verified <work_dir>
```

**Use cases:**
- **CI gate:** `einmo compare output checked` must show 0 differing files
  (all reviewed outputs match generated outputs).
- **Release gate:** `einmo compare checked verified` must show 0 differing
  files (all checked outputs have been verified).
- **Staleness check:** `einmo compare output checked` with `--stale-days 14`
  warns about checked files older than 14 days relative to output.

### 7. Passphrase resolution cascade

Same as the original FOOP-92 §9, applied to `einmo`:

1. `--passphrase <value>` or `--stdin-passphrase`
2. `EINMO_PASSPHRASE` env var
3. `einmo.toml` config file, `[signing]` section
4. Default `""` (computer key)

Promotion to `verified` always requires `--stdin-passphrase` (interactive
human act). Promotion to `checked` does not require a passphrase.

### 8. Library API

```rust
/// Run all tests in a work directory, generating outputs.
pub fn run_tests(config: &TestConfig, evaluator: &dyn Evaluator) -> TestResults;

/// Promote files from one stage to the next.
pub fn promote(config: &TestConfig, from: Stage, to: Stage, key_source: &KeySource) -> Result<PromotionReport>;

/// Compare two stages.
pub fn compare(config: &TestConfig, stage_a: Stage, stage_b: Stage) -> ComparisonResult;

/// Verify all signatures in a stage.
pub fn verify(config: &TestConfig, stage: Stage) -> VerificationReport;
```

### 9. CI integration

**Layer 1 (always on):** `einmo verify <work_dir>` — all `.einmo` files
must pass signature verification.

**Layer 2 (opt-in):** `einmo compare output checked <work_dir>` must show
0 differing files — all generated outputs must have been reviewed.

**Layer 3 (opt-in):** `einmo compare checked verified <work_dir>` must show
0 differing files — all checked outputs must have been verified (release
gate).

## Specification

### 1. Library location and crate structure

The library is extracted as a workspace member **`foolish-snap`** (path
`foolish/foolish-snap/`), depended on by `foolish-core` and `foolish-ubcb`
as a dev-dependency. Rationale: the signed-snapshot pattern is general
(FVM-output-independent) and deserves its own crate so it can later be
published or reused by non-Foolish projects. The crate re-exports the
existing `signature.rs` types (moved here from `foolish-core`) and the
`SnapshotSuite` orchestrator (moved here from `foolish-core` per FOOP-02,
generalised over an `Evaluator` trait that stays in `foolish-core`).

```
foolish/foolish-snap/
├── Cargo.toml
└── src/
    ├── lib.rs              # re-exports
    ├── lifecycle.rs        # SnapshotLifecycle state machine (NEW)
    ├── signature.rs        # moved from foolish-core/src/signature.rs
    ├── snapshot_suite.rs   # moved from foolish-core/src/snapshot_suite.rs
    ├── format.rs           # .snap file parse/serialize (the signed envelope)
    └── bin/
        └── verify_signatures.rs  # moved from foolish-core/src/bin/
```

`foolish-core` re-exports the public API (`SignedSnapshot`,
`SnapshotLifecycle`, `SignerEntry`, `SnapshotSuite`, `Evaluator`) for
backward compatibility, so existing test modules (`ubc_snapshot_tester.rs`,
`ubcb_snapshot_tester.rs`) compile unchanged.

### 2. The four-state lifecycle (the core requirement)

```rust
/// The lifecycle state of a snapshot artifact.
///
/// Generated  → the test ran, output differs or is new, computer signed.
/// Reviewed   → a human inspected and marked acceptable; NOT yet promoted.
/// Flagged    → a human inspected and found an issue; deferred for AI/agent.
/// Promoted   → a human passphrase signature appended; renamed to .snap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotLifecycle {
    Generated,
    Reviewed,
    Flagged,
    Promoted,
}
```

**State → on-disk extension mapping:**

| State | Extension | Signer entries present | Who can produce |
|---|---|---|---|
| Generated | `.snap.new` | `test` (computer key) | test runner (AI or human) |
| Reviewed | `.snap.new.approved` | `test` (computer key) — *unchanged* | human (review tool) |
| Flagged | `.snap.new.check` | `test` (computer key) — *unchanged*, plus `@agent` annotation log | human (review tool) |
| Promoted | `.snap` | `test` (computer key) **+ `util`** (human passphrase key, with "Entire file" sig) | human (promote tool) |

**Critical invariant:** the `test` entry is created once at generation and
**never modified or removed** by any subsequent transition. Review and
Flagged are pure file-rename + annotation operations — they do not touch
signatures. Only Promotion appends a signature.

### 3. Signature model — append, never replace (adopts FOOP-22)

The library adopts FOOP-22's multi-signer append format as the canonical
model (superseding the current replace-on-`--write-verified` behaviour):

```
SIGNATURES:
  * Signed by test: <hex_pk_computer>
    * Foolish: <sig over canon_input>
    * HFS: <sig over canon_input + canon_hs>
    * Comments: <sig over canon_input + canon_hs + canon_comments>
  * Signed by util: <hex_pk_human>
    * Entire file: <sig over all file bytes before this entry>
    * Foolish: <sig over canon_input>
    * HFS: <sig over canon_input + canon_hs>
    * Comments: <sig over canon_input + canon_hs + canon_comments>
```

- The `test` entry is the **generator's permanent attestation** ("a machine
  produced this output"). It satisfies the requirement *"generator of the
  snapshot always signs the content."*
- The `util` entry is the **human promotion attestation** ("a specific human
  reviewed and promoted this baseline"). Its `Entire file` signature covers
  every byte before it — including the `test` entry — so any tampering with
  prior content invalidates the human attestation.
- Multiple `util` entries may accumulate (second reviewer, re-promotion
  after a re-approval). Each subsequent `util`'s `Entire file` covers all
  prior entries, forming a tamper-evident chain.

**This is a behaviour change from the current `accept_approved.sh`**
(which replaces). The migration is automatic: `verify_signatures
--write-verified` switches from replace to append. Existing single-signer
`.snap` files (legacy flat footer) are parsed as a single `test` entry and
re-emitted in the indented format on first re-signing.

### 4. Library API

```rust
// foolish-snap/src/lifecycle.rs

/// A snapshot artifact with its signer history.
pub struct SignedSnapshot {
    /// The non-signature body: insta frontmatter + INPUT/RESULT/COMMENTS blocks.
    pub body: SnapshotBody,
    /// Ordered signer entries (test first, zero or more util after).
    pub signers: Vec<SignerEntry>,
}

impl SignedSnapshot {
    /// Parse a `.snap` / `.snap.new` / `.snap.new.approved` / `.snap.new.check` file.
    /// Handles both legacy flat footer and new indented multi-signer format.
    pub fn from_file(path: &Path) -> Result<Self, SnapError>;

    /// The lifecycle state, derived from the file extension and signer list.
    pub fn lifecycle(&self, path: &Path) -> SnapshotLifecycle;

    /// True iff a `util` (human) signer entry is present.
    pub fn is_promoted(&self) -> bool;

    /// True iff the `test` (computer) entry verifies under the empty passphrase.
    pub fn computer_signature_ok(&self) -> bool;

    /// Verify every signer entry. Returns per-entry results. Refuses to
    /// return Promoted if any signature fails.
    pub fn verify_all(&self) -> Vec<SignerVerification>;
}

/// State-machine transitions. Each consumes `self` and returns the next state's
/// artifact, enforcing the legal transitions at the type level.
pub trait SnapshotLifecycleOps {
    /// Generated → Reviewed. Pure rename (.snap.new → .snap.new.approved).
    /// Does NOT touch signatures. Performed by the review tool after a human
    /// marks the diff acceptable.
    fn review(self) -> Result<ReviewedSnapshot, SnapError>;

    /// Generated → Flagged. Appends an `@agent` annotation to the .check log
    /// and removes the .snap.new. Does NOT touch signatures.
    fn flag(self, comment: &str) -> Result<FlaggedSnapshot, SnapError>;

    /// Reviewed → Promoted. Appends a `util` signer entry signed under the
    /// given passphrase (with "Entire file" integrity), then renames
    /// .snap.new.approved → .snap. THIS IS THE ONLY SIGNATURE-ADDING TRANSITION.
    fn promote(self, human_passphrase: &str) -> Result<PromotedSnapshot, SnapError>;
}
```

The `Evaluator` trait is **generalised** so the library has no dependency on
Foolish FIR types — it returns `Vec<String>` (formatted output blocks, one
per result), not `Vec<FirRef>`. Foolish's `UbcEvaluator`/`UbcbEvaluator`
adapters format FIRs to strings internally (via `FirSequencer::format`)
before returning them. This makes the library usable by any project, not
just Foolish.

```rust
/// General-purpose evaluator: takes an input source string, returns the
/// formatted output block(s) to be signed and stored in the .snap file.
/// The library does NOT depend on Foolish FIR types.
pub trait Evaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String>;
}
```

The library provides **two entry points** — file-based inputs (Foolish's
`.foo`-file model) and inlined inputs (general-purpose, for systems without
input files on disk):

```rust
impl SnapshotSuite {
    /// File-based input (Foolish's use case): discovers .foo files in
    /// input_dir, evaluates each, asserts against approved/ via insta.
    pub fn evaluate(&self, path: &Path, evaluator: &dyn Evaluator) -> Result<String, String>;

    /// Inlined input (general-purpose): the input is a string passed in
    /// code, not a file. The library still produces a signed .snap file —
    /// the inlined input is captured into the INPUT: block and signed.
    /// `name` becomes the snapshot filename (<name>.<ext>.snap).
    pub fn evaluate_inline(&self, name: &str, input: &str, evaluator: &dyn Evaluator) -> Result<String, String>;
}
```

**Inline expected values are refused:** the library's contract is "every
snapshot's expected result is a signed `.snap` file." An inline
`assert_snapshot!(…, @"…")` expected value cannot carry a `SIGNATURES:`
footer and therefore cannot be content-protected; the library emits a CI
failure if it detects inline `@"..."` expected values in modules under
`SnapshotSuite` (a simple grep-based test). Inputs may be inlined (above);
expected results may not.

### 5. Generation always signs (the invariant enforced in code)

`SnapshotSuite::evaluate` (moved to `foolish-snap/src/snapshot_suite.rs`)
constructs the `SignedSnapshot` with **exactly one `test` signer entry**
before the string reaches `insta::assert_snapshot!`. The computer-key
signature is therefore embedded in the bytes insta writes to `.snap.new`.
This is the existing behaviour (FOOP-12) and this FOOP makes it a **typed
invariant**: `SignedSnapshot` cannot be constructed without at least a `test`
entry; `review`/`flag`/`promote` are the only legal transitions and none of
them removes the `test` entry.

```rust
impl SnapshotSuite {
    pub fn evaluate(&self, path: &Path, evaluator: &dyn Evaluator) -> Result<String, String> {
        // ... (existing INPUT/RESULT/COMMENTS assembly, unchanged) ...
        let sig = signature::sign_snapshot(
            "",                       // empty passphrase = computer/AI key (ALWAYS)
            &source,
            &hs_outputs,
            &comments,
        );
        // sig is a SignerEntry with role="test", no "Entire file" field.
        // It is embedded in the asserted string; insta writes it to .snap.new.
        Ok(format!("{}\n{}", body, sig.format_as_test_entry()))
    }
}
```

### 6. Review and promotion tools (formalising the shell scripts)

The worktree-local `foolish_review.sh` and `accept_approved.sh` become thin
wrappers over library calls. Their behaviour is preserved (vimdiff,
`@agent` flagging → append-to-`.check`-log + delete `.snap.new`, `@agent,
skip` defer, 10% reread sampling, `diff -I` ignoring signature lines) but
the state logic moves into the library so it is testable and not bash-dependent.

- **`foolish_review.sh`** → calls `SignedSnapshot::review()` /
  `flag()` after the human closes vimdiff. The `diff -I` signature-ignoring
  and the 10% reread sampling remain in the script (they are presentation
  concerns, not lifecycle concerns).
- **`accept_approved.sh`** → calls `SignedSnapshot::promote(passphrase)` for
  each `.snap.new.approved`. The single-passphrase-prompt-for-the-batch
  behaviour is preserved. The hardcoded `target/debug/verify_signatures` path
  is replaced with `cargo run -p foolish-snap --bin verify_signatures --`.

### 7. `verify_signatures` binary (moved, behaviour extended)

Moved to `foolish-snap/src/bin/verify_signatures.rs`. The CLI gains a
`--promote` mode (wraps `SignedSnapshot::promote`) distinct from
`--write-verified` (which becomes append-only per §3). Existing flags
(`--stdin-passphrase`, `--add-comment`, directory scanning of `.snap` and
`.snap.new`) are preserved.

### 8. Trust model (restated for clarity)

- **Computer/AI key** = `derive_keypair(<passphrase>)` where the passphrase
  is resolved via the cascade in §9 (default `""` — the canonical computer
  key). Ungated; anyone (human or agent) may produce it. Carries no human
  attestation. Used for `test` entries at generation and for any automated
  (non-interactive) re-sign. Configurable so other projects can set a
  non-empty automated key.
- **Human key** = `derive_keypair("<human_passphrase>")` where the passphrase
  is supplied interactively via `--stdin-passphrase` (the human types it, or
  it is piped). A `util` entry under a human key is the human-attestation
  that this FOOP makes first-class. The human passphrase is NOT drawn from
  the config-file/env tier of the cascade by default — promotion is an
  explicit interactive human act (see §9).
- AI agents are policy-barred from `promote`, from `--stdin-passphrase`
  signing, from `cargo insta accept`/`INSTA_UPDATE=always`, and from
  moving/deleting `.snap` or `.snap.new.approved`. The library does not
  enforce actor identity (it cannot); the gate is policy + the human-only
  interactivity of the passphrase prompt, with the signature providing
  post-hoc attribution.

### 9. Passphrase resolution cascade

Automated (non-interactive) signing — the `test` entry at generation, and any
`verify_signatures --write-verified` run that is NOT an explicit human
promotion — resolves the passphrase from a **fixed precedence, highest to
lowest**:

1. **Command-line parameters** — `--passphrase <value>` (non-interactive,
   for CI/scripted use) or `--stdin-passphrase` (reads one line from stdin;
   may be a terminal prompt or a pipe).
2. **Environment variables** — `FOOLISH_SNAP_PASSPHRASE`.
3. **Configuration files** — `.config/foolish-snap.toml` (or
   `foolish-snap.toml` in the workspace root), section `[signing]`, key
   `passphrase = "..."`.
4. **Default** — `""` (empty string) = the canonical computer/AI key
   (public key `dc5f586c…b683`). This preserves the current FOOP-12
   behaviour when nothing is configured.

The first tier that yields a non-`None` value wins; lower tiers are not
consulted. An explicit empty string (`--passphrase ""` or
`FOOLISH_SNAP_PASSPHRASE=""`) is treated as "set to empty" (the computer
key), NOT as "unset" — to unset, omit the flag/var entirely.

**Human promotion (`promote()`) is a distinct path:** it requires
`--stdin-passphrase` and reads the passphrase interactively. It is not
satisfied by the env-var or config-file tiers alone — promotion is an
explicit human act, not an automated one. (If a CI pipeline needs to perform
promotions non-interactively, it may pipe the passphrase into
`--stdin-passphrase`; the library does not judge whether the stdin source is
a human or a pipe. The signature still attributes the promotion to whatever
passphrase was used.)

**Why a cascade:** the Foolish project's computer key is the empty
passphrase, but the library must be reusable by other projects (another
project will need this imminently) that may want a non-empty automated key,
or may need to inject a CI-specific key via environment without touching
code. The cascade makes the key configurable without hardcoding.

### 10. CI integration — two-layer gate

The library's `verify_signatures` binary supports a two-layer CI gate with
distinct scope:

**Layer 1 (always on, non-negotiable) — integrity verification:**
`verify_signatures <dir>` (no special flags) verifies every `.snap` and
`.snap.new` in the directory: all `test`/`util` signer entries must validate,
and the snapshot content must match the canonicalised signed content. Any
failure (tampered signature, broken chain, mismatched content) exits
non-zero. This runs on **every branch** in CI — it catches corruption and
regressions regardless of promotion state. A failing Layer-1 check blocks
everything: no merge, no tag.

**Layer 2 (opt-in `require_human_promotion`, P0/tier-1 feature, default
OFF) — promotion enforcement:** `verify_signatures --require-human-promotion
<dir>` additionally fails if any `.snap` (committed golden) lacks a `util`
(human) signer entry — i.e. a snapshot that was generated (computer-signed)
but never promoted by a human. Enabled per-repository via config
(`[ci] require_human_promotion = true` in `foolish-snap.toml`) or the
`FOOLISH_SNAP_REQUIRE_HUMAN_PROMOTION=1` env var.

**Granularity (critical):** Layer 2 is a **merge-to-main and tag gate**,
NOT a commit/push gate. A computer-only `.snap` (no `util` entry) is:
- ✅ **Allowed** to be committed to a working/feature branch.
- ✅ **Allowed** to be pushed.
- ✅ **Allowed** in CI runs on the feature branch (Layer 1 still applies —
  it must verify).
- ❌ **Blocked** from merging into `main` via PR (when Layer 2 is enabled on
  the target repo).
- ❌ **Blocked** from being included in a release tag.

This lets developers and AI agents commit work-in-progress (computer-signed)
snapshots to their branches and push them for collaboration and CI, while the
human promotion (appending a `util` entry via `promote()` during PR review)
happens before merge. The gate is enforced at branch-protection (required
status check on `main`) and at the tagging step, not at `git commit` or
`git push`.

**Implementation surface:** the library exposes `SignedSnapshot::is_promoted()`
(returns true iff a `util` entry is present). CI wires this into a GitHub
Actions required-status-check on `main` and a pre-tag verification step. The
library does not itself enforce branch protection — that is the platform's
role (GitHub branch protection rules / `gh` tag protection); the library
provides the exit code that the platform check consumes.

## FIR Impact

None. This FOOP changes test infrastructure only. The `SequenceableFir` /
`HumanizingSequencer` / `FirQueryable` machinery (FOOP-02, FOOP-42) is
unchanged; Einmo consumes its `String` output opaquely.

## UBC Step Impact

None. FOOP-62's `catch_unwind` panic-capture contract in the snapshot
harness is preserved — Einmo's test runner continues to catch panics and
write `PANIC: <msg>` into the output before signing, so a panicking
evaluation still produces a signed, reviewable `.einmo` in `output/`.

## Test Plan

### Unit tests (`einmo/src/stage.rs`)

- **Stage derivation**: `output/` → Output; `checked/` → Checked;
  `verified/` → Verified.
- **Promote output→checked**: file copied, `test` entry preserved, actor
  recorded in metadata.
- **Promote checked→verified**: `util` entry appended with human key;
  `test` entry unchanged.
- **Promote refuses without `test` entry** (the always-signed invariant).
- **Promote refuses if any existing signature fails** (chain integrity).

### Unit tests (`einmo/src/compare.rs`)

- **Compare identical stages**: all files matching, 0 differing.
- **Compare with missing files**: correct `only_in_a` / `only_in_b`.
- **Compare with content differences**: correct `differing` entries.
- **Compare with signature-ignored**: differences in signature lines are
  ignored when comparing content.

### Integration tests (`einmo` CLI)

- **End-to-end**: generate → promote output→checked → promote
  checked→verified → verify all stages.
- **CI gate simulation:** `einmo compare output checked` exits non-zero
  when differing files exist.
- **Passphrase cascade:** `test` entry signs with empty passphrase when no
  cascade tier is set; `EINMO_PASSPHRASE` env var works; config file works.
- **Promote to verified refuses without `--stdin-passphrase`.

## Rejected Alternatives

### A. Replace-on-promotion (current behaviour)

`verify_signatures --write-verified` currently overwrites the footer with the
human key, destroying the computer attestation. Rejected because it violates
the "generator always signs" requirement and provides no chain of attestation
— only the last signer remains, and there is no way to know a machine
produced the output. FOOP-22 already proposes append; this FOOP adopts it.

### B. Conflate review and promotion (insta/Verify/Jest model)

Use insta's native `cargo insta review` accept step as the only transition
(Generated → Promoted directly). Rejected because it eliminates the
Reviewed intermediate state, which is the explicit requirement: we want a
human to mark "acceptable" without immediately committing the human
signature, leaving room for batching, second review, or CI gating.

### C. External signature sidecar files (`.snap.sig`)

Store signatures in a separate `.snap.signer1.sig` file. Rejected because
it decouples the signature from the artifact (signer files get lost or
mismatched), complicates the workflow (manage N files), and breaks the
"snapshot file is self-describing" property that makes review tractable.
The in-file `SIGNATURES:` block keeps everything co-located.

### D. Fork insta upstream

Fork `mitsuhiko/insta` to add signing/review/promotion natively. Rejected
for now: insta's file-write path is not interceptable (hardcoded `fs::write`
in `Snapshot::save`), so a fork would diverge permanently and absorb ongoing
merge cost. The library approach wraps insta (the generator still calls
`insta::assert_snapshot!` with a pre-signed string) and adds the lifecycle
layer on top — no fork required. A fork remains a future option if the
lifecycle layer needs to intercept insta's write path (e.g. to refuse
writing a `.snap` without a `test` entry).

### E. Do nothing (leave it in shell scripts + AGENTS.md prose)

Rejected because the lifecycle is currently untyped, untestable, and
worktree-local (the scripts are not on the main branch). A typed library
makes the invariant ("generator always signs", "review ≠ promote")
enforceable in code and CI, not dependent on bash discipline.

## Resolved Decisions

- **OQ-1 (crate vs module): RESOLVED — crate.** The library is a new
  workspace member crate `foolish-snap` (path `foolish/foolish-snap/`).
  Rationale: another project will need this imminently, so the signed-
  snapshot library must be a reusable, publishable artifact, not a module
  buried in `foolish-core`.
- **OQ-2 (CI gate — two-layer model): RESOLVED.** The gate has two layers
  with distinct scope and granularity:
  - **Layer 1 (always on, non-negotiable):** every `.snap` must pass
    signature verification (all `test`/`util` entries valid) AND content
    match. A tampered, broken, or failing snapshot halts CI on **any**
    branch — no merging known regressions or bugs. Catches corruption and
    regressions.
  - **Layer 2 (opt-in `require_human_promotion`, P0/tier-1 feature,
    default OFF):** when a repository enables this config, a PR carrying a
    NEW or CHANGED `.snap` with only the `test` (computer) entry — no `util`
    (human) signature — is **blocked from merging into `main`** and
    **blocked from being tagged** (release tags). It does NOT block commits
    or pushes to working/feature branches — developers and AI agents must be
    free to commit work-in-progress (computer-signed) snaps to their branches
    and push them for collaboration and CI. The human promotion (appending a
    `util` entry) happens during PR review, before merge.
  - The **promotion transition** (`promote()`) always requires a human
    passphrase (§2/§4) — invariant, not config-gated. What Layer 2
    config-gates is whether CI *blocks merge/tag* of un-promoted snaps.
  - Layer 2 is **the primary improvement to insta** this FOOP makes — it
    must be implemented and available (tier-1/P0) — but its default is OFF
    so the existing legacy corpus and opt-out projects are not blocked.
- **OQ-3 (reviewed-state staleness): RESOLVED — warn-only, 14-day default,
  mtime-based, configurable.** The library emits a warning (not a CI
  failure) for `.snap.new.approved` files whose mtime is older than a
  configurable threshold (default 14 days). Threshold configurable via the
  same config file as the passphrase cascade. Deferred to a follow-up if it
  proves noisy.
- **OQ-4 (timestamps in `util` entries): RESOLVED — yes, as an unsigned
  comment line.** Each `util` (human) entry gains an advisory `# promoted:
  <ISO8601>` comment line above the entry. The timestamp is **outside** the
  signed content, so signature determinism is preserved for reproducible
  re-signing (a re-sign produces the same signature bytes). The timestamp is
  advisory — editable without invalidating the signature — and provides the
  audit hint that OQ-3's staleness check can also consult. (If signed
  timestamps are later required for stronger audit, that becomes a FOOP-22
  amendment.)
- **OQ-7 (plan file): RESOLVED — yes.** A `FOOP-92.plan.md` will be written.
  All blocking OQs are resolved; the design is frozen.
- **OQ-6 (inline snapshots): RESOLVED — expected results never inlined;
  inputs MAY be inlined.** The library's contract is "every snapshot's
  expected result is a signed `.snap` file" — an inline `@"..."` expected
  value cannot carry a `SIGNATURES:` footer, so it cannot have its content
  protected, so the library **refuses** inline expected values (a CI/test
  check fails on `assert_snapshot!(…, @"…")` in modules under `SnapshotSuite`).
  However, **inputs may be inlined**: not every system using this library
  has `.foo` input files on disk. The library provides an
  `evaluate_inline(name, input, evaluator)` entry point so a test can pass
  its input as a string in code; the library still produces a signed `.snap`
  file for the output (the inlined input is captured into the `INPUT:` block
  of the snapshot and signed along with the output). This makes the library
  general-purpose: it does not assume file-based inputs. The `Evaluator`
  trait is correspondingly generalised — it returns `Vec<String>` (formatted
  output blocks) rather than `Vec<FirRef>`, so the library has no dependency
  on Foolish FIR types; Foolish's `UbcEvaluator`/`UbcbEvaluator` adapters
  format FIRs to strings internally before returning them.

## Open Questions

- **Structured `.check` log (OQ-5, to be flushed out).** Whether `flag()`
  writes JSON-lines instead of the current free-text `$(date) cat $x` +
  content append. Affects AI-agent ingestion of flagged snapshots. Design
  options: JSON-lines `{timestamp, snapshot_path, agent_comment,
  flagged_content_digest}` vs free-text human-readable log. Deferred for
  further discussion — not blocking the initial implementation.
- **Plan file (OQ-7, resolved).** Yes — a `FOOP-92.plan.md` with checkboxed
  execution tasks will follow. All blocking OQs (1, 2, 6, 7) are resolved;
  the design is frozen. OQ-5 (structured `.check` log) is deferred and does
  not block the plan.

## References

- Prior FOOPs:
  - **FOOP-12** — Signature scheme (Ed25519/Argon2id, canonicalization,
    dual-signing, `verify_signatures`). The cryptographic foundation this
    library builds on. Status: Final.
  - **FOOP-22** — Multi-signer append format (`test`/`util` roles, "Entire
    file" integrity). This FOOP adopts its format as canonical. Status: Draft.
  - **FOOP-02** — `SnapshotSuite` current home in `foolish-core`, generalised
    over `Evaluator`. This FOOP moves it into `foolish-snap`. Status: Draft.
  - **FOOP-42** — Humanizing FIR Sequencer (HFS) output byte format
    (`hfssnap`). The signed body must conform. Status: Draft.
  - **FOOP-71** — cargo-insta adoption. The generator layer this library
    wraps. Status: Draft.
  - **FOOP-81** — Historical `SnapshotSuite`/`HumanizingSequencer` origin
    (superseded by FOOP-02). Status: Superseded.
  - **FOOP-62** — UBCa; snapshots are the hard acceptance gate; `catch_unwind`
    panic-capture contract the library must honour. Status: Brewing.
  - **FOOP-21** — Alarms emitted into snapshot output. Status: Brewing.
- External docs (full citation in Appendix D):
  - insta (v1.48.0) — https://insta.rs/docs/, https://docs.rs/insta
  - in-toto Attestation Framework — https://github.com/in-toto/attestation
  - sigstore-rs — https://github.com/sigstore/sigstore-rs
  - SLSA attestation model — https://slsa.dev/attestation-model
  - GitHub Artifact Attestations — https://github.com/actions/attest
  - jlevy/tbd Golden Sessions — https://github.com/jlevy/tbd/blob/main/packages/tbd/docs/guidelines/golden-testing-guidelines.md
  - insta issue #792 (TOFU/immutable snapshots, OPEN) — https://github.com/mitsuhiko/insta/issues/792
  - insta PR #815 (non-interactive review for LLMs/CI) — https://github.com/mitsuhiko/insta/pull/815
- Code locations (pre-extraction):
  - `foolish/foolish-core/src/signature.rs` (644 lines, FOOP-12)
  - `foolish/foolish-core/src/snapshot_suite.rs` (255 lines, FOOP-02)
  - `foolish/foolish-core/src/bin/verify_signatures.rs` (246 lines)
  - `foolish/foolish-core/src/ubc_snapshot_tester.rs`
  - `foolish/foolish-ubcb/src/ubcb_snapshot_tester.rs`
  - `foolish_review.sh`, `accept_approved.sh` (worktree `foop-62-ubca-mimo`)

---

# Appendix A — Insta Best Practices & Pitfalls

*(Distilled from official insta docs, source at commit `7f23d2e`, GitHub
issues, and community write-ups. Full source list in Appendix D.)*

## A.1 Recommended usage patterns

- **File snapshots, not inline, for anything signed or whitespace-sensitive.**
  Inline snapshots (`assert_snapshot!(v, @"...")`) embed the expected value in
  the `.rs` source; issue [#117](https://github.com/mitsuhiko/insta/issues/117)
  documents that leading/indented whitespace in inline snapshots is "basically
  intractable" and [#865](https://github.com/mitsuhiko/insta/issues/865) shows
  inline snapshots can enter an unfixable loop where `--accept` "fixes" but
  `cargo test` still fails. Signed `.snap` files must stay file-based. ✅ The
  Foolish project already does this.

- **Prefer structured (YAML/JSON) snapshots over raw `Debug` dumps.** Official
  quickstart: *"For most real world applications the recommendation is to use
  YAML snapshots of serializable values… they look best under version control
  and support redactions."* `Debug` output is not stable across Rust
  versions/field-reordering. For pre-formatted text output (the `hfssnap`
  blocks), `assert_snapshot!` is correct.

- **`cargo insta test` auto-switches to `--check` in CI.** This is automatic:
  when `is_ci()` is truthy and no explicit `--accept`/`--review`/`--force` is
  passed, insta forces `--check` → `INSTA_UPDATE=no`, no `INSTA_FORCE_PASS` →
  any mismatch fails the test and writes nothing. Add `cargo insta
  pending-snapshots` (now CI/LLM-friendly since 1.44, PR
  [#815](https://github.com/mitsuhiko/insta/pull/815)) as an explicit second
  gate.

- **`--unreferenced=reject` in CI** (default `auto` already rejects in CI) to
  catch orphaned `.snap` files from deleted tests. Complements
  `SnapshotSuite::get_missing_inputs`.

- **Compile insta/similar with `opt-level=3` in dev** for speed:
  ```toml
  [profile.dev.package.insta]
  opt-level = 3
  [profile.dev.package.similar]
  opt-level = 3
  ```

- **Add `.gitattributes` for `.snap` files** (insta ships none):
  ```
  *.snap linguist-language=YAML
  *.snap.new linguist-generated=true
  ```

## A.2 Pitfalls (specific GitHub issues)

| Issue | Problem | Mitigation |
|---|---|---|
| [#865](https://github.com/mitsuhiko/insta/issues/865) | Inline snapshot can loop: `--accept` "fixes" but `cargo test` still fails | File snapshots for whitespace-sensitive output |
| [#478](https://github.com/mitsuhiko/insta/issues/478) | `INSTA_UPDATE=always` exits 0 even on failure | Use `INSTA_UPDATE=no` in CI; never trust exit codes under `always` |
| [#117](https://github.com/mitsuhiko/insta/issues/117) | Leading whitespace in inline snapshots intractable | File snapshots |
| [#527](https://github.com/mitsuhiko/insta/issues/527) | `cargo insta test` shows no diff, only "1 to review" | Run `cargo insta review` |
| [#313](https://github.com/mitsuhiko/insta/issues/313) | Snapshots in loops/helper fns collide | `set_snapshot_suffix` per iteration |
| [#792](https://github.com/mitsuhiko/insta/issues/792) | TOFU/immutable snapshots — *feature request, OPEN, -0.1 from maintainer, no signing* | (relevant to this FOOP — see Appendix D) |
| [#659](https://github.com/mitsuhiko/insta/issues/659) | `--accept-unseen` pending deprecation | Don't build CI on it |

## A.3 Cross-cutting pitfalls (universal)

Over-snapshotting (brittle to cosmetic change); snapshotting unformatted
`Debug` that drifts; `HashSet`/`HashMap` non-deterministic order (use
`sorted_redaction`/`set_sort_maps`); **auto-accepting** (the cardinal sin —
every ecosystem independently concluded this "defeats the entire point");
snapshots masking semantic bugs (snapshot encodes a bug, then locks it in);
snapshot drift across teams; large diffs in code review; stale/orphaned
`.snap` files.

## A.4 Correction to AGENTS.md

**`Settings::bind_dynamic` does not exist.** Grep across the entire insta
source (commit `7f23d2e`) returns zero matches. The real binding APIs are
`bind(|| …)` (sync closure), `bind_async` (future), and `bind_to_scope()`
(RAII drop-guard, recommended). Settings are **thread-local** and `!Send`.
The "dynamic" redaction tool is `dynamic_redaction`. AGENTS.md should be
corrected.

## A.5 Insta internals — what's stable, what's not

- **Stable public API:** `Settings`, `Snapshot`, `MetaData`, `Comparator`,
  `DefaultComparator` (semver-guaranteed).
- **`internals` module** — explicitly *not* semver-stable; PR #640 has
  already removed methods from `SnapshotContents`. Avoid.
- **`_cargo_insta_support`** — feature-gated private contract between insta
  and cargo-insta. Off-limits.
- **The file-write path is NOT interceptable** — `Snapshot::save`/`save_new`
  hardcode `fs::write`. There is no hook between "passed comparison" and
  "bytes hit disk." This is why Foolish signs in `SnapshotSuite::evaluate`
  *before* the string reaches insta — the correct architecture given this
  constraint, and the reason this FOOP wraps rather than forks insta.

---

# Appendix B — Cross-Language Survey of Snapshot/Approval Testing

*(Distilled from research across JS/TS, Python, Rust, Ruby, Go, JVM, Swift,
and the testing-theory literature. Full source list in Appendix D.)*

## B.1 The genre — four names, one mechanism

- **Characterization test** (Michael Feathers, *Working Effectively with
  Legacy Code*, 2004) — documents *actual* behaviour of code you don't
  understand; "essentially a change detector."
- **Approval test** (Llewellyn Falco, ApprovalTests, ~2008) — defer the
  assertion; "I'll know it when I see it"; *approval* is an explicit human act.
- **Golden master** (record-industry "gold master" disc) — implies
  permanence; Emily Bache argues the term is misleading and should die
  ("pouring concrete on your software").
- **Snapshot test** (Jest, 2016) — implies *transience*; the JS-world default.

**The universal lifecycle:** `generate → review → approve → commit → (on
change) re-review`. The philosophical core: a machine can only verify "the
received output equals the approved file" — it can never verify "this output
is the behaviour we want." The genre *trades the oracle problem for a review
problem*.

## B.2 Cross-ecosystem tool map

| Ecosystem | Tool | Storage | Review→Promote model |
|---|---|---|---|
| **JS/TS** | Jest `toMatchSnapshot` | `__snapshots__/*.snap` | `-u` overwrites; interactive `i` mode. **1 state, no review gate.** |
| **JS/TS** | Vitest | `__snapshots__/*.snap` | `-u`; **refuses to write in CI**; obsolete snapshots *fail*. |
| **JS/TS** | Chromatic (Storybook) | cloud images | PR status check blocks merge until reviewed. |
| **Python** | Syrupy | `__snapshots__/*.ambr` | `--snapshot-update`; **fails if snapshot missing** (soundness). |
| **Python** | pytest-regtest | `_regtest_outputs/` | `--regtest-reset`; float tolerances; converter hooks. |
| **Rust** | insta | `.snap` / `.snap.new` | `cargo insta review` TUI; `--check` in CI. **2 states, review == promotion.** |
| **Rust** | expect-test | inline `expect!["…"]` | `UPDATE_EXPECT=1`. Minimalist. |
| **Rust** | trycmd | `.stdout`/`.stderr` | `TRYCMD=overwrite`; elision for volatile substrings. CLI-focused. |
| **Ruby** | ApprovalTests.Ruby | `.received`/`.approved` | `approvals verify --ask`. |
| **Go** | `google/golden`, cupaloy, go-snaps | `testdata/*.golden` | `-update` flag. cupaloy: updating *fails* the test to block CI auto-update. |
| **JVM** | ApprovalTests.Java | `.received`/`.approved` | Pluggable **Reporters** (diff tools, image diff, even audio). Scrubbers. |
| **Swift** | SnapshotTesting (Pointfree) | `.snap`/images | `record` mode; device-agnostic strategies. |

## B.3 Common anti-patterns (cross-ecosystem)

- **Snapshotting too much** (entire component trees) → huge undiffable
  snapshots; reviewers rubber-stamp.
- **Auto-accepting in CI** — every ecosystem that confronted this
  (ApprovalTests `AutoApprover`, Verify `AutoVerify=true`, Jest `-u` in CI)
  independently concluded it "defeats the entire point."
- **Snapshots masking semantic bugs** — snapshot creep + blind `-u` = a
  change is recorded, not caught.
- **Non-deterministic output** — "snapshot drift"; fix = redactions/scrubbers,
  not `setSystemTime`.
- **Large diffs in code review** — undiffable PRs; no size limit → nobody reads.
- **Stale/orphaned snapshots** — Vitest/Syrupy make unused snapshots *hard
  failures*; insta has `--unreferenced`.
- **Image-snapshot simulator mismatch** (Swift SnapshotTesting) — must
  compare on the same simulator/OS.

## B.4 Common best practices (cross-ecosystem)

1. **Redact/mask volatile values** — *every* mature ecosystem has a mechanism
   (Jest property matchers, ApprovalTests Scrubbers, insta redactions/filters,
   Verify default GUID/timestamp scrubbing, @emotion `classNameReplacer`).
2. **Structured (YAML/JSON) snapshots** for complex output, not raw debug dumps.
3. **Split large snapshots**; snapshot only what matters.
4. **CI gates rejecting pending/unreviewed snapshots** — Jest/Vitest no
   auto-write in CI; Vitest/Syrupy obsolete-snapshots fail; cupaloy updating
   fails the test; insta `INSTA_UPDATE=no` + `--unreferenced`.
5. **Pre-commit hooks & lint** (`no-large-snapshots` eslint rule, insta
   `.pre-commit-config.yaml`).
6. **A dedicated review tool, not just an env var** — `cargo insta review`,
   `dotnet verify`, `approvals verify --ask`, ApprovalTests Reporters,
   Chromatic dashboard. Bulk `-u`/`--update` is the *anti-pattern* form.
7. **Idempotency checks** (Go gofmt pattern) — after updating, re-run to
   confirm output is stable.
8. **Floating-point tolerances** (pytest-regtest `snapshot.check`).

## B.5 Lessons worth borrowing into Foolish-snap

| Source | Lesson |
|---|---|
| Go `testdata/` + `-update` | Minimalism validates a small insta surface. |
| `expect-test` | Inline `expect!` for small stable unit-test invariants. |
| `trycmd` | For CLI output snapshots (`foolish-cli run/step/repl`), use trycmd, not insta. |
| Syrupy / Vitest | Fail on missing AND unused/orphaned snapshots (wire `--unreferenced=reject` into CI). |
| @emotion/jest / Verify(C#) | Redact by default for known-volatile fields. |
| pytest-regtest | Tolerances for numeric output; platform-specific snapshot versions. |
| ApprovalTests.Java | The Reporter abstraction mindset — *how diffs are viewed* is first-class. |
| Chromatic | PR-status-check-as-gate — "CI rejects unsigned `.snap`." |

---

# Appendix C — Approval Testing Theory & Philosophy

*(Distilled from Feathers, Falco, Bache, Macrae, Beck, and the critique
literature. Full source list in Appendix D.)*

## C.1 The canonical lifecycle

`Write Test → Run → Fail (no .approved) → Diff Tool → Review → Approve →
Commit → (on change) Re-review`. Mechanical loop: `Run → Capture → Compare →
(Pass | Fail → Reporter)`. (ApprovalTests documentation mermaid flowchart.)

## C.2 "This matches" ≠ "this is correct"

The genre's defining feature and the root of every critique. A machine
verifies only byte-equality. Correctness is a human judgment relocated to
diff time. Wikipedia: characterization tests *"do not infer correctness of
the results. It merely helps detect unwanted effects of software changes."*
The genre **trades the oracle problem for a review problem**.

## C.3 When the genre shines

1. **Legacy code without tests** (characterization) — get coverage without
   understanding the code.
2. **Complex output hard to assert piecemeal** — multi-line text, nested JSON.
3. **UI / rendering output** — Jest's original motivation.
4. **Compiler / interpreter VM output** — *directly the Foolish use case.*
   Mochi MEP-0008 (5-layer golden harness), naga (40 golden files, SPIR-V
   disassembler for diff-friendliness), Pharo VM (hybrid methodology),
   `jfecher/golden-tests` (purpose-built for compilers).
5. **Serialization format stability / cross-validation.**

## C.4 When the genre harms

1. **Substituting for real assertions.**
2. **Non-deterministic output** — breaks repeatability (the defining
   requirement). Tian Pan (2026): for stochastic/LLM output, snapshots
   "actively lie."
3. **Snapshots too large to review.**
4. **Auto-accepting defeats the purpose** — the most universal critique.
   Peter Hrynkow: "snapshot fatigue → blind updating → tests are useless."
5. **Masks bugs** — first snapshot captured from *current* (possibly buggy)
   behaviour, then locks it in.
6. **Maintenance burden / coupling to structure** — Kent Beck scores snapshots
   poorly on "Structure-insensitive" (fire on implementation change, not
   behaviour change).

## C.5 The Jest controversy (2016→)

**Facebook's original guidance** (Jest 14, Jul 2016): snapshot testing pitched
as solving "keeping input and output in sync." **Community pushback:**
Ben McCormick (endorsed by Jest team) — snapshots are a *complement*, not a
replacement, and need a healthy code review process. Randy Coulman —
cognitive-load-at-worst-moment. Josh Ribakoff — snapshots assert on
implementation details. Ran Bar-Zik (Soluto) — "NOBODY does the careful
review." Tom Gold — stopped using Jest snapshots. Alex Vernacchia (2024) —
removed from production. Dermot Hughes (2025) — "bubble wrap: satisfying but
useless when overdone."

**Key distinction:** the controversy is almost entirely a *frontend/UI*
phenomenon. The compiler/VM community treats golden files as uncontroversial
— because compiler output is **deterministic, fully-specified, and reviewable
by domain experts**, neutralising the failure modes.

## C.6 Kent Beck — the most balanced establishment view

Beck scores snapshots against his 12 Test Desiderata: ★★★★★ on Behavioural
and Writable; ★ on Inspiring ("I wouldn't choose to rely only on snapshot
tests") and Structure-insensitive. Verdict: **measured endorsement** — a
legitimate point in test-design space, not a substitute for tests that make
you "think through the problem 2 independent ways."

---

# Appendix D — Novelty & Prior-Art Analysis

*(Full research with citations; supports the novelty claims in §Motivation.)*

## D.1 No mainstream framework signs test snapshots

Verified across: Jest, Vitest, Syrupy, ApprovalTests (JS/Python/Ruby/Java/Go),
Swift SnapshotTesting, cupaloy, go-snaps, expect-test, trycmd, pytest-regtest,
and the Mochi/naga/Pharo VM golden-file systems. **None cryptographically
signs snapshot files**, nor gates acceptance behind a human passphrase with an
audit trail distinguishing AI-generated from human-reviewed output.

The closest upstream recognition is insta issue
[#792](https://github.com/mitsuhiko/insta/issues/792) "TOFU/Immutable
snapshots" — **OPEN**, opened Aug 15 2025 by jalil-salame, only 2
participants, no labels/PR, max-sixty (collaborator) said "-0.1 without more
demand signals" and flagged it would "break the modularity of tests vs
commands." It proposes immutability after first generation — *not* signing or
attestation. Materially narrower than this FOOP.

## D.2 No Rust crate bridges snapshot testing + signing

Searches across crates.io, docs.rs, and GitHub for (snapshot/golden/approval)
+ (sign/attest/cryptographic) returned only pure snapshot libraries (insta,
expect-test, insta-cmd) and pure signing libraries (sigstore-rs,
in_toto_attestation). **No crate combines them.** The Foolish project's own
`verify_signatures` is the thing being specified — there is no off-the-shelf
prior art.

## D.3 No framework separates review from promotion

All surveyed frameworks conflate review and promotion into a single `accept`
step (insta `.snap`/`.snap.new` + `cargo insta review`; Verify `.received`/
`.verified`; ApprovalTests `.received`/`.approved`; Jest single `.snap` +
`-u`; jlevy/tbd `--update`). **This FOOP's 4-state model — Generated /
Reviewed / Flagged / Promoted — has no precedent.**

## D.4 Supply-chain attestation theory — the conceptual basis

The FOOP's design maps almost exactly onto **software supply-chain
attestation** theory, which is well-established:

- **SLSA (Supply-chain Levels for Software Artifacts)** — provenance as "an
  attestation that a particular build platform produced a set of software
  artifacts"; signing authenticates *who created the attestation*.
  ([slsa.dev/attestation-model](https://slsa.dev/attestation-model))
- **in-toto Attestation Framework** — Rust crate `in_toto_attestation` v0.1.0.
  Data model: **Statement** (binds Subject + Predicate) + **Envelope**
  (signed Statement = payload + Signature).
  ([github.com/in-toto/attestation](https://github.com/in-toto/attestation),
  [docs.rs/in_toto_attestation](https://docs.rs/in_toto_attestation))
- **sigstore-rs** — experimental; `KeyInterface` trait (generate, sign,
  verify, export/import PEM/DER). Known limitation: "does not handle
  verification of attestations yet." ([github.com/sigstore/sigstore-rs](https://github.com/sigstore/sigstore-rs))
- **GitHub Artifact Attestations** (`actions/attest`) — Sigstore-powered,
  wraps in-toto predicate, signs keyless via OIDC. Build-provenance
  granularity, not per-test-snapshot. ([github.com/actions/attest](https://github.com/actions/attest))

**What the Foolish project has done** is take the SLSA/in-toto attestation
model and apply it to a *test artifact*: the Ed25519 signature is an
attestation that *a specific human reviewer promoted this snapshot*, cryptographically
distinguishable from AI-generated output. This is, as far as the evidence
shows, **an original synthesis** — a legitimate cross-domain transfer of a
proven security pattern onto a testing genre whose entire failure mode is
review fatigue.

## D.5 Design delta from in-toto

in-toto's model is **append-many-signatures** (a Statement can carry multiple
independent Envelopes). FOOP-22 (adopted here) is also append — so the
`test` (computer) entry and `util` (human) entries accumulate, matching
in-toto. The novel element is the **typed lifecycle** (Generated → Reviewed
→ Promoted) that gates *which* append operations are legal at each state —
in-toto has no such lifecycle; it is a flat append-any-signer model. The
Foolish lifecycle makes "review without promotion" a first-class signed
state, which in-toto cannot express.

## D.6 Adjacent AI-provenance context (2024–2026)

No published work addresses "cryptographically distinguishing AI-generated
test baselines from human-reviewed ones." Adjacent:

- **insta PR #815** — non-interactive `review --snapshot` / `reject --snapshot`
  for non-TTY (LLMs, CI). insta adapting to AI agents as review actors — but
  with **no identity, no signing**. This FOOP fills exactly that gap.
- **DigiCert, "The New Trust Architecture for AI" (2026)** — cryptographic
  identity/authorization/integrity for AI agents/models/content.
- **CSA Agentic Trust Framework (Feb 2026)** — Zero-Trust governance for AI
  agents.
- **Alexander Zanfir, "Who Signed This? Provenance for AI Agents"** — chain
  proving human-approved rule → agent suggested → human reviewed. Conceptually
  the computer-key-then-human-key chain, but for agent action provenance, not
  test baselines.
- **C2PA** — provenance metadata for AI-generated media.

## D.7 jlevy/tbd "Golden Sessions" — closest spiritual prior art

The strongest expression of "golden testing designed *for* AI-assisted
development" ([github.com/jlevy/tbd](https://github.com/jlevy/tbd/blob/main/packages/tbd/docs/guidelines/golden-testing-guidelines.md)):

- **Golden session test** = capture complete execution trace as YAML, commit
  as golden reference. "Transparent box testing."
- **`MOCK_MODE` env var** — `mocked` (CI, stubbed, <100ms) vs `live` (real
  services, regenerating). Same test code both modes.
- **Stable/unstable field classification** — volatile values normalized at
  serialization time.
- **Agent workflow** — run → read structured failure → decide fix/update →
  review diffs → commit. "Session files are behavioural specifications.
  They deserve the same review rigour as code."

**What it does NOT have (this FOOP's delta):**
- **No cryptographic signing.** Review = `git diff` + human eyeballing.
- **No "reviewed but not promoted" state.** `--update` directly rewrites;
  review *is* promotion (conflated).
- **MOCK_MODE is about determinism, not attestation.**

**Citation value:** cite as the strongest "golden testing for AI-assisted
dev" prior art, then position this FOOP's signing/promotion-gate as the
**missing trust layer** on top of its MOCK_MODE/review-diff model.

## D.8 Verdict on the Foolish-rust approach

**Aligned with industry thinking — and in one respect, ahead of it.**

1. The use case (VM output snapshots) is the genre's canonical sweet spot.
2. The two-tier test architecture (`*_nyes_transitions` unit tests + `.snap`
   approval tests) matches community consensus.
3. The signing layer is genuinely novel but a faithful SLSA/in-toto transfer,
   directly resolving the #1 critique (review fatigue / rubber-stamping).
4. The AI-vs-human key distinction is well-motivated for 2026 (jlevy/tbd,
  Tian Pan 2026 both flag AI-assisted dev as a new threat to the review
  social contract).

**Where the over-engineering risk lives (not the signing):**
- Signing is pointless if human review is itself a rubber stamp. The vimdiff
  step is load-bearing; the signature is the audit trail. If vimdiff degrades
  to `:wq`, signatures become theatre.
- The signing ceremony must not block legitimate fixes (SLSA: attestation
  overhead should be automated).
- Snapshot size discipline still applies (Beck's "Readable"/"Structure-
  insensitive" desiderata are unsolved by signing).

---

# Appendix E — Suggestions for Enhancing the Foolish Snapshot Experience

*(Prioritised recommendations from the research, mapped to the Foolish-rust
codebase gaps identified during analysis.)*

## E.1 P0 — Close the enforcement gaps (the signing scheme is currently aspirational on the main branch)

1. **Add a Rust CI job.** `.github/workflows/tests.yml` currently runs only
   Java/Scala cross-validation. Add: `cargo insta test --check` + `cargo insta
   pending-snapshots` + `verify_signatures <approved-dirs>`. This is the
   single highest-leverage change — without it, snapshot regressions are
   invisible to CI.
2. **Add `.snap.new`, `.snap.new.approved`, `.snap.new.check` to `.gitignore`.**
   Only `.snap` should be tracked. Currently 80 `.snap.new` files are
   committed on the main worktree.
3. **Remove `INSTA_UPDATE=always cargo test *` from `.claude/settings.json`.**
   It directly contradicts AGENTS.md's hard rule — the one technical
   "safeguard" that exists *permits* the forbidden command.
4. **Merge `foolish_review.sh` / `accept_approved.sh`** from the
   `foop-62-ubca-mimo` worktree to the main branch (or update AGENTS.md to
   reflect they're worktree-local). The documented human-review pipeline is
   non-functional on the main branch today.

## E.2 P1 — Hardening

5. **Add `.gitattributes`**: `*.snap linguist-language=YAML`,
   `*.snap.new linguist-generated=true`, and consider `*.snap merge=ours`
   to prevent signature corruption on merge.
6. **Add `.config/insta.yaml`** with `behavior.require_full_match: true`
   (pins header metadata, not just body) and `test.unreferenced: reject`
   equivalent.
7. **Fix the `accept_approved.sh` hardcoded path** — use
   `cargo run -p foolish-snap --bin verify_signatures --` instead of the
   worktree-pinned `target/debug/verify_signatures`.

## E.3 P2 — Documentation reconciliation

8. **Correct AGENTS.md:** `hssnap` → `hfssnap`; "HS signature" → "HFS
   signature"; add the 4th line (`Comments signature`); fix the
   `bind_dynamic` reference (should be `bind_to_scope`/`bind`/`dynamic_redaction`);
   reconcile `.snap.new.check` description (append-log, not rename).
9. **Correct README.md:** `cargo test -p foolish-ubcb-cli --lib` →
   `cargo test -p foolish-ubcb --lib` (approval_all lives in foolish-ubcb,
   not foolish-ubcb-cli).

## E.4 P3 — Hygiene

10. **Remove the dead insta dev-dep** from `foolish-parser/Cargo.toml`.
11. **Add `[profile.dev.package.insta] opt-level = 3`** and
    `[profile.dev.package.similar] opt-level = 3` to the workspace `Cargo.toml`.
12. **Migrate the test harness** from `Settings::clone_current()` + `set_*`
    + `bind` to the more idiomatic `with_settings!` macro.

## E.5 P4 — Experience enhancements (from cross-language research)

13. **Enable insta's `filters`/`redactions` features** and make a project
    convention to always filter timestamps/IDs *before* the signed body — so
    signatures are naturally stable across runs. (Currently only the `yaml`
    feature is enabled.)
14. **Use `dynamic_redaction`** to validate-then-mask volatile values —
    assert the field *is* a valid UUID/timestamp *before* replacing it, so a
    regression to empty/garbage fails the test instead of being silently redacted.
15. **Use `cargo insta review --snapshot <path>`** (PR #815, non-interactive)
    for agent-assisted review presentation — agents surface diffs, humans
    accept. This is the insta-native path that aligns with the AI-vs-human
    distinction this FOOP formalises.
16. **Keep the 10% reread sampling** in `foolish_review.sh` — it's a cheap
    statistical drift detector not present in vanilla insta. Worth preserving
    as an opt-in QA mode in the library.
17. **Consider `trycmd`** for CLI output snapshots (`foolish-cli run/step/repl`)
    — it's purpose-built for CLI textual interaction, with elision for
    volatile substrings. Complements insta (values) rather than competing.
18. **Pin insta to a specific minor version** (currently `version = "1"`
    floats). Insta's output format has changed across minors; pinning gives
    reproducible snapshot formatting.
19. **Adopt the naga/Mochi practice** of normalising non-deterministic fields
    and splitting by backend/layer if FVM snapshots grow large.
20. **Pair every approval test with a focused unit test on invariants**
    (already done with `*_nyes_transitions` — keep it). Beck's desiderata:
    snapshots are low on "Inspiring"; real assertions force dual-verification.

## Last Updated

**Date**: 2026-07-01
**Updated By**: Sisyphus / mimo-v2.5-pro
**Changes**: Renamed from `foolish-snap` to **Einmo**. Changed from
file-extension-based lifecycle to **directory-based** design (`output/`,
`checked/`, `verified/`). Added **stage comparison** API (`einmo compare
<stage-a> <stage-b>`) for enforcing different correspondences. Added
**generation timestamp** in COMMENTS block (inside signed content).
Promotion is now CLI-driven (`einmo promote output→checked`,
`einmo promote checked→verified`). Checked→verified requires human signing
phrase; output→checked does not. Simplified to three stages (removed Flagged
state — flagging is an annotation concern, not a lifecycle state).

**Date**: 2026-06-27
**Updated By**: Sisyphus / z-ai/glm-5.2
**Changes**: Resolved all blocking Open Questions. OQ-1 (crate — another
project needs it imminently). OQ-2 (two-layer CI gate: Layer 1 always-on
integrity verify on all branches; Layer 2 opt-in `require_human_promotion`
as the P0/tier-1 feature, default OFF, enforced at merge-to-main and tag —
NOT at commit/push). OQ-3 (warn-only 14-day staleness). OQ-4 (unsigned
comment-line timestamp in `util` entries). OQ-6 (expected results NEVER
inlined — always signed `.snap` files; inputs MAY be inlined via
`evaluate_inline`; `Evaluator` trait generalised to return `Vec<String>` so
the library has no Foolish-FIR dependency). OQ-7 (plan file — design now
frozen, plan to follow). Added §9 passphrase cascade, §10 two-layer CI
gate, generalised `Evaluator` trait + `evaluate_inline` API. Only OQ-5
(structured `.check` log) remains open, deferred — not blocking.
