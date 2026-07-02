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

This FOOP specifies **Einmo**, a directory-based, cryptographically signed
snapshot-testing library with a staged promotion pipeline. It replaces `insta`
for the Foolish project and is designed for reuse by any project whose tests
take human-readable input and produce human-readable output.

Einmo's defining requirement — the feature no surveyed snapshot/approval-testing
framework provides — is that **promotion is a staged pipeline** (output →
checked → flagged/verified), each stage is a directory, and **every generated
output is timestamped and signed by the test runner**. Promotion between stages
is a separate, cryptographically attributable act performed via CLI (by human or
AI agent). The library supports **comparing any two stages** so that different
processes can enforce different correspondences (e.g. "output matches checked"
or "checked matches verified").

The body is organised as: Abstract → Motivation → Product Description → Use
Cases A & B → Specification → Web-UI/app-UI → Technical Design → standard FOOP
sections → Appendices carrying the supporting research (self-contained; no
additional searches needed to implement).

## Abstract

**Einmo** is a workspace crate (`einmo`) providing directory-based signed-
snapshot testing with a four-stage promotion pipeline: **output** → **checked**
→ **flagged** / **verified**. Each test suite is configured with a work
directory containing `input/` (test triggers) and four stage directories, each
holding `.einmo` files. The stage directory tree **mirrors** the `input/` tree
at any depth. Generated outputs are timestamped and signed by the test runner.
Promotion from `output→checked` is a CLI operation available to AI agents or
humans (no passphrase). Promotion from `*→verified` appends a human-keyed
signature (passphrase-resolved; defaults to interactive prompt). Any state may
transition to `flagged/` (a terminal sink). The library supports comparing any
two stages; the comparison is per-section (INPUT + RESULT required, COMMENTS
optionally required), with both files independently verified before content is
compared. The crate replaces `insta` for Foolish snapshot testing and is
designed for reuse by other projects.

## Motivation

### What we (the FOOP project) want

The Foolish language implementation uses snapshot/approval tests as the **hard
acceptance gate** for VM behaviour (FOOP-62): a `.foo` input is evaluated, the
humanizing-sequencer output is captured, and a signed `.einmo` file is the
approved behavioural contract. Because the project is developed collaboratively
by humans **and AI agents**, we need:

1. **Staged promotion, not a single accept step.** `insta` collapses "human
   looked at the diff" and "human promoted the baseline" into one `accept`
   step. We want four stages: `output` (generated), `checked` (reviewed),
   `flagged` (set aside), `verified` (human-signed). Each stage is a directory,
   making the pipeline inspectable by CI and by other tools.
2. **Every output is timestamped and signed.** The test runner always signs its
   output with a computer/AI key, embedding the generation timestamp inside the
   signed content. This attestation ("a machine produced this output at this
   time") is permanent and survives promotion.
3. **Compare any two stages.** Different processes enforce different
   correspondences: "output matches checked" (CI gate), "checked matches
   verified" (release gate). The comparison API is stage-agnostic and per-section.
4. **CLI-driven promotion.** Promotion is a command-line operation that can be
   invoked by AI agents (`output→checked`) or humans (`*→verified`, passphrase).
   The CLI determines where to get the signing key — from keyboard console, an
   external API, or test code.

### The world after this FOOP

- A reusable crate `einmo` with directory-based, hierarchical test configuration.
- Each test's work directory: `input/` → `output/*.einmo` → `checked/*.einmo` →
  `flagged/*.einmo` / `verified/*.einmo`, all mirroring the `input/` tree.
- Generated outputs carry a `test` signer entry with a signed generation timestamp.
- `cargo einmo promote output→checked` (AI/human, no passphrase).
- `cargo einmo promote checked→verified` (human, passphrase; defaults to interactive).
- `cargo einmo flag <stage>` (any state → flagged, terminal sink).
- `cargo einmo compare <stage-a> <stage-b>` — stage-agnostic, per-section comparison.
- CI gates on stage correspondence, not just existence.

## Product Description

**Einmo** is a workspace crate (`einmo`) providing directory-based,
cryptographically signed snapshot testing with a staged promotion pipeline. It
is built on two capabilities that, together, no surveyed framework provides:

1. **A programmatic test-construction API** that gives test code first-class
   access to every stage of verification — so a test can assert "output matches
   checked" (CI gate), "checked matches verified" (release gate), or "every
   verified file is signed by key `eb108…`" (release attestation), all against
   signed, tamper-evident artifacts.
2. **A CLI (`cargo einmo …`) that manages stage-wise promotion with
   cryptographic signing at each transition** — promotion is a separate,
   attributable act, not an automated `accept`. Every generated output is
   timestamped and signed by the test runner; every promotion to `verified`
   appends a human-keyed signature.

And four structural commitments that distinguish it from `insta` and all
surveyed frameworks:

3. **Hierarchical storage** — stage directories mirror the `input/` tree at any
   depth, decoupling test organisation from test code. `insta`'s flat,
   module-coupled `.snap` storage cannot express hierarchical input
   organisation; Einmo makes it the structural backbone (and a software-
   granularity model — see §Design preference).
4. **Randomized re-inspection** — a configurable random sample of promoted
   files is demoted and re-presented for review, catching baseline rot that
   code-triggered diff checks cannot. No surveyed framework does this.
5. **No automated update ever** — the test runner writes only to `output/`;
   `checked`/`flagged`/`verified` are never touched except by a deliberate
   promotion command. There is no `--accept`, no `--update`, no `INSTA_UPDATE`
   equivalent. Einmo does not depend on `insta` for output generation (it writes
   `.einmo` files directly).
6. **Self-describing signed artifacts** — each `.einmo` file carries its input,
   output, timestamps, and signatures in one human-readable text file. Unlike
   `insta`'s unsigned `.snap`, the signature is part of the file; the artifact
   is co-located with its attestation.

### The four stages and their signing

| Stage | Directory | Signer entries | Who can produce | Passphrase |
|---|---|---|---|---|
| **Output** | `output/` | `test` (computer key + signed generation timestamp) | test runner (always) | default `""` (computer key) |
| **Checked** | `checked/` | `test` (preserved) | AI agent or human via CLI (`promote output→checked`) | none |
| **Flagged** | `flagged/` | `test` (preserved) + advisory `# flagged:` line | any state → flagged via CLI | none |
| **Verified** | `verified/` | `test` (preserved) + `util` (promotion key + signed promotion timestamp) | human via CLI (`promote *→verified`) | resolved via cascade; default falls through to interactive prompt |

**Critical invariants (typed, enforced in code):**

1. A file cannot enter `output/` without a `test` entry — generation always signs.
2. The `test` entry is never modified or removed by any transition.
3. Only `*→verified` appends a signature (`util`); all other transitions are
   move/copy.
4. Flagging = **move** (origin vacated, `flagged/` populated); collisions get a
   timestamp suffix.
5. **Verify-on-inspect**: any operation that reads a `.einmo` file verifies *all*
   signer entries first; tampered files are refused, never operated on.

## Use Case A — Constructing tests: human-readable input → human-readable output

### A.1 What-if — the vision

Imagine a test framework where the test author never writes an expected value —
they write an input and an evaluator, and the framework captures the behaviour
as a signed, reviewable artifact. Now imagine extending that:

- **Parameterized / generated inputs.** A test generates inputs from a seed
  (property-based style) and writes one `.einmo` per generated case — pinning
  the VM's behaviour across a fuzz space.
- **Cross-implementation comparison.** The same `input/` tree evaluated by two
  `Evaluator`s (e.g. ubca vs ubcb) into `output-ubca/` and `output-ubcb/`, with
  a `compare output-ubca output-ubcb` gate. (Deferred per FOOP-03 for JVM, but
  the Rust-vs-Rust path is structurally available.)
- **Tolerance-based matching.** `compare` with a numeric tolerance for
  floating-point RESULT sections (pytest-regtest style), not just byte-identity.
- **A redaction library.** Built-in redactors for common volatile values
  (timestamps, UUIDs, memory addresses, run-ids) that normalise the RESULT
  before canonicalisation, so signatures are stable across runs.
- **Dependency-aware test selection.** Given a code change, infer which
  `input/` leaves are affected (via the granularity DAG) and run only those,
  leaving the rest of `output/` untouched — faster CI on large suites.
- **Snapshot fuzzing / minimisation.** When a `.einmo` mismatches, auto-minimise
  the input to the smallest case that still reproduces the divergence.
- **Inline-expected-value generation from a one-off run.** `evaluate_inline`
  that, on first run, emits the `.einmo` to `output/` for review rather than
  failing — bootstrapping new tests without manual baseline authoring.
- **Time-travel debugging.** Since every `output/` is signed and timestamped,
  reconstruct the VM's behaviour at any past commit by replaying the `input/`
  against that commit and diffing against the historical `output/`.

The common thread: the signed `.einmo` is a behavioural contract that scales
from a single inlined assertion to a whole-program fuzz corpus, all under one
signed-corpus regime with a unified diagnostic.

### A.2 So, then — what the first version of einmo supports

From that vision, the first version chooses to support:
- `Evaluator` trait returning `Vec<String>` (no FIR dependency).
- `EinmoSuite::evaluate` (file input), `evaluate_inline` (in-code input),
  `evaluate_all` / `evaluate_all_inline` (parallel).
- `TestConfig` with `require_correspondence` and `match_sections` (per-section
  matching: INPUT+RESULT required, COMMENTS optional).
- Tests write signed `.einmo` to `output/<mirror-path>`; the test asserts
  correspondence via `compare`.
- Inline expected values (`@"…"`) are refused.

Explicitly **out of scope** for v1: parameterized/generated inputs, cross-impl
comparison, numeric tolerances, a redaction library (redaction is the test
author's responsibility via pre-canonicalisation string ops), dependency-aware
selection, fuzz-minimisation, one-off baseline bootstrapping, time-travel
replay. These are documented as future enhancements (§Open Questions / Deferred).

### A.3 The test author's contract

A test takes a **human-readable input** (a `.foo` file on disk, or a string
passed in code) and produces a **human-readable output** (the formatted
sequencer result). Einmo captures both into a single signed `.einmo` artifact —
a text file containing the input, the output, a comments block, per-file
metadata (with generation timestamp), and a signatures block. The `.einmo` file
*is* the behavioural contract; it is human-readable, diffable, and
cryptographically attested.

The test author writes assertions against **stage correspondence**, never
against inline expected values. An inline expected value
(`assert_snapshot!(…, @"…")`) cannot carry a `SIGNATURES:` block and therefore
cannot be content-protected — Einmo refuses it. Expected results always live as
signed `.einmo` files in a stage directory. Inputs, however, may be inlined
(not every system has input files on disk).

### A.4 The API

```rust
/// Generalised evaluator — no Foolish FIR dependency. Returns formatted
/// output blocks (human-readable strings). Adapters format FIRs to strings
/// internally before returning them.
pub trait Evaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String>;
}

pub struct EinmoSuite { config: TestConfig, /* ... */ }

impl EinmoSuite {
    pub fn new(config: TestConfig) -> Self;
    /// File-based input: discovers .foo files under input/, evaluates each,
    /// writes a signed .einmo to output/<mirror-path>.
    pub fn evaluate(&self, path: &Path, evaluator: &dyn Evaluator) -> Result<String, String>;
    /// Inlined input: the input is a string in code, not a file. Einmo still
    /// produces a signed .einmo (the inlined input is captured into the
    /// INPUT block and signed). `name` becomes the filename.
    pub fn evaluate_inline(&self, name: &str, input: &str, evaluator: &dyn Evaluator) -> Result<String, String>;
    /// Discover all inputs in input/ tree, evaluate in parallel, write all
    /// outputs. Returns per-file results.
    pub fn evaluate_all(&self, threads: usize, evaluator: &dyn Evaluator) -> TestResults;
    pub fn evaluate_all_inline(&self, pairs: &[(name, input)], threads: usize, evaluator: &dyn Evaluator) -> TestResults;
}

pub struct TestConfig {
    pub work_dir: PathBuf,
    pub input_dir: String,            // default "input"
    pub stages: StageDirs,            // defaults: output/checked/flagged/verified
    pub require_correspondence: Vec<(Stage, Stage)>,  // e.g. [(Output, Checked)] for CI
    pub match_sections: MatchSections, // default InputResult
}
pub enum Stage { Output, Checked, Flagged, Verified }
```

### A.5 What a test module looks like

```rust
#[test]
fn ubca_approval_all() {
    let config = TestConfig::default("foolish-ubca/snapshot_tests")
        .require_correspondence(Stage::Output, Stage::Checked);   // CI gate
    let suite = EinmoSuite::new(config);
    let evaluator = UbcaEvaluator::new();   // adapts FIR → Vec<String>
    let results = suite.evaluate_all(num_cpus::get(), &evaluator);
    assert!(results.all_output_written_and_verified());
    // The correspondence assertion (output == checked) is enforced inside
    // evaluate_all via `compare`; a mismatch fails the test with a diff.
}
```

## Use Case B — Stage-wise cryptographic signing + CLI navigation

### B.1 What-if — the vision

Imagine every test output carrying a permanent, attributable attestation chain
— a machine signed it at generation, a human signed it at promotion, a second
human re-signed it at re-inspection, and every signature is tamper-evident
against all prior bytes. Now extend that:

- **Threshold multi-sig.** `verified/` requires m-of-n human signatures (e.g. 2
  of 3 release officers) — a single coerced key cannot ship.
- **Key rotation & delegation chains.** A release officer delegates a scoped
  signing key to a team for a release window; the delegation itself is signed
  and expires.
- **HSM / hardware-token integration.** The promotion key never leaves the
  device; `einmo promote` hands the canonical bytes to the HSM for signing.
- **Air-gapped offline signing.** The `verified/` corpus is signed on an
  offline machine; signatures are carried back to the online repo on
  removable media, never exposing the key to a network.
- **Per-agent cryptographic identity.** Each AI agent has its own derived key,
  so `test` entries are attributable not just to "a machine" but to "agent X" —
  the post-hoc attribution becomes per-actor, not just computer-vs-human.
- **Signature-graph visualisation.** A UI rendering the attestation chain of
  the corpus over time — who signed what, when, in what order.
- **Signed test-selection / coverage attestation.** CI signs not just the
  outputs but *which tests ran*, so a release can prove coverage of a
  sub-attribute set.
- **Cross-repo release attestation.** A release aggregates signed einmo corpora
  from multiple repos into one release attestation (in-toto/SLSA-shaped).
- **Time-locked signatures.** A signature valid only after a date (vulnerable-
  disclosure embargoes, timed releases).

The common thread: the signature is not a gatekeeping stamp but a living
provenance graph over the behavioural contract.

### B.2 So, then — what the first version of einmo supports

From that vision, the first version chooses to support:
- Ed25519 via Argon2id key derivation (FOOP-12); `test` (computer) + `util`
  (promotion) signer roles in FOOP-22 append format with `Entire file`
  progressive-chain integrity.
- Passphrase cascade (5 tiers: `--passphrase` > `--stdin-passphrase` >
  `EINMO_PASSPHRASE` env > `einmo.toml` > interactive `/dev/tty` prompt);
  `--interactive` forces the prompt.
- `confirm-signatures <path> <pubkey-prefix> [--require-all]` for release-key
  attestation.
- **Emergent human-attestation**: the repo deliberately omits a verified-stage
  passphrase, so `*→verified` falls through to the interactive prompt. An AI
  piping `--passphrase ""` produces a `util` under the computer key — post-hoc
  detectable (`util` pubkey == `test` pubkey).
- Signed generation + promotion timestamps (inside signed content).

Explicitly **out of scope** for v1: threshold multi-sig, key rotation /
delegation, HSM / hardware-token integration, air-gapped offline signing,
per-agent cryptographic identity (all agents share the computer key),
signature-graph visualisation (beyond `einmo show`'s signer summary), signed
test-selection / coverage attestation, cross-repo release attestation,
time-locked signatures. Documented as future enhancements.

### B.3 Timestamping

| Event | Where it lives | Signed? |
|---|---|---|
| Generation (output written) | per-file metadata `generated: <ISO8601>` | **Yes** — covered by `test`'s `Metadata` signature |
| Promotion to verified | inside `util` entry's signed content (`promoted: <ISO8601>`) | **Yes** — covered by `util`'s signatures |
| Flag reason + time | advisory `# flagged: <reason> <ISO8601>` line outside signed content | No (advisory; original sigs stay valid without re-signing) |

Signing both timestamps means re-signing produces different signature bytes —
acceptable: the *content* a signature covers is verifiable regardless.
Tamper-evidence is the load-bearing property; byte-reproducible re-signing is not.

### B.4 Signing — keys and algorithms

- **Algorithm**: Ed25519, key derived from passphrase via Argon2id (FOOP-12).
- **`test` entry** (computer/AI key): the test runner signs with the empty-
  passphrase key by default (canonical computer key, pubkey `dc5f586c…b683`).
  Created once at generation, never removed.
- **`util` entry** (promotion key): resolved via the passphrase cascade (B.3).
  Carries `Entire file` + progressive triple. Multiple `util` entries accumulate
  (re-review, re-promotion); each subsequent `util`'s `Entire file` covers all
  prior entries, forming a tamper-evident chain.
- **Emergent human-attestation** (not enforced): the system is configured
  *without* a passphrase for `verified`-stage promotion, so `*→verified` falls
  through the cascade to the interactive prompt. A human types it; an AI agent
  running non-interactively with no tty cannot complete the promotion. If an
  agent pipes `--passphrase ""`, it produces a `util` entry under the
  **computer key** — **post-hoc detectable**: the `util` pubkey equals the
  `test` pubkey. The signature provides attribution; the policy provides the gate.

### B.5 Key sources — passphrase resolution cascade

Uniform cascade for all signing operations:

| Tier | Source | Notes |
|---|---|---|
| 1 | `--passphrase <value>` | explicit, non-interactive (CI/scripted) |
| 2 | `--stdin-passphrase` | read one line from stdin (pipe or tty) |
| 3 | `EINMO_PASSPHRASE` env var | per-process override |
| 4 | `einmo.toml` `[signing] passphrase = "…"` | per-repo default (typically the computer key `""`) |
| 5 | **interactive prompt** on `/dev/tty` | only if no tier above yielded a value |

- An explicit empty string (`--passphrase ""` or `EINMO_PASSPHRASE=""`) is **set
  to empty** (computer key), not "unset" — to unset, omit the flag/var entirely.
- `--interactive` flag: **forces the prompt**, skipping tiers 1–4.
- Per-stage deployment convention: a repo's `einmo.toml` sets
  `[signing] passphrase = ""` for the computer key. It deliberately does **not**
  set a verified-stage passphrase, so `*→verified` falls through to the
  interactive prompt — the human-attestation gate is emergent from
  configuration, not a code-enforced rule.

### B.6 The CLI (`cargo einmo …`)

Binary is `cargo-einmo` (cargo subcommand convention). **Every subcommand
verifies all signatures on any file it touches before proceeding**
(verify-on-inspect); tampered files are refused.

```bash
# Promotion & flagging
cargo einmo promote <from>→<to> <work_dir> [--filter <glob>] [--passphrase <v> | --stdin-passphrase | --interactive] [--batch]
  # legal: output→checked, output→verified, checked→verified,
  #        output→flagged, checked→flagged, verified→flagged
  # output→checked : copy (test preserved, no new sig)
  # *→verified     : copy + append util; warns if util key == test key
  # *→flagged      : move (same as `flag`)
  # --batch        : one passphrase prompt for all matching files

cargo einmo flag <work_dir> <stage> [--filter <glob>] [--reason <text>]
  # moves <stage>/<rel> → flagged/<rel>; collision → timestamp suffix
  # appends advisory "# flagged: <reason> <ISO8601>"

# Comparison & verification (CI)
cargo einmo compare <stage-a> <stage-b> <work_dir> [--match-sections input,result] [--require-comments-match] [--stale-days N] [--filter <glob>] [--require-match] [--json]
cargo einmo verify <work_dir> [--stage <s> | --all]

# Signature inspection
cargo einmo confirm-signatures <path> <pubkey-prefix> [--require-all]

# Inspection
cargo einmo show <file>

# Review (replaces foolish_review.sh)
cargo einmo console-review <work_dir> <from>→<to> [--filter <glob>] [--full] [--reexamine-rate <pct>] [--reexamine-seed <seed>] [--vim | --list] [--root-cause]

# UI server (Proposal A)
cargo einmo serve <work_dir> [--bind <addr>]

# Self-attestation (integrity check on the CLI's own binary)
cargo einmo self-check [--expected <sha256>] [--quiet]
  # computes SHA-256 of env::current_exe()?; prints path + hash
  # --expected <sha256>: exit non-zero if the computed hash does not match
  # --quiet: print only the hash (for scripting)
```

## Use Case C — Provide inspection service

This use case covers the **inspection layer**: the services that let actors
(AI agents and human reviewers) navigate, review, and act on the einmo corpus.
Einmo is the backend in every case; the output under inspection is always
byte-steady, signable content.

### C.1 What-if — the vision

Imagine any actor — an AI agent or a human — inspecting the same signed corpus
through whichever frontend suits them, with every action mediated by the einmo
library so the invariants always hold:

- **Agent inspection via MCP.** A Model Context Protocol server exposes every
  einmo operation as a structured tool: navigate the directory tree, list by
  stage, fetch a diff, promote, flag, verify, confirm-signatures, run
  root-cause bisection. An agent reviews `output→checked` by calling tools,
  not by shelling out and parsing text.
- **AGENTS.md skills as encoded review flows.** The standard review protocols
  (reconcile output vs checked; decide repair/promote/escalate; the
  burden-of-correction; flag-vs-escalate decision) are written as reusable
  skills — step-by-step instructions an agent loads and follows, so review
  discipline is not reinvented per agent.
- **Agent self-attribution.** Each agent signs with its own derived key, so
  `test`/`util` entries are attributable not just to "a machine" but to "agent
  X" — post-hoc attribution becomes per-actor.
- **Agent-driven auto-bisection.** On a mismatch, the agent descends the
  granularity tree (`--root-cause`) automatically, finding the deepest
  differing leaf without human steering.
- **Rich human inspection.** For outputs that are themselves rich — an HTML
  report, an SVG diagram, a structured data dump — the `.einmo` RESULT block
  *contains* that rich content. The browser renders it with built-in
  search/analysis: syntax highlighting, folding, a data-structure explorer,
  in-content grep. Approve/disapprove buttons sit inline on the rendered
  output. The einmo service is the backend; the rich content is the signed
  payload; the UI is a view layer over it.
- **Collaborative review.** Multiple humans on one corpus: review assignments,
  inline comments, PR/Slack integration, review queues shared across a team.
- **Cross-repo attestation dashboards.** A release officer inspects an
  aggregated view of signed corpora across repos before tagging.

The common thread: inspection is not a separate tool layered on top of einmo —
it is einmo's service surface, exposing the same signed corpus to every actor
through fit-for-purpose frontends.

### C.2 So, then — what the first version of einmo supports

From that vision, the first version chooses to support:
- An **MCP server** (`cargo einmo serve --mcp`, or a dedicated `cargo-einmo-mcp`
  binary) exposing: `list`, `diff`, `promote`, `flag`, `verify`,
  `confirm_signatures`, `root_cause`, `show` as structured tools. The MCP
  server calls the einmo library — it is a frontend, never touches `.einmo`
  files directly. (Agents that prefer shells use the `cargo einmo … --json`
  CLI; both call the same library.)
- An **AGENTS.md documentation template + skills** encoding the standard review
  flows (reconcile output vs checked; burden-of-correction; flag-vs-escalate).
  Shipped as part of the einmo crate's docs so adopting projects drop them into
  their own `AGENTS.md`.
- **`einmo serve`** (axum, REST/WebSocket): suite overview tree, per-section
  diff view, promote/flag/verify/confirm-signatures/show endpoints, and a
  WebSocket alert feed (output≠checked, checked≠verified, flagged, staleness,
  signature failures).
- **Rich-output rendering.** The RESULT block may contain HTML (or other
  browser-renderable content). The SPA fetches the `.einmo` via `/api/show`
  (the backend verifies-on-inspect server-side), extracts the RESULT, and
  renders it. An **einmo-in-HTML metadata convention** carries the einmo
  metadata inside the HTML (a `<script type="application/json"
  id="einmo-meta">{…}</script>` block) so the SPA renders stage/signature chrome
  + approve/disapprove buttons around the content. The buttons POST to
  `/api/promote` or `/api/flag`.
- **Byte-steadiness invariant** (critical, see C.4): the signed RESULT content
  is canonicalised to a deterministic byte string before signing; the rich
  HTML's interactive features are client-side transforms of those signed bytes,
  never part of what is mutated post-signing.
- One SPA frontend (static, served by `einmo serve`).

Explicitly **out of scope** for v1: per-agent cryptographic identity (agents
share the computer key — that is Use Case B's v1 scope), collaborative
multi-human review / assignments / inline comments, PR/Slack integration,
agent auto-bisection as a built-in (the `--root-cause` flag provides the data;
agents orchestrate the bisection), custom per-output interactive widgets
beyond HTML rendering, cross-repo attestation dashboards. Documented as future
enhancements.

### C.3 AI-agent inspection service (MCP + AGENTS.md skills)

**MCP server.** A Model Context Protocol server exposes einmo operations as
structured tools. Each tool calls the einmo library (Rust); the server is a
frontend that enforces verify-on-inspect on every read and never derives keys
outside the library. Tool surface:

| MCP tool | Library call | Purpose |
|---|---|---|
| `einmo_list(work_dir, stage)` | `walk_stage` | list files in a stage (mirror-relative paths) |
| `einmo_diff(work_dir, a, b, rel_path)` | `compare` (single file) | per-section diff between two stages for one file |
| `einmo_promote(work_dir, from, to, filter, passphrase?)` | `promote` | promote (passphrase only for `*→verified`) |
| `einmo_flag(work_dir, stage, filter, reason)` | `flag` | move to flagged/ |
| `einmo_verify(work_dir, stage?)` | `verify` | signature integrity |
| `einmo_confirm_signatures(path, prefix, require_all)` | `confirm_signatures` | release-key attestation |
| `einmo_root_cause(work_dir, a, b, rel_path)` | `compare --root-cause` | descend subtree; deepest differing descendants |
| `einmo_show(file)` | `EinmoFile::from_file` + `signers` | verified content + signer summary |

Agents that prefer shells use `cargo einmo … --json`; both paths call the same
library, so invariants are identical.

**AGENTS.md skills (documentation template).** Einmo ships a documented set of
review-flow skills that adopting projects drop into their `AGENTS.md`. Each
skill is a step-by-step protocol:

- **Reconcile output vs checked** (the agent's pre-commit flow): run
  `compare output checked`; for each `differing`/`only_in_a` file, decide
  repair (fix code so output matches checked), promote (review the diff and
  `promote output→checked`), or escalate (`flag` with a reason and surface to
  a human). Re-run until `compare` is clean, then commit.
- **Burden of correction** (on gate failure): if `output≠checked`, the producer
  of the divergent output repairs or escalates; if `checked≠verified`, the
  producer of the checked version corrects or escalates. The skill encodes
  *who* bears the burden (attributable via the signer chain) and the accepted
  resolution paths.
- **Flag vs escalate decision**: flag when the artifact itself is wrong (move
  to `flagged/`, regenerate output after fixing code); escalate when the
  agent cannot determine whether the divergence is a regression or a new-correct
  baseline (flag + surface to a human reviewer with the diff).
- **Randomized re-inspection participation**: when a scheduled re-inspection
  demotes a sample, the agent reviews the demoted files using the same
  reconcile flow.

The skills are prose + tool-call sequences, designed to be loaded by an agent
at the start of a review task.

### C.4 Human inspection service (web service + rich output rendering)

**`einmo serve` backend.** The axum HTTP/WebSocket server (Technical Design §1)
is the backend for human inspection. REST endpoints: `/api/tree` (suite
overview), `/api/diff` (per-section diff, signature lines hidden), `/api/promote`,
`/api/flag`, `/api/verify`, `/api/confirm-signatures`, `/api/show`, and a
`/ws/alerts` stream. The server verifies-on-inspect on every read; passphrases
arrive via POST body, are derived to a key, used to sign, and dropped — the UI
never holds a private key.

**Rich output rendering.** The `.einmo` RESULT block may contain HTML (or SVG,
or other browser-renderable content), not just plain text. The SPA fetches the
`.einmo` via `/api/show`; the backend verifies all signatures server-side
(verify-on-inspect) and returns the parsed content; the SPA renders the RESULT
HTML in a sandboxed container with the einmo chrome around it.

**Einmo-in-HTML metadata convention.** When the output IS HTML, it carries an
embedded metadata block so the SPA can render stage/signature chrome and
approve/disapprove buttons without a separate API round-trip per render:

```html
<!-- the signed HTML output, byte-steady -->
<script type="application/json" id="einmo-meta">
{"input_path":"stage1/section3/specific.test","stage":"output",
 "signers":[{"role":"test","pubkey":"dc5f586c…"}]}
</script>
<!-- ...the renderable, searchable, analysable output content... -->
```

The SPA detects `#einmo-meta`, renders the stage badge + signature status +
approve/disapprove buttons, and renders the HTML content. Approve POSTs to
`/api/promote`; disapprove opens a reason field and POSTs to `/api/flag`. The
built-in search/analysis (syntax highlighting, folding, data-structure
explorer, in-content grep) is client-side JS that operates **on the signed HTML
bytes** — it renders/transforms them for display; it does not mutate the signed
content.

**Byte-steadiness invariant (critical).** *In all cases, the output has to be
byte-steady output that is signable using cryptographic signature.* Concretely:

1. **Canonicalise before signing.** The RESULT content (HTML or otherwise) is
   canonicalised to a deterministic byte string before the `Result`/`Metadata`
   signatures are computed. No volatile values (timestamps, random IDs,
   run-order-dependent serialisation) in the signed bytes — redact/mask them
   before canonicalisation (the test author's responsibility in v1; a redaction
   library is a future enhancement, Use Case A §A.1).
2. **The signature covers the canonical bytes.** The `test`/`util` `Result`
   signature covers `canon_input + canon_result`; the `Entire file` signature
   covers all prior bytes. Any change to the canonical HTML invalidates the
   signature — tamper-evident.
3. **Interactive features are a view layer.** The HTML's client-side JS
   (search, fold, highlight, data-structure exploration) renders/transforms the
   signed bytes for display. It never writes back to the `.einmo` file; it
   never mutates what is signed. The signed bytes are immutable in the file;
   the UI is a projector over them.
4. **External resources are not signed.** If the HTML embeds external resources
   (images, scripts fetched at view time), those are NOT part of the signed
   RESULT bytes — only the HTML text in the RESULT block is signed. For full
   attestation of external resources, they would need their own signing (out
   of scope v1; documented as a future enhancement).
5. **Verify-on-inspect applies.** Before the SPA renders the RESULT, the
   backend loads the `.einmo` via `EinmoFile::from_file` (which verifies all
   signatures); a tampered file is refused and surfaced as an alert, never
   rendered as if valid.

This invariant is what makes rich, interactive, searchable output compatible
with cryptographic signing: the signed payload is byte-steady; the
interactivity is a non-mutating view layer over it.

## Specification

### 1. Crate structure

The library is a workspace member **`einmo`** (path `foolish/einmo/`), depended
on by `foolish-core`, `foolish-ubca`, and `foolish-ubcb` as a dev-dependency.

```
foolish/einmo/
├── Cargo.toml
└── src/
    ├── lib.rs              # re-exports
    ├── config.rs           # TestConfig, StageDirs, MatchSections
    ├── stage.rs            # Stage enum + directory operations
    ├── compare.rs          # per-section stage-to-stage comparison
    ├── format.rs           # .einmo file parse/serialize (per-section canonical)
    ├── signature.rs        # moved from foolish-core/src/signature.rs; FOOP-22 append
    ├── snapshot_suite.rs   # moved from foolish-core/src/snapshot_suite.rs; generalised Evaluator
    ├── migrate.rs          # .snap → .einmo converter
    ├── verify.rs           # verify-on-inspect + verify-all (clean submodule; no fs/tty/argon2)
    └── bin/
        ├── cargo_einmo.rs          # CLI: promote, flag, compare, verify, confirm-signatures, show, console-review, serve
        └── verify_signatures.rs    # moved from foolish-core/src/bin/ (restore; fix .gitignore bin/ bug)
```

**`.gitignore` fix (blocking):** the root `.gitignore` currently has a `bin/`
pattern that ignores *any* `bin/` directory including `src/bin/`, which is why
`foolish-core/src/bin/verify_signatures.rs` is missing (dangling `[[bin]]`
declaration in `foolish-core/Cargo.toml`). Narrow to `/bin/` or
`target/**/bin/` before creating `einmo/src/bin/`.

### 2. Directory-based, hierarchical test configuration

Each test suite is configured with a **work directory** whose stage directories
**mirror the `input/` tree at any depth**:

```
test_suite/
├── input/                  # test trigger files (*.foo), free-form hierarchical tree
│   └── stage1/section3/specific.test
├── output/                 # generated outputs (test-runner signed)
│   └── stage1/section3/specific.test.einmo
├── checked/                # reviewed outputs (AI or human promoted)
│   └── stage1/section3/specific.test.einmo
├── flagged/                # set-aside files (any state → flagged; terminal sink)
│   └── stage1/section3/specific.test.einmo
└── verified/               # human-signed outputs (passphrase required)
    └── stage1/section3/specific.test.einmo
```

A flat `input/` yields a flat stage tree. The same basename in different
branches coexists without collision. `compare` walks both trees in parallel and
reports per-path correspondence. **All directories are git-tracked** — output
drift is visible in `git diff`, forcing review.

### 3. The four-stage lifecycle

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Output,    // Generated by test runner. Signed generation timestamp + computer key.
    Checked,   // Reviewed — promoted from output by AI agent or human. No passphrase. `test` preserved.
    Flagged,   // Set aside — any state → flagged via CLI. Move (origin vacated). Advisory `# flagged:` line. Terminal sink.
    Verified,  // Promoted from checked (or output) by human with passphrase. `util` entry appended.
}
```

**Transitions:**
- `output→checked`: copy (test entry preserved, no new sig).
- `output→verified`, `checked→verified`: copy + append `util` entry.
- `output→flagged`, `checked→flagged`, `verified→flagged`: move (remove from
  origin, create in `flagged/`).
- `console-review` demotion: `verified→checked` (move; `util` entry preserved
  as history; re-promotion appends a second `util`).

**Flagged collision handling:** if `flagged/<rel>` already exists when a new
file is flagged to the same path, the new file gets a timestamp suffix. The
common case (one flag per path) keeps the clean mirror; collisions disambiguate.
The flag reason + origin stage are captured in the advisory `# flagged:` line.

### 4. Per-file metadata and the `.einmo` format

```
--- metadata ---
generated: 2026-07-01T15:30:45Z          # per-file generation timestamp (ISO8601 UTC)
suite: foolish-ubca/snapshot_tests       # suite identity
input_path: stage1/section3/specific.test # mirror-relative input path

--- INPUT ---
```foolish
<source>
```
--- RESULT ---
```hfssnap
<formatted output>
```
--- COMMENTS ---
```markdown
<test name; promotion history; etc.>
```
--- SIGNATURES ---
  * Signed by test: <hex_pubkey>
    * Input:     <sig over canon_input>
    * Result:    <sig over canon_input + canon_result>
    * Comments:  <sig over canon_input + canon_result + canon_comments>
    * Metadata:  <sig over canon_input + canon_result + canon_comments + canon_metadata>
  * Signed by util: <hex_pubkey>          # present only in verified/
    * Entire file: <sig over all file bytes before this entry>
    * Input:       <sig over canon_input>
    * Result:      <sig over canon_input + canon_result>
    * Comments:    <sig over canon_input + canon_result + canon_comments>
    * Metadata:    <sig over canon_input + canon_result + canon_comments + canon_metadata + promoted_timestamp>
```

**Per-section signatures.** Each canonical block (Input, Result, Comments,
Metadata) is progressively signed — each signature covers its own block plus all
prior blocks, forming a tamper-evident content chain. The `Metadata` signature
covers the generation timestamp (so the timestamp is signed). The `util` entry
repeats the progressive triple plus an `Entire file` signature over every byte
before it (the inter-signer chain).

**Per-file metadata** (the `--- metadata ---` block): generation timestamp,
suite identity, mirror-relative input path. Signed content. Promotion timestamps
live inside the `util` entry's signed content, not in the per-file metadata.

**Advisory lines (unsigned, outside signed content).** Two advisory lines may
appear in a `.einmo`, excluded from all canonical signed content (like the
`# flagged:` line) so they do not affect signatures:
- `# flagged: <reason> <ISO8601>` — present in `flagged/` files (§3).
- `# produced-by: cargo-einmo <version> sha256:<binary-hash>` — appended by the
  tool on write (§12); attributable producer provenance, non-binding on the
  signature.

Both are informational; the parser distinguishes them from the signed blocks
(INPUT/RESULT/COMMENTS/metadata/SIGNATURES) and excludes them from
canonicalisation.

### 5. Formal comparison semantics (stage matching)

The `compare <stage-a> <stage-b>` operation walks both stage trees in parallel
and, for each mirror-relative path present in both, applies the **matching
test**, which is **per-section**:

> Two `.einmo` files **match** iff:
> 1. File A **verifies correctly against its own signatures** — every signer
>    entry validates, and each section's signature matches the section's
>    canonical content.
> 2. File B **verifies correctly against its own signatures** — same.
> 3. The **configured sections** of A and B are **byte-identical**, section by
>    section:
>    - **INPUT** — required (always compared)
>    - **RESULT** — required (always compared)
>    - **COMMENTS** — *optionally required* (configurable)

Sections not listed are **excluded from content comparison**:
- **SIGNATURES** — legitimately differs between stages; never compared.
- **metadata** — identical across stages by construction (promotion preserves
  it). If it drifted, verify-on-inspect (steps 1–2) would catch the corruption.

**Why COMMENTS is optionally required:** COMMENTS holds the test name (stable)
but may also carry review annotations that legitimately differ between `output`
and `checked`. Some suites want COMMENTS locked; others treat it as advisory.

```rust
pub fn compare(config: &TestConfig, a: Stage, b: Stage, sections: MatchSections) -> ComparisonResult;

pub enum MatchSections { InputResult, InputResultComments }

pub struct ComparisonResult {
    pub matching: Vec<PathBuf>,
    pub differing: Vec<DiffEntry>,        // names which section(s) differed
    pub only_in_a: Vec<PathBuf>,
    pub only_in_b: Vec<PathBuf>,
    pub tampered: Vec<PathBuf>,           // failed verification; refused
}
```

### 6. Promotion CLI

```bash
# Promote output → checked (AI agent or human, no passphrase)
cargo einmo promote output→checked <work_dir> [--filter <glob>]

# Promote checked → verified (human, passphrase; defaults to interactive)
cargo einmo promote checked→verified <work_dir> [--passphrase <v> | --stdin-passphrase | --interactive] [--batch]

# Promote output → verified directly (skipping checked, human only)
cargo einmo promote output→verified <work_dir> [--stdin-passphrase | --interactive]
```

The CLI resolves the signing key via the cascade (B.3). Promotion to `verified`
warns if the resolved key equals the computer key (the `util` pubkey would equal
the `test` pubkey — post-hoc detectable as a non-human attestation).

### 7. Stage comparison CLI

```bash
cargo einmo compare <stage-a> <stage-b> <work_dir> [--match-sections input,result] [--require-comments-match] [--stale-days N] [--filter <glob>] [--require-match] [--json] [--root-cause]
```

- `--match-sections <list>` — which sections must be byte-identical. Default
  `input,result`. Use `input,result,comments` to require COMMENTS too.
- `--require-match` — exit non-zero if any file is `differing`, `only_in_a`, or
  `only_in_b` (used by the gates).
- `--root-cause` — on a `differing` file, descend its subtree; report the
  deepest `differing` descendants (the candidate root causes). See §Design
  preference.
- `--stale-days N` — warn about files in stage-b whose mtime is older than N
  days relative to stage-a.

### 8. Passphrase resolution cascade

See B.3. Implemented in `einmo::config::resolve_passphrase()` with precedence:
`--passphrase` > `--stdin-passphrase` > `EINMO_PASSPHRASE` env > `einmo.toml`
`[signing] passphrase` > interactive `/dev/tty` prompt. `--interactive` forces
the prompt. Config-file parsing: `einmo.toml` with `[signing] passphrase` and
`[ci]` / `[review]` sections.

### 9. Library API

```rust
pub fn run_tests(config: &TestConfig, evaluator: &dyn Evaluator) -> TestResults;
pub fn promote(config: &TestConfig, from: Stage, to: Stage, key: &KeySource) -> Result<PromotionReport>;
pub fn compare(config: &TestConfig, a: Stage, b: Stage, sections: MatchSections) -> ComparisonResult;
pub fn flag(config: &TestConfig, stage: Stage, filter: &str, reason: &str) -> Result<FlagReport>;
pub fn verify(config: &TestConfig, stage: Option<Stage>) -> VerificationReport;
pub fn confirm_signatures(path: &Path, pubkey_prefix: &str) -> SignatureReport;

pub struct EinmoFile { /* parsed + verified */ }
impl EinmoFile {
    pub fn from_file(path: &Path) -> Result<Self, EinmoError>;  // verify-on-load
    pub fn stage(&self, path: &Path) -> Stage;
    pub fn signers(&self) -> &[SignerEntry];
    pub fn is_promoted(&self) -> bool;                          // has util entry
    pub fn signed_by(&self, pubkey_prefix: &str) -> bool;
    pub fn verify_all(&self) -> Vec<SignerVerification>;
}
```

### 10. CI integration — gates implementable in einmo

The commit / merge / tag gates are configurations of einmo commands wrapped in
tiny shell glue. Einmo provides the primitives; the gates are *configurations*,
not separate code.

**Commit gate (pre-commit hook):**
```bash
#!/bin/sh
cargo einmo compare output checked --work-dir . --require-match || {
  echo "einmo: output does not match checked. Promote (review) or repair."
  echo "  burden: the producer of the divergent output must repair or escalate."
  exit 1
}
```
`--require-match` = exit non-zero if any `differing`, `only_in_a`, or `only_in_b`.
This is the rule: agents must pass all checked einmos before committing.

**Merge gate (PR required status check):**
```yaml
# .github/workflows/einmo-gates.yml
name: einmo gates
on: [pull_request]
jobs:
  merge-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build -p einmo --release
      - run: ./target/release/cargo-einmo verify --work-dir . --all
      - run: ./target/release/cargo-einmo compare checked verified --work-dir . --require-match
```
`compare checked verified --require-match` = 0 differing, no `only_in_checked`
(new checks that need promotion). Enforces "merging requires all checked einmos
to be verified." A new check appears as `only_in_checked` and blocks merge until
a human runs `cargo einmo promote checked→verified --interactive` on the PR.

**Tag / release gate (pre-tag):**
```bash
#!/bin/sh
set -e
cargo einmo compare checked verified --work-dir . --require-match
cargo einmo confirm-signatures verified --pubkey-prefix "$RELEASE_KEY_PREFIX" --require-all
# both pass → git tag -s "$VERSION"
```
`confirm-signatures verified <prefix> --require-all` = every `verified/*.einmo`
carries a `util` entry whose pubkey starts with the release officer's key. An
AI-generated `util` (computer key, `util` pubkey == `test` pubkey) does not
match → tag blocked.

**Burden-of-correction rules (encoded in gate failure messages):**
- `compare output checked` fails → the producer of the divergent `output`
  repairs (fix code so output matches checked) or escalates (flag the checked
  file, promote new output to checked after review).
- `compare checked verified` fails → the producer of the `checked` version
  corrects (re-promote to match verified) or escalates. Attributable via the
  `util` signature chain.

### 11. Randomized re-inspection

**The problem:** even when `output==checked` (no new drift), reviewed baselines
can rot — a `checked` or `verified` file may encode a bug locked in at review
time. Kent Beck scores snapshots low on "Inspiring" (reviewers don't think
twice). Statistical re-inspection forces a random sample of already-promoted
files back through review. No surveyed framework does this.

**Specification:** `cargo einmo console-review <work_dir> <from>→<to>
--reexamine-rate <pct> [--reexamine-seed <seed>]` — in addition to demoting
files that genuinely differ, randomly sample `pct`% (default 10) of files
already in `<to>`, demote them back to `<from>` (move; `util` entry preserved),
and re-present them for review. `--reexamine-seed <seed>` pins the RNG for
reproducibility (CI can fix a seed per cycle). `--full` is shorthand for
`--reexamine-rate 10`. The re-examined file's re-promotion appends a **second
`util` entry** (the re-inspection is itself attributable). A re-examination that
*rejects* a file (flagging it) surfaces baseline rot — the burden of correction
falls on the process that originally produced the checked version.

### 12. CLI self-attestation (binary integrity check)

The `cargo-einmo` binary can perform an integrity check on **its own program
file**, adding provenance evidence to the attestation chain: alongside the
cryptographic signatures on `.einmo` content, the *tool that produced/verified*
the artifacts is itself identifiable.

```rust
let exe_path = env::current_exe()?;
let hash = sha256_of_file(&exe_path)?;
```

**`cargo einmo self-check [--expected <sha256>] [--quiet]`** computes the
SHA-256 of `env::current_exe()?`, prints the path and hash, and — if
`--expected <sha256>` is given (or a sidecar `cargo-einmo.sha256` ships next to
the binary, or a release-attestation file records it) — exits non-zero on
mismatch. Use cases:

- A release officer runs `cargo einmo self-check --expected <release-hash>`
  before `promote checked→verified`, confirming the signing binary is the
  audited build.
- CI runs `cargo einmo self-check --expected <pinned-hash>` as a gate before
  any `verify`/`compare`, so a tampered or substituted `cargo-einmo` cannot
  rubber-stamp a corrupted corpus.
- An auditor records the binary hash alongside a release attestation.

**Advisory `# produced-by:` line in `.einmo`.** When the test runner or a
promotion/flag operation writes a `.einmo`, it appends an **unsigned advisory
line** outside the signed content:

```
# produced-by: cargo-einmo <version> sha256:<binary-hash>
```

This line is **not** part of the canonical signed content (it is excluded from
`canon_input`/`canon_result`/`canon_comments`/`canon_metadata`, exactly like
the `# flagged:` advisory line) — so byte-steadiness is preserved: rebuilding
the binary changes the hash, but the existing `.einmo` signatures remain valid
(the advisory line is informational, not signed). The line provides
*attributable provenance* — "this artifact was produced by einmo binary X" —
queryable by `cargo einmo show <file>` and surfaced in the UI's signer-summary
panel, without entangling the binary's identity with the content's signature.

**Why advisory (unsigned), not signed:** signing the binary hash into the
content would make every `.einmo` signature binary-version-coupled — rebuilding
the tool would invalidate every existing signature, breaking byte-steadiness
and the verify-both-then-content-identical matching test across tool versions.
The binary's integrity is instead attested *out-of-band* via `self-check`
against a pinned/recorded hash, and its identity is recorded *advisory* in each
`.einmo`. The two together — signed content + advisory producer provenance +
out-of-band binary self-attestation — constitute "more evidence to the
attestation" without sacrificing reproducibility.

## Design preference — hierarchical granularity as a software-composition model

### The preference

Einmo's hierarchical storage is not merely organizational. The structure of the
`input/` tree — mirrored across all four stage directories — is intended to
**model the software's composition granularity**:

- **Deeper-nested `.einmo` files are unit-granularity.** A file at
  `input/operators/division/by_zero.test` pins the behavior of a small, leaf
  unit (analogous to a unit test).
- **Shallower-nested `.einmo` files are integration-granularity.** A file at
  `input/operators/division.test` or `input/operators.test` pulls multiple units
  together to produce a composite behavior (analogous to an integration test).
  The shallower file's behavior is *constituted from* the deeper files' behaviors.

When organizing a suite, structure the `input/` tree to mirror the software's
composition hierarchy: units at the leaves, integrations at shallower levels.

### The diagnostic property

**A higher-level mismatch implicates lower-level mismatches as its likely
cause.** When `compare output checked` flags a shallower-nested file as
`differing`, the root cause is *most likely* a `differing` file deeper in the
same subtree — the integration broke because one of its constituent units broke.

The debugging flow:
1. `compare output checked` reports `input/operators.test.einmo` as `differing`.
2. Descend: `compare output checked --filter operators/*` reports
   `input/operators/division.test.einmo` as `differing`.
3. Descend again: `compare output checked --filter operators/division/*` reports
   `input/operators/division/by_zero.test.einmo` as `differing`.
4. That deepest mismatch is the root cause. Fix the code so the deepest file
   matches; re-run; shallower mismatches resolve (cascade upward) — unless a
   shallower file has an independent issue, in which case it remains `differing`
   after the deeper fix and is itself the next root cause.

**Design preference (normative):** Einmo tooling and documentation should treat
the hierarchy as a dependency DAG. The `--root-cause` flag on `compare` (and
`console-review`) automatically descends a `differing` file's subtree and
reports the deepest `differing` descendants. The CLI/UI should present
mismatches *tree-ordered, deepest first*, so root causes are addressed before
symptoms.

Einmo is thus not just a test-instrumentation library — it is a **software-
granularity modeling tool**: the input/output directory hierarchy expresses how
the software composes, and the signed `.einmo` at each level pins the behavior
contract at that granularity. A team can instrument unit tests (leaves),
integration tests (mid-tree), and whole-program approval tests (root) under a
single signed-corpus regime, with a unified diagnostic that respects the
composition hierarchy.

## Web-UI / app-UI for the approval / promotion / alert process

### The principle: the UI is a frontend, never the core

The Rust `einmo` library is the **sole code path** that touches `.einmo` files,
derives keys, or verifies signatures. The UI is one of several frontends (CLI,
TUI, web, desktop) that call the library. The UI **never holds private keys**
and **never reads/writes `.einmo` files directly** — every operation goes
through the library, which enforces verify-on-inspect.

### The service layer: `einmo serve`

A Rust HTTP/WebSocket server (`cargo einmo serve`) wraps the library and
exposes its operations to any frontend. This is the single integration point for
all UIs — the CLI, a web SPA, a desktop app, and a TUI all talk to the same
server (or embed the library directly). The server:

- **Reads** `.einmo` files (via the library, which verifies-on-inspect) and
  returns parsed content + signer summaries + diffs — never raw file bytes that
  could be tampered with in transit.
- **Executes** promotion/flag/compare/verify operations by calling the library
  functions; enforces all invariants server-side.
- **Brokers passphrases**: the UI sends a passphrase over a local (loopback,
  auth-gated) channel; the server derives the key via Argon2id, signs, and
  discards the passphrase (never stores it). The private key exists only
  ephemerally in the server process.
- **Streams alerts** over WebSocket: real-time notification of `output≠checked`,
  `checked≠verified`, flagged files, staleness triggers, signature failures.

### Operations the UI surfaces

| UI operation | Library call | Who uses it |
|---|---|---|
| **Suite overview** — tree view of `input/` with per-file stage badges + signature status | `list_files` + per-file `stage()` + `verify_all()` | all actors |
| **Diff view** — side-by-side (e.g. output vs checked), signature lines hidden | `compare` + `DiffEntry` | reviewers |
| **Approve / promote** — promote output→checked (no passphrase) or checked→verified (prompts in-UI) | `promote(from, to, key)` | agents; humans |
| **Flag** — move file to flagged/ with reason | `flag(stage, filter, reason)` | any actor |
| **Verify all** | `verify(all)` | CI, reviewers |
| **Confirm-signatures** — filter by pubkey prefix | `confirm_signatures(path, prefix)` | release officer |
| **Console-review** — guided review with vimdiff/in-UI diff, @agent handling, randomized re-inspection | `console_review(from, to, opts)` | reviewers |
| **Signer inspection** — per-file signer chain | `signers()` | auditors |

### Alert process

| Alert | Trigger | Severity | Burden |
|---|---|---|---|
| **Output drift** | `compare output checked` ≠ 0 | error (blocks commit) | the agent that produced the divergent output repairs or escalates |
| **Unverified checked** | `checked` file with no corresponding `verified` | error (blocks merge) | a human must promote checked→verified, or the checked producer corrects/escalates |
| **Flagged file** | any file in `flagged/` | warning | the flagger resolves or regenerates |
| **Stale baseline** | `compare --stale-days N` flags an old file | warning | scheduled re-inspection picks it up |
| **Signature failure** | verify-on-inspect refuses a tampered file | critical | investigate via the signer chain |
| **Non-human util** | `confirm-signatures verified <release-key>` finds a `util` whose pubkey == `test` pubkey | critical | an AI agent bypassed the human gate; attributable to the computer key |

### Frontend forms (all talk to `einmo serve` or embed the library)

1. **CLI** (`cargo einmo …`) — primary frontend for agents and terminal-native humans.
2. **TUI** (optional, ratatui) — console dashboard; embeds the library directly.
3. **Web SPA** (static, served by `einmo serve`) — for distributed teams, PR-review dashboards.
4. **Desktop app** (optional, egui or Tauri) — offline review with native vimdiff; embeds the library.

### Crypto boundary (restated for the UI)

- The Ed25519 private key is derived from the passphrase **inside the Rust
  library/server process**, used once to sign, and dropped. The UI sends the
  passphrase over a local authenticated channel and never persists it.
- The UI never sees a private key, never signs directly, never reads/writes
  `.einmo` bytes. A compromised UI can at worst relay a wrong passphrase
  (producing a detectable bad signature) or surface misleading diffs — it cannot
  silently corrupt the signed corpus, because the library verifies-on-inspect.

## Technical Design

### 1. Architecture boundary — what's Rust, what's shell, what's Python

| Layer | Language | What it owns | Rationale |
|---|---|---|---|
| **Core** (`einmo` crate) | Rust | `.einmo` parse/serialize; Ed25519/Argon2id; FOOP-22 append-chain; verify-on-inspect; promote/flag/compare; passphrase cascade; `Evaluator`; `EinmoSuite`; randomized re-inspection sampler | Security + invariants must have one owner. The Rust core is the **sole code path** that touches `.einmo` files or derives keys. |
| **CLI** (`cargo-einmo`) | Rust | all verbs: promote/flag/compare/verify/confirm-signatures/show/console-review/serve | Thin wrapper over the core; the stable scriptable surface (`--json` on every verb). |
| **Git hooks / CI glue** | Shell | pre-commit, pre-tag scripts; GH Actions `run:` steps | 3–5 lines each, pure `cargo einmo …` invocation. |
| **Presentation (optional)** | Python (or Rust) | rich CI reports; optional `textual` TUI | Reads `cargo einmo compare --json`. Pure presentation — never touches files. |
| **UI frontends** | varies | web SPA / TUI / desktop app | Talk to `cargo einmo serve` (REST/WS) or embed the core. Never hold keys. |

**The discipline (invariant):** Shell, Python, and UI are **frontends** that
call the Rust core (subprocess CLI, HTTP to `serve`, or PyO3 in Proposal B).
They may **never** read/write/parse/serialize `.einmo`, derive keys, or
implement the passphrase cascade. Any logic that is an invariant lives in the
Rust core and is *invoked, not re-implemented*.

### 2. Proposals

#### Proposal A — Rust monolith + embedded `serve` (recommended)

Everything that touches `.einmo` or keys is Rust; shell is git-hook/CI glue
only; the web UI is `cargo einmo serve` (axum) inside the same binary.
`console-review` (vimdiff + diff -I + randomized re-inspection) is Rust — the
demote-random-sample is an invariant (`→flagged` move) and must not live outside
the core. UI: static SPA (SvelteKit/Vite+React) served by `cargo einmo serve`
via `rust-embed`; browser calls REST; passphrase sent over loopback POST.
**Binding:** subprocess CLI (`--json`) is the only non-Rust binding; UI is
in-process HTTP. **Pros:** single invariant owner; single release artifact; no
FFI/ABI coupling; matches Rust-first project. **Cons:** `console-review` TUI
orchestration in Rust is verbose (~150 lines). **Migration:** Medium.

#### Proposal B — Rust core + PyO3 Python orchestration

Rust owns crypto+format; Python owns ergonomics — CI reporting, review-queue
UX, a `rich`/`textual` TUI, FastAPI web service. PyO3 binds in-process.
**Binding:** PyO3 for Python; subprocess CLI for shell/CI; HTTP for UI. **Pros:**
Python's `rich`/`textual` make review UX nicer; PyO3 avoids subprocess overhead
for batch. **Cons (concrete):** two code paths to the core (CLI + PyO3) —
invariants must be enforced *in the core*, not the CLI verb, or PyO3 bypasses
them; PyO3 couples Python to Rust ABI (struct layout changes force wheel
rebuild); Python becomes a required toolchain (contradicts Rust-first); a
Python `serve` re-implementing the passphrase cascade is the one realistic way
to silently break the emergent human-attestation property. **Migration:** Medium-Large.

#### Proposal C — Rust core + WASM-verify-in-browser + signing server

Ship the *verify* path to the browser as WASM (no keys, read-only); a thin Rust
signing server (`cargo einmo serve`) holds keys and is the only mutation path.
`einmo` crate factored: `einmo::verify` (no-key, WASM-targetable) + `einmo::sign`
(full). **Binding:** subprocess CLI for shell/CI; HTTP for UI mutations; WASM
for UI verification (same Rust code recompiled). **Pros:** verify-on-inspect
enforced *in the browser* (strongest trust story); decouples read from write;
`einmo::verify` being WASM-clean forces an auditable boundary. **Cons:** verify/
sign factoring is real architectural work; two compile targets (native +
wasm32); must use `ed25519-dalek`+`argon2` (not `ring`) for WASM compat.
**Migration:** Large.

#### Proposal D — Minimal CLI-only, defer all UI (variant)

Ship the core + CLI only. No `serve`, no UI, no Python. The `--json` interface
on every verb is the integration point; a future UI (A or C) plugs in later.
**Pros:** smallest shippable surface; fastest to land the invariants and gates.
**Cons:** review UX is CLI-only; no web dashboard until later. **Migration:**
Medium (core only). **When to choose:** if the priority is locking down the
signed-corpus invariants and gates before investing in UI.

### 3. Recommendation

**Ship Proposal A**, with two deliberate hooks:

1. **Structure `einmo::verify` as a clean submodule** with no
   filesystem/tty/Argon2id dependency — costs ~nothing now and keeps Proposal C
   (browser-side WASM verify) available as a *future* enhancement without a rewrite.
2. **Expose `--json` on every verb from day one** — this is what shell, CI, a
   future Python reporting layer, and a future UI all consume. The CLI contract
   is the stable surface, not the Rust ABI.

**Why A over B:** the project is Rust-first; the invariants must have one owner;
PyO3 buys negligible ergonomics for commands that mostly call back into Rust
anyway; the provenance ecosystem (sigstore-rs, in-toto-rs, cargo-deny, gittuf)
has converged on A's shape. Projects with parallel Python+Rust impls (in-toto)
treat it as a cost. **Why A over C now:** C's value (browser-side verify) is a
UI-quality win, not an invariant win; A's clean `verify` submodule keeps C cheap
*when* the UI arrives. **Why A over D:** A includes `serve` for ~zero extra
complexity and unblocks the UI whenever wanted.

### 4. Prior art (self-contained — no searches needed to implement)

The dominant pattern in the Rust provenance ecosystem is "library + CLI in one
crate family, shell/CI as the only scripting, any web UI as a separate thin
client calling the same library":

- **sigstore-rs / cosign:** Rust crate `sigstore-rs` is a library + thin CLI.
  Cosign (Go) is CLI-first; the web UI (Rekor search) is a separate service
  talking to the same library. Mirrors Proposal A/C, not B.
- **in-toto (`in-toto-rs`):** Rust crate + CLI, no separate scripting layer. The
  Python reference impl is a *parallel implementation*, not an orchestration
  layer — and the project has paid the cost of keeping two impls in sync. Lesson:
  don't have two implementations of the spec. Validates A/C over B.
- **cargo-deny:** Pure Rust crate + CLI + library. No UI, no Python. CI consumes
  via subprocess. Validates A.
- **gittuf (TUF-for-git, Rust):** Rust crate + CLI; gittuf-github is a separate
  Go service for GitHub app integration. Closest to Proposal C's split.
- **rebuilderd:** Rust daemon + CLI; web UI is a separate minimal service
  querying the daemon.

**Proposal B (PyO3 orchestration) has essentially no precedent in this space.**

### 5. Migration order (do not start UI until the core passes its own gates)

1. Migrate `signature.rs`: REPLACE → FOOP-22 append-chain. **Write tamper/
   forgery tests first** — this is the single highest-risk step.
2. Generalise `snapshot_suite.rs`: `Vec<FirRef>` → `Vec<String>`; move FIR→String
   formatting into the `UbcEvaluator`/`UbcaEvaluator`/`UbcbEvaluator` adapters.
3. Restore `verify_signatures.rs` (fix the `.gitignore` `bin/` bug — narrow to
   `/bin/` or `target/**/bin/`).
4. Write the `.snap` → `.einmo` converter (`einmo::migrate`); run over the
   ~289-file corpus; human re-sign pass (wall-clock time).
5. Implement `compare` with the formal per-section matching semantics (§5);
   implement the gates (§10).
6. **Einmo's own CI uses its own gates** (commit: `compare output checked`;
   merge: `compare checked verified`) — the library eats its own dog food before
   any UI lands.
7. Only then: `cargo einmo serve` + SPA (Proposal A's UI layer).

## FIR Impact

None. This FOOP changes test infrastructure only. The `SequenceableFir` /
`HumanizingSequencer` / `FirQueryable` machinery (FOOP-02, FOOP-42) is
unchanged; Einmo consumes its `String` output opaquely via the generalised
`Evaluator` trait (returns `Vec<String>`, no `FirRef` dependency).

## UBC Step Impact

None. FOOP-62's `catch_unwind` panic-capture contract in the snapshot harness is
preserved — Einmo's test runner continues to catch panics and write
`PANIC: <msg>` into the output before signing, so a panicking evaluation still
produces a signed, reviewable `.einmo` in `output/`.

## Test-tier configurations

Each tier maps to an `EinmoSuite` `TestConfig` with a specific
`require_correspondence` set and gate. Grounded in Foolish's actual test
structure: unit tests incl. `*_nyes_transitions` in `foolish-ubca/src/fir_kinds.rs`
(88 inline tests, 16 nyes-transitions), approval/snapshot suites via
`SnapshotSuite` in `foolish-core`/`foolish-ubca`/`foolish-ubcb`
`*_snapshot_tester.rs`, parser unit tests in `foolish-parser`. **No Rust CI
exists today** (only Java/Scala Maven cross-validation); **no CLI/trycmd tier**;
**no bench tier**.

| Tier | Inputs | Evaluator | Stages | `require_correspondence` | Gate |
|---|---|---|---|---|---|
| **1. Unit (inlined)** | inlined strings (`evaluate_inline`) | function under test | `output`, `checked` | `[(Output, Checked)]` | commit: `output==checked` |
| **2. Approval / snapshot (VM gate)** | `.foo` in `input/` (hierarchical) | UBC/UBCb VM + humanizing sequencer | full pipeline | CI: `[(Output, Checked)]`; release: `+[(Checked, Verified)]` | commit: `output==checked`; merge: `checked==verified` |
| **3. Integration** | `.foo` exercising parser+VM+sequencer | end-to-end | full pipeline | same as approval | same; `--stale-days 30` |
| **4. CI (automated)** | re-runs tier 2/3 | same | `output`, `checked` | `[(Output, Checked)]` + `verify --all` | push/PR: verify + compare |
| **5. Release / deployment** | re-runs 2/3 | same | `verified` | `[(Checked, Verified)]` + `confirm-signatures verified <release-key> --require-all` | tag gate |
| **6. Regression / re-inspection** | existing `input/` | same | demotes `verified→checked` sample | ad-hoc | scheduled: `console-review checked→verified --reexamine-rate 10 --reexamine-seed $WEEK` |
| **7. Performance (optional)** | `.foo` + timing | timing→formatted string | `output`, `checked` | `[(Output, Checked)]` | commit; redact volatile values before signing |

**Note on JVM removal (FOOP-03):** Per **FOOP-03** (deprecating JVM
implementations), the cross-validation tier (Java/Scala/Rust output comparison)
is **dropped from Einmo's scope**. The current `.github/workflows/tests.yml`
"Cross Validation" workflow (Java/Scala/Maven only, no Rust) is not migrated
into Einmo. Einmo's scope is the Rust implementation's signed-snapshot lifecycle
only. If JVM implementations are un-deprecated later, the per-impl stage-dir
model (`output-rust/`, `output-java/`, etc. with cross-`compare`) is the
structure that would host cross-validation — out of scope today.

### Per-tier `TestConfig` example (approval, CI profile)

```rust
let config = TestConfig::default("foolish-ubca/snapshot_tests")
    .require_correspondence(Stage::Output, Stage::Checked);   // CI gate
let suite = EinmoSuite::new(config);
let results = suite.evaluate_all(num_cpus::get(), &UbcaEvaluator::new());
assert!(results.all_written_and_correspondence_holds());
```

## Idealized development flow

### The actors

- **Coding agents** (many, parallel) — write code on feature branches, run
  tests, generate `output/`, promote `output→checked`, commit.
- **Human reviewers** — review diffs, run `console-review`, promote
  `checked→verified` (interactive passphrase), merge.
- **Release officer** (human) — holds the release passphrase; promotes/tags.
- **CI** — automated `verify` + `compare`; never holds a passphrase.

### The flow

1. An agent starts a feature branch, writes code, runs `cargo test`. Einmo
   regenerates `output/*.einmo` (computer-signed, timestamped). `output/` is
   git-tracked, so the diff shows exactly which behaviors changed.
2. The agent reconciles `output` vs `checked` (`einmo compare output checked`):
   - 0 differing → proceed to commit.
   - differing > 0 → the agent has the **burden to repair or escalate**: repair
     (fix code so output matches checked), promote (review and
     `output→checked`), or escalate (flag and surface to human).
3. The agent commits. Pre-commit hook runs `einmo compare output checked
   --require-match`; blocked unless `output==checked`.
4. The agent pushes and opens a PR. CI runs `einmo verify --all` (Layer 1) and
   `einmo compare output checked --require-match` (Layer 2).
5. A human reviews: `einmo console-review checked→verified --interactive`
   (passphrase not in repo config → prompt appears; human types it). Promotes
   accepted baselines (appends `util`). Rejected → `einmo flag checked --reason …`.
6. Merge gate: `einmo compare checked verified --require-match` = 0 differing.
   New checks (promoted `output→checked` but not `checked→verified`) block merge
   until a human verifies them.
7. `checked ≠ verified` at merge → the producer of the `checked` version has the
   burden: re-promote or flag-and-escalate. Attributable via the `util` chain.
8. Release: `einmo compare checked verified` + `einmo confirm-signatures
   verified <release-key> --require-all`. Both pass → tag.
9. Continuous re-inspection: weekly cron `einmo console-review checked→verified
   --reexamine-rate 10 --reexamine-seed $WEEK`. A rejection surfaces baseline
   rot; re-promotion appends a second `util` entry.

### The invariant this flow enforces

No code change reaches `main` without: (a) its outputs matching a reviewed
`checked` baseline, AND (b) that baseline carrying a human `util` signature in
`verified/`. The signature chain attributes every state to an actor. An AI agent
that bypasses the human gate (`--passphrase ""`) produces a `util` under the
computer key — `confirm-signatures verified <release-key>` catches it.

## Rejected Alternatives

### A. Replace-on-promotion (current `insta`/`verify_signatures` behaviour)

`verify_signatures --write-verified` currently overwrites the footer with the
human key, destroying the computer attestation. Rejected: violates "generator
always signs" and provides no chain of attestation. FOOP-22 already proposes
append; this FOOP adopts it.

### B. Conflate review and promotion (insta/Verify/Jest model)

Use `insta`'s `cargo insta review` accept as the only transition. Rejected:
eliminates the `Checked` intermediate state, which is the explicit requirement
(a human marks "acceptable" without immediately committing the human signature).

### C. External signature sidecar files (`.einmo.sig`)

Rejected: decouples the signature from the artifact (sidecar files get lost),
complicates the workflow, and breaks the "snapshot file is self-describing"
property. The in-file `SIGNATURES:` block keeps everything co-located.

### D. Fork insta upstream

Rejected for now: `insta`'s file-write path is not interceptable (hardcoded
`fs::write` in `Snapshot::save`), so a fork would diverge permanently. Einmo
writes `.einmo` files directly (does not depend on `insta` for output). A fork
remains a future option if needed.

### E. Do nothing (leave it in shell scripts + AGENTS.md prose)

Rejected: the lifecycle is currently untyped, untestable, and worktree-local. A
typed library makes the invariants enforceable in code and CI.

## Resolved Decisions

- **OQ-1 (crate vs module): RESOLVED — crate `einmo`.** Reusable, publishable.
- **OQ-2 (CI gate): RESOLVED — correspondence model, no separate
  require-human-promotion gate.** Layer 1 (always-on `verify` on all branches);
  Layer 2 (commit gate `compare output checked`); Layer 3 (merge gate `compare
  checked verified`); tag gate (`confirm-signatures verified <release-key>`).
  The human-attestation property is emergent (the repo deliberately omits a
  verified-stage passphrase), not a CI config flag.
- **OQ-3 (staleness): RESOLVED — `compare --stale-days N`, warn-only.**
- **OQ-4 (timestamps): RESOLVED — both generation and promotion timestamps are
  SIGNED (inside content).** Re-signing produces different bytes; acceptable.
  Flag reason is unsigned advisory.
- **OQ-5 (structured flag log): RESOLVED — flag annotation is an in-file
  advisory `# flagged:` line, not a separate log.** Dissolves the JSON-lines
  vs free-text question.
- **OQ-6 (inline snapshots): RESOLVED — expected results never inlined; inputs
  MAY be inlined via `evaluate_inline`.** `Evaluator` generalised to
  `Vec<String>`.
- **OQ-7 (plan file): RESOLVED — yes.** `FOOP-92.plan.md` follows.
- **Flagged stage added:** any state → `flagged` (terminal sink, move semantics,
  collision → timestamp suffix). Resolves the stage-count question (four
  stages: output/checked/flagged/verified).
- **No automated update:** Einmo writes `output/` directly; no `INSTA_UPDATE`
  equivalent; no `insta` dependency for output.
- **Per-section matching:** `compare` matches INPUT+RESULT (required) +
  COMMENTS (optional); both files independently verify first.
- **Randomized re-inspection:** first-class feature (`--reexamine-rate`,
  `--reexamine-seed`).

## Open Questions

- **Desktop app framework (Tauri vs egui).** Deferred — both reuse `cargo einmo
  serve`'s REST API unchanged; nothing to decide now.
- **WASM verify in browser (Proposal C).** Deferred — `einmo::verify` clean
  submodule keeps this available as a future enhancement.
- **Per-stage passphrase config.** Whether `einmo.toml` should support
  `[signing.<stage>]` per-stage overrides (beyond the deployment convention of
  omitting the verified passphrase). Deferred — the cascade + convention
  suffices for the initial implementation.

## References

- Prior FOOPs:
  - **FOOP-12** — Signature scheme (Ed25519/Argon2id, canonicalization,
    dual-signing, `verify_signatures`). The cryptographic foundation. Status: Final.
  - **FOOP-22** — Multi-signer append format (`test`/`util` roles, "Entire
    file" integrity). Adopted as canonical. Status: Draft.
  - **FOOP-02** — `SnapshotSuite` current home in `foolish-core`, generalised
    over `Evaluator`. This FOOP moves it into `einmo`. Status: Draft.
  - **FOOP-42** — Humanizing FIR Sequencer (HFS) output byte format
    (`hfssnap`). The signed body must conform. Status: Draft.
  - **FOOP-62** — UBCa; snapshots are the hard acceptance gate; `catch_unwind`
    panic-capture contract. Status: Brewing.
  - **FOOP-21** — Alarms emitted into snapshot output. Status: Brewing.
  - **FOOP-03** — Deprecating JVM implementations. Cross-validation tier
    dropped from Einmo scope. Status: (see FOOP-03).
- External docs (verified mid-2026; full citation in Appendix D):
  - insta (v1.48.0, commit `7f23d2e`) — https://insta.rs/docs/, https://docs.rs/insta
  - in-toto Attestation Framework — https://github.com/in-toto/attestation
  - sigstore-rs — https://github.com/sigstore/sigstore-rs
  - SLSA attestation model — https://slsa.dev/attestation-model
  - GitHub Artifact Attestations — https://github.com/actions/attest
  - jlevy/tbd Golden Sessions — https://github.com/jlevy/tbd/blob/main/packages/tbd/docs/guidelines/golden-testing-guidelines.md
  - insta issue #792 (TOFU/immutable snapshots, OPEN) — https://github.com/mitsuhiko/insta/issues/792
  - insta PR #815 (non-interactive review for LLMs/CI, shipped 1.44) — https://github.com/mitsuhiko/insta/pull/815
- Code locations (pre-extraction, verified mid-2026):
  - `foolish/foolish-core/src/signature.rs` (644 lines, FOOP-12; currently
    REPLACE-based single-signer — migrate to FOOP-22 append)
  - `foolish/foolish-core/src/snapshot_suite.rs` (258 lines, FOOP-02;
    `Evaluator` returns `Vec<FirRef>` — generalise to `Vec<String>`)
  - `foolish/foolish-core/src/bin/verify_signatures.rs` — **MISSING** (dangling
    `[[bin]]` in `Cargo.toml`; gitignored by root `bin/` pattern — fix before
    creating `einmo/src/bin/`)
  - `foolish/foolish-core/src/ubc_snapshot_tester.rs`,
    `foolish/foolish-ubca/src/ubca_snapshot_tester.rs`,
    `foolish/foolish-ubcb/src/ubcb_snapshot_tester.rs` — `Evaluator` adapters
  - `foolish_review.sh` (98 lines), `accept_approved.sh` (53 lines) —
    worktree-local; replaced by `cargo einmo` subcommands
  - `foolish/.claude/settings.json` — contains `INSTA_UPDATE=always` in allow
    list (contradicts AGENTS.md; moot under Einmo which has no automated update)
  - ~289 committed `.snap`/`.snap.new` files (140 core + 145 ubca + 4 ubcb) to
    migrate to `.einmo`

---

# Appendix A — Insta facts (verified, self-contained)

*(Verified mid-2026 against github.com/mitsuhiko/insta, docs.rs, crates.io.)*

## A.1 Insta version and source

- insta **v1.48.0** is the current latest release (as of 2026-07-01). Commit
  `7f23d2e` ("Release 1.48.0 (#925)") is a valid permalink target.

## A.2 Insta GitHub issues (real; descriptions corrected)

| Issue | Title | Verified content |
|---|---|---|
| [#478](https://github.com/mitsuhiko/insta/issues/478) | `INSTA_UPDATE=always cargo test` with an incorrect inline snapshot passes | `INSTA_UPDATE=always` returns zero exit code despite incorrect snapshot. Matches "exits 0 even on failure." |
| [#527](https://github.com/mitsuhiko/insta/issues/527) | Show diffs on `cargo insta test`? | `cargo insta test` shows no diff, only `info: 1 snapshot to review`. |
| [#659](https://github.com/mitsuhiko/insta/issues/659) | Does anyone use `unseen`? | `--accept-unseen` pending deprecation (maintainer proposed deprecating). |
| [#865](https://github.com/mitsuhiko/insta/issues/865) | Snapshot not fixed by accepting it | Inline snapshot can loop: `--accept` "fixes" but `cargo test` still fails (leading-whitespace/`trim` normalization). Closed by PR #866. |
| [#792](https://github.com/mitsuhiko/insta/issues/792) | TOFU/Immutable snapshots | OPEN, opened 2025-08-15 by jalil-salame. Proposes immutability after first generation — *not* signing or attestation. A maintainer expressed reluctance ("-0.1 without more demand signals"). Materially narrower than this FOOP. |
| [#117](https://github.com/mitsuhiko/insta/issues/117) | Whitespace issues with inline snapshots | Reports whitespace-roundtrip bugs with inline snapshots. NOTE: the phrase "basically intractable" is the FOOP author's editorial, not a quote from the issue. |
| [#313](https://github.com/mitsuhiko/insta/issues/313) | Better ways of dealing with snapshots in loops or helper functions | Snapshots in loops/helper fns collide. NOTE: `set_snapshot_suffix` is the general insta mechanism for parameterized snapshots, but is *not mentioned in #313* — it is this FOOP's suggestion, not the issue's. |

## A.3 Insta PR #815

[PR #815](https://github.com/mitsuhiko/insta/pull/815) "Add LLM-friendly
non-interactive snapshot management", merged 2025-11-20 by max-sixty. Adds
non-interactive `cargo insta review --snapshot <path>` and `reject --snapshot
<path>` for non-TTY (LLMs/CI). Shipped in insta 1.44.0.

## A.4 Insta Settings API (corrected)

**`Settings::bind_dynamic` does NOT exist.** The real binding APIs are:

- `bind(|| …)` — sync closure.
- `bind_async` — future.
- `bind_to_scope()` — returns a `SettingsBindDropGuard` (RAII drop-guard, recommended).

The "dynamic" redaction tool is `dynamic_redaction` (used via
`add_dynamic_redaction`). `Settings` are thread-bound at runtime ("Settings are
always bound to a thread") BUT **`Settings` is `Send + Sync`** (docs.rs Auto
Trait Implementations list `impl Send for Settings` and `impl Sync for
Settings`). The real caveats are `!RefUnwindSafe` / `!UnwindSafe`. The
thread-locality is a runtime binding, not a `!Send` constraint. (A prior draft of
this FOOP incorrectly claimed `Settings` is `!Send` — corrected.)

## A.5 Insta file-write path is NOT interceptable

`Snapshot::save`/`save_new` hardcode `fs::write`. There is no hook between
"passed comparison" and "bytes hit disk." This is why Einmo writes `.einmo`
files directly rather than wrapping insta's write path.

---

# Appendix B — Cross-language survey (condensed)

The genre has four names, one mechanism: **characterization test** (Feathers),
**approval test** (Falco), **golden master** (record industry), **snapshot
test** (Jest). The universal lifecycle: `generate → review → approve → commit →
(on change) re-review`. The philosophical core: a machine can only verify "the
received output equals the approved file" — it can never verify "this output is
the behaviour we want." The genre *trades the oracle problem for a review
problem*.

| Ecosystem | Tool | Storage | Review→Promote model |
|---|---|---|---|
| JS/TS | Jest `toMatchSnapshot` | `__snapshots__/*.snap` | `-u` overwrites; **1 state, no review gate** |
| JS/TS | Vitest | `__snapshots__/*.snap` | `-u`; refuses to write in CI; obsolete snapshots fail |
| Python | Syrupy | `__snapshots__/*.ambr` | `--snapshot-update`; fails if snapshot missing |
| Rust | insta | `.snap`/`.snap.new` | `cargo insta review` TUI; `--check` in CI; **2 states, review == promotion** |
| Rust | expect-test | inline `expect!["…"]` | `UPDATE_EXPECT=1` |
| Rust | trycmd | `.stdout`/`.stderr` | `TRYCMD=overwrite`; elision for volatile substrings |
| Ruby | ApprovalTests.Ruby | `.received`/`.approved` | `approvals verify --ask` |
| Go | cupaloy | `testdata/*.golden` | `-update`; updating *fails* the test to block CI auto-update |
| JVM | ApprovalTests.Java | `.received`/`.approved` | Pluggable Reporters; Scrubbers |
| Swift | SnapshotTesting | `.snap`/images | `record` mode |

**Common anti-patterns (universal):** snapshotting too much; auto-accepting in
CI ("defeats the entire point" — every ecosystem concluded this independently);
snapshots masking semantic bugs; non-deterministic output; large diffs in code
review; stale/orphaned snapshots. **Common best practices:** redact/mask
volatile values; structured (YAML/JSON) snapshots; split large snapshots; CI
gates rejecting pending/unreviewed snapshots; dedicated review tool (not just
env var); idempotency checks; floating-point tolerances.

**None cryptographically signs snapshot files** (the gap Einmo fills).

---

# Appendix C — Approval testing theory (corrected)

- **Michael Feathers, *Working Effectively with Legacy Code* (2004):** defines
  characterization tests as documenting actual behaviour ("document your system's
  actual behavior, not check for the behavior you wish your system had"). The
  phrase "essentially a change detector" is from the characterization-test
  literature (Wikipedia-derived), not Feathers' own wording.
- **Llewellyn Falco, ApprovalTests (Oct 2008):** originated the approval-test
  genre. His original tagline was **"a picture is worth a 1000 tests"** — *not*
  "I'll know it when I see it" (the latter is a later community paraphrase of
  Justice Potter Stewart; the "I know exactly what I want!" post is by Dan
  Gilkerson, not Falco).
- **Emily Bache (2020):** argues "golden master" is misleading and should die;
  she endorses the likening "pouring concrete on your software."
- **Kent Beck, 12 Test Desiderata:** snapshots score ★★★★★ on **Behavioural**
  and **Writable**. Beck gives **no star rating** to "Inspiring" or "Structure-
  insensitive" — he uses prose ("I wouldn't choose to rely only on snapshot
  tests" under Inspiring; "depends…" under Structure-insensitive). A prior draft
  fabricated ★ ratings for those two; corrected. Verdict: measured endorsement.

**The Jest controversy (2016→)** is almost entirely a frontend/UI phenomenon.
The compiler/VM community treats golden files as uncontroversial — compiler
output is deterministic, fully-specified, and reviewable by domain experts.

---

# Appendix D — Novelty & prior-art analysis (corrected, self-contained)

## D.1 No mainstream framework signs test snapshots

Verified across: Jest, Vitest, Syrupy, ApprovalTests (JS/Python/Ruby/Java/Go),
Swift SnapshotTesting, cupaloy, go-snaps, expect-test, trycmd, pytest-regtest,
and the Mochi/naga/Pharo VM golden-file systems. **None cryptographically signs
snapshot files**, nor gates acceptance behind a human passphrase with an audit
trail distinguishing AI-generated from human-reviewed output. The closest
upstream recognition is insta issue #792 (immutability, not signing).

## D.2 No Rust crate bridges snapshot testing + signing

Searches across crates.io, docs.rs, and GitHub for (snapshot/golden/approval) +
(sign/attest/cryptographic) returned only pure snapshot libraries (insta,
expect-test, insta-cmd) and pure signing libraries (sigstore-rs,
in_toto_attestation). No crate combines them.

## D.3 No framework separates review from promotion

All surveyed frameworks conflate review and promotion into a single `accept`
step. Einmo's 4-stage model (output/checked/flagged/verified) has no precedent.

## D.4 Supply-chain attestation theory — the conceptual basis

- **SLSA (slsa.dev/attestation-model):** provenance as "an attestation that a
  particular build platform produced a set of software artifacts"; signing
  authenticates *who created the attestation*.
- **in-toto Attestation Framework (github.com/in-toto/attestation):** data model
  = Statement (Subject + PredicateType + Predicate) + DSSE Envelope (payload +
  signatures). Rust crate `in_toto_attestation` **v0.1.0** is real (crates.io,
  mid-2026).
- **sigstore-rs (github.com/sigstore/sigstore-rs):** README "Key Interface"
  lists generate/sign/verify/export-import PEM-DER. Known limitation: "The crate
  does not handle verification of attestations yet." (Caveat: README heading is
  "Key Interface"; the concrete Rust trait identifier may not be literally
  `KeyInterface`.)
- **GitHub Artifact Attestations (github.com/actions/attest):** Sigstore-powered,
  in-toto predicate, keyless via OIDC, build-provenance granularity (not
  per-test-snapshot).
- **jlevy/tbd "Golden Sessions" (golden-testing-guidelines.md, 819 lines):**
  "Transparent box testing"; YAML session capture; `MOCK_MODE` env (mocked/live);
  stable/unstable field classification. Quote (American spelling): "Session
  files are behavioral specifications. They deserve the same review **rigor** as
  code." Closest spiritual prior art — but no cryptographic signing, no
  "reviewed but not promoted" state, MOCK_MODE is about determinism not
  attestation.

## D.5 Adjacent AI-provenance context (2024–2026)

- **insta PR #815** — non-interactive review for LLMs/CI. insta adapting to AI
  agents as review actors — but with **no identity, no signing**. Einmo fills
  that gap.
- **DigiCert, "The New Trust Architecture for AI"** (white paper, published
  2026-04-30) — cryptographic identity/authorization/integrity for AI
  agents/models/content (uses C2PA).
- **CSA Agentic Trust Framework** (published 2026-02-02; spec at
  github.com/massivescale-ai/agentic-trust-framework) — Zero-Trust governance
  for AI agents.
- **Alexander Zanfir, "Who Signed This? Provenance for AI Agents"** (Medium,
  published **2025-11-29** — not 2026) — chain proving human-approved rule →
  agent suggested → human reviewed.
- **C2PA** (spec.c2pa.org) — provenance metadata for AI-generated media.

## D.6 Verdict

The use case (VM output snapshots) is the genre's canonical sweet spot. The
signing layer is genuinely novel but a faithful SLSA/in-toto transfer, directly
resolving the #1 critique (review fatigue / rubber-stamping). The AI-vs-human
key distinction is well-motivated for 2026. **Where the over-engineering risk
lives:** signing is pointless if human review is itself a rubber stamp; the
vimdiff step is load-bearing, the signature is the audit trail. Snapshot size
discipline still applies (Beck's "Readable"/"Structure-insensitive" desiderata
are unsolved by signing).

---

# Appendix E — Suggestions surviving under Einmo

Most insta-specific items (e.g. `INSTA_UPDATE=always` in settings, insta
opt-level, `.gitattributes` for `.snap`) are **moot under Einmo** (Einmo does
not depend on insta for output). The following survive:

1. **Redact/mask volatile values** *before* the signed body — so signatures are
   naturally stable across runs. Filter timestamps/IDs/paths before the
   `INPUT`/`RESULT` blocks are canonicalized and signed.
2. **Use `dynamic_redaction`-style validate-then-mask** — assert a field *is* a
   valid UUID/timestamp *before* replacing it, so a regression to
   empty/garbage fails the test instead of being silently redacted.
3. **Structured (YAML/JSON) snapshots** for complex output, not raw debug dumps.
4. **Split large snapshots**; snapshot only what matters; review size limits.
5. **Consider `trycmd`** for CLI output snapshots (`foolish-cli run/step/repl`)
   — purpose-built for CLI textual interaction with elision for volatile
   substrings. Complements Einmo (values) rather than competing.
6. **Pair every approval test with a focused unit test on invariants** (already
   done with `*_nyes_transitions` — keep it). Beck's desiderata: snapshots are
   low on "Inspiring"; real assertions force dual-verification.

---

# Appendix F — Codebase state (supplemental, verified mid-2026)

## F.1 Test-bearing files (the real tiers)

- `foolish-core/src/`: `fir.rs` (22 tests), `signature.rs` (27 tests),
  `sequencer_tests.rs` (21), `unit_tests.rs` (14), `ubc_snapshot_tester.rs` (1:
  `approval_all`).
- `foolish-ubca/src/`: `fir_kinds.rs` (88 tests, incl. all 16
  `*_nyes_transitions`), `fir_trait.rs` (30, incl.
  `step_leaf_through_nyes_transitions`), `proto_brane.rs` (5), `nyes_ext.rs` (5),
  `ubca_snapshot_tester.rs` (1).
- `foolish-ubcb/src/`: `unit_tests.rs` (19), `fir.rs` (10), `channel.rs` (6),
  `luid.rs` (3), `ubcb_snapshot_tester.rs` (2: `approval_all`,
  `approval_all_states`).
- `foolish-parser/src/`: `parser.rs` (14), `lexer.rs` (8).

No `tests/` integration dirs. No CLI/trycmd tests. No benchmarks (`#[bench]`/
criterion). The `*_nyes_transitions` tests live in `foolish-ubca/src/fir_kinds.rs`
(inline `#[cfg(test)] mod tests`); AGENTS.md mandates extending them when FIR
kinds/NYES states change.

## F.2 Snapshot directories

Three crates have `snapshot_tests/{input,approved}/`:
- `foolish-core/snapshot_tests/approved/` — `.snap` (100+), `.foo` inputs; no `.snap.new`.
- `foolish-ubca/snapshot_tests/approved/` — `.snap` (100+); stray `.snap.approved`, editor `.swp/.swo`.
- `foolish-ubcb/snapshot_tests/approved/` — 4 `.snap` + 4 `.snap.new` (pending human review).

~289 committed `.snap`/`.snap.new` files total to migrate to `.einmo`.

## F.3 CI (no Rust)

`.github/workflows/tests.yml` ("Cross Validation") runs Java/Scala/Maven only.
**No `cargo test` in any workflow.** `foolish-crossvalidation` is a Maven/Java
module, not a Rust crate. Einmo's CI gates are greenfield (Rust).

## F.4 Test-harness helpers (to migrate into `einmo`)

- `foolish-core/src/snapshot_suite.rs` — the shared `SnapshotSuite` +
  `Evaluator` trait (currently `Vec<FirRef>`).
- `foolish-core/src/ubc_snapshot_tester.rs`,
  `foolish-ubca/src/ubca_snapshot_tester.rs`,
  `foolish-ubcb/src/ubcb_snapshot_tester.rs` — `Evaluator` adapters
  (`UbcEvaluator`/`UbcaEvaluator`/`UbcbEvaluator`).
- `foolish-core/src/signature.rs` — Ed25519/Argon2id (currently REPLACE-based).
- `foolish-core/Cargo.toml` — dangling `[[bin]] verify_signatures` (file missing).
- `foolish_review.sh` (98 lines), `accept_approved.sh` (53 lines) — worktree-local.

Insta workspace dep: `insta = { version = "1", features = ["yaml"] }`; dev-dep in
foolish-core, foolish-ubca, foolish-ubcb, foolish-parser. (Einmo drops the insta
dependency for output generation.)

## Last Updated

**Date**: 2026-07-02
**Updated By**: Sisyphus / z-ai/glm-5.2
**Changes**: Added **Use Case C — Provide inspection service** (AI-agent
inspection via MCP + AGENTS.md skills; human inspection via `einmo serve` web
service with rich HTML output rendering, einmo-in-HTML metadata convention,
approve/disapprove buttons, and the **byte-steadiness invariant** — signed
content is canonical/deterministic; interactive search/analysis is a
non-mutating view layer over the signed bytes). Restructured all three use
cases (A, B, C) to lead with a **What-if** vision subsection, then a **So, then**
v1-scope subsection, then the design subsections progressing toward code.
Use Case A what-if: parameterized inputs, cross-impl compare, tolerances,
redaction library, dependency-aware selection, fuzz-minimisation,
time-travel. Use Case B what-if: threshold multi-sig, key rotation/delegation,
HSM, air-gapped signing, per-agent identity, signature-graph viz, cross-repo
attestation, time-locks. Use Case C what-if: MCP tools, encoded review-flow
skills, agent self-attribution, auto-bisection, rich human inspection,
collaborative review, cross-repo dashboards.
