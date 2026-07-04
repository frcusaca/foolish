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

**Einmo** is a **standalone** workspace crate (`einmo`) providing directory-
based signed-snapshot testing with a four-stage promotion pipeline: **output**
→ **checked** → **flagged** / **verified**. Each test suite is configured with a
work directory containing `input/` (test triggers) and four stage directories,
each holding `.einmo` files. The stage directory tree **mirrors** the `input/`
tree at any depth. Generated outputs are timestamped and signed by the test
runner. Promotion from `output->checked` is a CLI operation available to AI
agents or humans (no passphrase). Promotion from `*->verified` appends a human-
keyed signature (passphrase-resolved; defaults to interactive prompt). Any
state may transition to `flagged/` (a terminal sink). The library supports
comparing any two stages; the comparison is per-section (INPUT + OUTPUT
required, COMMENTS optionally required), with both files independently verified
before content is compared.

**Standalone scope (critical).** Einmo reimplements the full signed-snapshot
machinery from scratch — it does **not** depend on, modify, or migrate any
existing crate (`foolish-core`, `foolish-ubca`, etc.). The existing
`foolish-core/src/signature.rs` and `foolish-core/src/snapshot_suite.rs` stay
untouched; they are a *design reference*, not a dependency. The existing `.snap`
corpus stays as-is; migrating it to `.einmo` is a future, separate effort.
Einmo is structured so it can be **promoted to its own repository** later
without dragging Foolish-specific dependencies along.

**Companion test crate — zweimomo.** Einmo is exercised by a second new
workspace crate, **`zweimomo`**, which embeds three **pure-Rust** interpreters
as `Evaluator` implementations (Foolish via `foolish-ubca`, Python via
`rustpython-vm`, JavaScript via `boa_engine`) and writes parallel test inputs
in all three languages. Zweimomo is both einmo's first real test suite and the
proof that einmo's `Evaluator` trait is language-agnostic. The three
interpreters were chosen because they are **pure-Rust implementations** — no
C/FFI toolchain required, keeping the whole test harness portable and
einmo-repo-promotable.

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
   invoked by AI agents (`output->checked`) or humans (`*->verified`, passphrase).
   The CLI determines where to get the signing key — from keyboard console, an
   external API, or test code.

### The world after this FOOP

- A **standalone** reusable crate `einmo` with directory-based, hierarchical
  test configuration. Einmo reimplements the signed-snapshot machinery from
  scratch (no dependency on `foolish-core`/`foolish-ubca`); it is promotable to
  its own repository.
- A **companion test crate `zweimomo`** embedding three pure-Rust interpreters
  (foolish-ubca, rustpython-vm, boa_engine) as `Evaluator` impls, with parallel
  test inputs in Foolish, Python, and JavaScript.
- Each test's work directory: `input/` → `output/*.einmo` → `checked/*.einmo` →
  `flagged/*.einmo` / `verified/*.einmo`, all mirroring the `input/` tree.
- Generated outputs carry a `test` signer entry with a signed generation timestamp.
- `einmo promote output->checked` (AI/human, no passphrase).
- `einmo promote checked->verified` (human, passphrase; defaults to interactive).
- `einmo flag <stage>` (any state → flagged, terminal sink).
- `einmo compare <stage-a> <stage-b>` — stage-agnostic, per-section comparison.
- CI gates on stage correspondence, not just existence.
- The existing `insta`-based `.snap` corpus in `foolish-core`/`foolish-ubca` is
  **untouched**; migration to `.einmo` is a future, separate effort once einmo
  is stable.

## Product Description

**Einmo** is a **standalone** workspace crate (`einmo`) providing directory-
based, cryptographically signed snapshot testing with a staged promotion
pipeline. It is built on two capabilities that, together, no surveyed framework
provides:

1. **A programmatic test-construction API** that gives test code first-class
   access to every stage of verification — so a test can assert "output matches
   checked" (CI gate), "checked matches verified" (release gate), or "every
   verified file is signed by key `eb108…`" (release attestation), all against
   signed, tamper-evident artifacts.
2. **A CLI (`einmo …`) that manages stage-wise promotion with
   cryptographic signing at each transition** — promotion is a separate,
   attributable act, not an automated `accept`. Every generated output is
   timestamped and signed by the test runner; every promotion to `verified`
   appends a human-keyed signature.

**Standalone scope.** Einmo owns its own copy of the signing/format machinery
(`ed25519-dalek`, `argon2`, etc.); it does not depend on `foolish-core`. It is
exercised by **`zweimomo`**, a companion crate that embeds three pure-Rust
interpreters (Foolish via `foolish-ubca`, Python via `rustpython-vm`, JavaScript
via `boa_engine`) as `Evaluator` impls, proving the trait is language-agnostic.

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

Signing uses the three-role key model of §4.4: a **Compiled Key** (embedded at
compile time), a **Configured Key** (set at configuration time), and per-stage
**Stage Keys**. Generation writes the Compiled + Configured certification
stamps plus the `stage:output` stamp; every promotion **appends** the
destination stage's stamp.

| Stage | Directory | Stamps present | Who can produce | Stage-key source |
|---|---|---|---|---|
| **Output** | `output/` | `compiled`, `configured`, `stage:output` (signed generation timestamp) | test runner (always) | configured; default empty-passphrase key |
| **Checked** | `checked/` | + `stage:checked` (signed promotion timestamp) | AI agent or human via CLI (`promote output->checked`) | configured; default empty-passphrase key |
| **Flagged** | `flagged/` | unchanged + advisory `# flagged:` line | any state → flagged via CLI | none (no stamp) |
| **Verified** | `verified/` | + `stage:verified` (signed promotion timestamp) | human via CLI (`promote *->verified`) | resolved via cascade; deliberately unconfigured → interactive prompt |

**Critical invariants (typed, enforced in code):**

1. A file cannot enter `output/` without the full generation chain — `compiled`
   + `configured` + `stage:output` stamps. Generation always signs.
2. Existing stamps are never modified or removed by any transition; every
   promotion appends exactly one destination-stage stamp.
3. Flagging = **move** (origin vacated, `flagged/` populated) + advisory line;
   no stamp; collisions get a timestamp suffix.
4. **Verify-on-inspect**: any operation that reads a `.einmo` file verifies *all*
   stamps first (certifications + every stage stamp's prior-bytes signature);
   tampered files are refused, never operated on.

## Use Case A — Constructing tests: human-readable input → human-readable output

### A.1 What-if — the vision

Imagine a test framework where the test author never writes an expected value —
they write an input and an evaluator, and the framework captures the behaviour
as a signed, reviewable artifact. Now imagine extending that:

- **Parameterized / generated inputs.** A test generates inputs from a seed
  (property-based style) and writes one `.einmo` per generated case — pinning
  the VM's behaviour across a fuzz space.
- **Cross-implementation comparison.** The same `input/` tree evaluated by two
  `Evaluator`s into `output-a/` and `output-b/`, with a
  `compare output-a output-b` gate. (The Rust-vs-Rust path is structurally
  available; `foolish-ubcb` was removed by FOOP-03, but the per-language
  stage-dir model is exactly what `zweimomo` exercises — see §Use Case D.)
- **Tolerance-based matching.** `compare` with a numeric tolerance for
  floating-point OUTPUT sections (pytest-regtest style), not just byte-identity.
- **A redaction library.** Built-in redactors for common volatile values
  (timestamps, UUIDs, memory addresses, run-ids) that normalise the OUTPUT
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
  matching: INPUT+OUTPUT required, COMMENTS optional).
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
    pub match_sections: MatchSections, // default InputOutput
    pub encoding: String,             // default "utf-8" (§4.1)
    pub separator: String,            // default "①\n"; Foolish suites "!!\n" (§4.1)
    pub perspectives: Vec<Perspective>, // statically configured views (§4.5)
    pub parallel: Option<usize>,      // None = serial; Some(n) = n threads
}
pub enum Stage { Output, Checked, Flagged, Verified }
```

### A.5 What a test module looks like

```rust
#[test]
fn foolish_suite_approval_all() {
    let config = TestConfig::default("zweimomo/suites/foolish")
        .require_correspondence(Stage::Output, Stage::Checked);   // CI gate
    let suite = EinmoSuite::new(config);
    let evaluator = UbcaEvaluatorAdapter;   // adapts FIR → Vec<String> (§D.3)
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
- Ed25519 via Argon2id key derivation (FOOP-12-style); the **three-role key
  model** (§4.4): Compiled Key + Configured Key certify public keys, per-stage
  Stage Keys sign all prior bytes in an append chain (generalising FOOP-22's
  append format).
- Stage-key cascade (5 tiers: `--passphrase` > `--stdin-passphrase` >
  `EINMO_PASSPHRASE` env > `einmo.toml` `[signing.<stage>]` > interactive
  `/dev/tty` prompt); `--interactive` forces the prompt.
- `confirm-signatures <path> <pubkey-prefix> [--require-all]` for release-key
  attestation.
- **Emergent human-attestation**: the repo deliberately omits a verified-stage
  key, so `*->verified` falls through to the interactive prompt. An AI piping
  `--passphrase ""` produces a `stage:verified` stamp under the well-known
  empty-passphrase key — post-hoc detectable by pubkey.
- Signed generation + promotion timestamps (inside each stage stamp).

Explicitly **out of scope** for v1: threshold multi-sig, key rotation /
delegation, HSM / hardware-token integration, air-gapped offline signing,
per-agent cryptographic identity (all agents share the computer key),
signature-graph visualisation (beyond `einmo show`'s signer summary), signed
test-selection / coverage attestation, cross-repo release attestation,
time-locked signatures. Documented as future enhancements.

### B.3 Timestamping

| Event | Where it lives | Signed? |
|---|---|---|
| Generation (output written) | metadata `generated:` **and** inside the `stage:output` stamp | **Yes** — the stamp's own field, covered by later stamps' prior-bytes signatures |
| Each promotion | inside the destination stage's stamp (`timestamp` field) | **Yes** — covered by any subsequent stamp |
| Flag reason + time | advisory `# flagged: <reason> <ISO8601>` line outside signed content | No (advisory; original stamps stay valid without re-signing) |

Signing timestamps means every run/promotion produces different signature
bytes. **Accepted tradeoff (BDFL decision 2026-07-03):** the resulting
git-diff churn in `output/` is tolerated for now; redesign only if it becomes
a real problem in practice. Tamper-evidence is the load-bearing property;
byte-reproducible re-signing is not.

### B.4 Signing — keys and algorithms

- **Algorithm**: Ed25519; passphrase-derived keys use Argon2id (FOOP-12-style
  derivation with parameters pinned by einmo).
- **Key roles** (full definition in §4.4): **Compiled Key** (embedded at
  compile time — secret in custom builds, public knowledge in the stock
  open-source build), **Configured Key** (configuration time), **Stage Keys**
  (one per stage; each stage may have a different configured key). Compiled
  and Configured stamps certify public keys; Stage stamps sign all prior file
  bytes and **append**, stage after stage.
- **Emergent human-attestation** (not enforced): the deployment deliberately
  does **not** configure a `verified`-stage key, so `*->verified` falls
  through the cascade to the interactive prompt. A human types the
  passphrase; an AI agent running non-interactively with no tty cannot
  complete the promotion. If an agent pipes `--passphrase ""`, the
  `stage:verified` stamp's pubkey equals the well-known empty-passphrase key —
  **post-hoc detectable**. The signature provides attribution; the policy
  provides the gate.

### B.5 Key sources — stage-key resolution cascade

Uniform cascade for resolving the **stage key** in any signing operation:

| Tier | Source | Notes |
|---|---|---|
| 1 | `--passphrase <value>` | explicit, non-interactive (CI/scripted) |
| 2 | `--stdin-passphrase` | read one line from stdin (pipe or tty) |
| 3 | `EINMO_PASSPHRASE` env var | per-process override |
| 4 | `einmo.toml` `[signing.<stage>] passphrase = "…"` | per-repo, per-stage default (output/checked typically `""`) |
| 5 | **interactive prompt** on `/dev/tty` | only if no tier above yielded a value |

- An explicit empty string (`--passphrase ""` or `EINMO_PASSPHRASE=""`) is **set
  to empty** (the well-known empty-passphrase key), not "unset" — to unset,
  omit the flag/var entirely.
- `--interactive` flag: **forces the prompt**, skipping tiers 1–4.
- The **Configured Key** comes from `einmo.toml` `[signing] configured-key`
  (or programmatic `TestConfig`); the **Compiled Key** is baked into the
  binary at build time (stock builds embed the published default).
- Per-stage deployment convention: `einmo.toml` sets output/checked stage
  passphrases to `""` and deliberately does **not** set a verified-stage
  passphrase, so `*->verified` falls through to the interactive prompt — the
  human-attestation gate is emergent from configuration, not a code-enforced
  rule.

### B.6 The CLI (`einmo …`)

**Einmo is a single CLI app** — one CLI surface named `einmo`; everything is a
subcommand (`einmo verify-signatures …`, not a separate binary). Installed via
`cargo install einmo`; the `cargo-einmo` alias binary makes `cargo einmo …`
work identically (§1). **Every
subcommand verifies all stamps on any file it touches before proceeding**
(verify-on-inspect); tampered files are refused. Stage-pair arguments use the
ASCII arrow `->`; stage names match `[A-Za-z0-9_-]+` (no `>`, no other
punctuation). Operations that walk many files run parallel or serial per the
`--parallel <n>`/config setting.

```bash
# Promotion & flagging
einmo promote <from>-><to> <work_dir> [--filter <glob>] [--passphrase <v> | --stdin-passphrase | --interactive] [--batch]
  # legal: output->checked, output->verified, checked->verified,
  #        output->flagged, checked->flagged, verified->flagged
  # every promotion appends the destination stage's stamp
  # *->verified : warns if the stamp key equals a well-known computer key
  # *->flagged  : move (same as `flag`)
  # --batch     : one passphrase prompt for all matching files

einmo flag <work_dir> <stage> [--filter <glob>] [--reason <text>]
  # moves <stage>/<rel> → flagged/<rel>; collision → timestamp suffix
  # appends advisory "# flagged: <reason> <ISO8601>"

# Comparison & verification (CI)
einmo compare <stage-a> <stage-b> <work_dir> [--match-sections input,output] [--require-comments-match] [--stale-days N] [--filter <glob>] [--require-match] [--json]
einmo verify <work_dir> [--stage <s> | --all]

# Signature inspection
einmo confirm-signatures <path> <pubkey-prefix> [--require-all]
einmo verify-signatures <path> [--write-verified] [--stdin-passphrase]

# Inspection
einmo show <file>

# Review (replaces foolish_review.sh)
einmo console-review <work_dir> <from>-><to> [--filter <glob>] [--full] [--reexamine-rate <pct>] [--reexamine-seed <seed>] [--vim | --list] [--root-cause]

# UI server (Proposal A; post-MVP)
einmo serve <work_dir> [--bind <addr>]

# Self-attestation (integrity check on the CLI's own binary)
einmo self-check [--expected <sha256>] [--quiet]
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
  root-cause bisection. An agent reviews `output->checked` by calling tools,
  not by shelling out and parsing text.
- **AGENTS.md skills as encoded review flows.** The standard review protocols
  (reconcile output vs checked; decide repair/promote/escalate; the
  burden-of-correction; flag-vs-escalate decision) are written as reusable
  skills — step-by-step instructions an agent loads and follows, so review
  discipline is not reinvented per agent.
- **Agent self-attribution.** Each agent signs with its own derived key, so
  stamps are attributable not just to "a machine" but to "agent
  X" — post-hoc attribution becomes per-actor.
- **Agent-driven auto-bisection.** On a mismatch, the agent descends the
  granularity tree (`--root-cause`) automatically, finding the deepest
  differing leaf without human steering.
- **Rich human inspection.** For outputs that are themselves rich — an HTML
  report, an SVG diagram, a structured data dump — the `.einmo` OUTPUT block
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
- An **MCP server** (`einmo serve --mcp`, or a dedicated `einmo-mcp`
  binary) exposing: `list`, `diff`, `promote`, `flag`, `verify`,
  `confirm_signatures`, `root_cause`, `show` as structured tools. The MCP
  server calls the einmo library — it is a frontend, never touches `.einmo`
  files directly. (Agents that prefer shells use the `einmo … --json`
  CLI; both call the same library.)
- An **AGENTS.md documentation template + skills** encoding the standard review
  flows (reconcile output vs checked; burden-of-correction; flag-vs-escalate).
  Shipped as part of the einmo crate's docs so adopting projects drop them into
  their own `AGENTS.md`.
- **`einmo serve`** (axum, REST/WebSocket): suite overview tree, per-section
  diff view, promote/flag/verify/confirm-signatures/show endpoints, and a
  WebSocket alert feed (output≠checked, checked≠verified, flagged, staleness,
  signature failures).
- **Rich-output rendering.** The OUTPUT block may contain HTML (or other
  browser-renderable content). The SPA fetches the `.einmo` via `/api/show`
  (the backend verifies-on-inspect server-side), extracts the OUTPUT, and
  renders it. An **einmo-in-HTML metadata convention** carries the einmo
  metadata inside the HTML (a `<script type="application/json"
  id="einmo-meta">{…}</script>` block) so the SPA renders stage/signature chrome
  + approve/disapprove buttons around the content. The buttons POST to
  `/api/promote` or `/api/flag`.
- **Byte-steadiness invariant** (critical, see C.4): the signed OUTPUT content
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
| `einmo_promote(work_dir, from, to, filter, passphrase?)` | `promote` | promote (passphrase only for `*->verified`) |
| `einmo_flag(work_dir, stage, filter, reason)` | `flag` | move to flagged/ |
| `einmo_verify(work_dir, stage?)` | `verify` | signature integrity |
| `einmo_confirm_signatures(path, prefix, require_all)` | `confirm_signatures` | release-key attestation |
| `einmo_root_cause(work_dir, a, b, rel_path)` | `compare --root-cause` | descend subtree; deepest differing descendants |
| `einmo_show(file)` | `EinmoFile::from_file` + `stamps` | verified content + stamp-chain summary |

Agents that prefer shells use `einmo … --json`; both paths call the same
library, so invariants are identical.

**AGENTS.md skills (documentation template).** Einmo ships a documented set of
review-flow skills that adopting projects drop into their `AGENTS.md`. Each
skill is a step-by-step protocol:

- **Reconcile output vs checked** (the agent's pre-commit flow): run
  `compare output checked`; for each `differing`/`only_in_a` file, decide
  repair (fix code so output matches checked), promote (review the diff and
  `promote output->checked`), or escalate (`flag` with a reason and surface to
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

**Rich output rendering.** The `.einmo` OUTPUT block may contain HTML (or SVG,
or other browser-renderable content), not just plain text. The SPA fetches the
`.einmo` via `/api/show`; the backend verifies all signatures server-side
(verify-on-inspect) and returns the parsed content; the SPA renders the OUTPUT
HTML in a sandboxed container with the einmo chrome around it.

**Einmo-in-HTML metadata convention.** When the output IS HTML, it carries an
embedded metadata block so the SPA can render stage/signature chrome and
approve/disapprove buttons without a separate API round-trip per render:

```html
<!-- the signed HTML output, byte-steady -->
<script type="application/json" id="einmo-meta">
{"input_path":"stage1/section3/specific.test","stage":"output",
 "stamps":[{"key":"stage:output","pubkey":"dc5f586c…"}]}
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

1. **Canonicalise before signing.** The OUTPUT content (HTML or otherwise) is
   canonicalised to a deterministic byte string before the `Result`/`Metadata`
   signatures are computed. No volatile values (timestamps, random IDs,
   run-order-dependent serialisation) in the signed bytes — redact/mask them
   before canonicalisation (the test author's responsibility in v1; a redaction
   library is a future enhancement, Use Case A §A.1).
2. **The signature covers the canonical bytes.** Each stage stamp's signature
   covers all file bytes before it — metadata, INPUT, every OUTPUT and
   perspective section, COMMENTS, and all earlier stamps. Any change to the
   canonical HTML invalidates the stamp — tamper-evident.
3. **Interactive features are a view layer.** The HTML's client-side JS
   (search, fold, highlight, data-structure exploration) renders/transforms the
   signed bytes for display. It never writes back to the `.einmo` file; it
   never mutates what is signed. The signed bytes are immutable in the file;
   the UI is a projector over them.
4. **External resources are not signed.** If the HTML embeds external resources
   (images, scripts fetched at view time), those are NOT part of the signed
   OUTPUT bytes — only the HTML text in the OUTPUT block is signed. For full
   attestation of external resources, they would need their own signing (out
   of scope v1; documented as a future enhancement).
5. **Verify-on-inspect applies.** Before the SPA renders the OUTPUT, the
   backend loads the `.einmo` via `EinmoFile::from_file` (which verifies all
   signatures); a tampered file is refused and surfaced as an alert, never
   rendered as if valid.

This invariant is what makes rich, interactive, searchable output compatible
with cryptographic signing: the signed payload is byte-steady; the
interactivity is a non-mutating view layer over it.

## Use Case D — Zweimomo: cross-language validation harness

Zweimomo is einmo's **companion test crate**. It proves einmo's `Evaluator`
trait is language-agnostic by embedding three interpreters, producing parallel
test inputs in three languages, and running each through einmo's signed-snapshot
pipeline. Zweimomo is both einmo's first real test suite (einmo eats its own dog
food) and the design pressure that keeps the `Evaluator` trait honest.

### D.1 Why three interpreters, and why these three

Zweimomo embeds exactly three interpreters, all **pure-Rust implementations**:

| Language | Interpreter crate | Version | Notes |
|---|---|---|---|
| **Foolish** | `foolish-ubca` (workspace) | — | the project's own VM; the reference interpreter |
| **Python** | `rustpython-vm` | 0.5.0 | pure-Rust CPython reimplementation; no CPython linkage |
| **JavaScript** | `boa_engine` | 0.21.1 | pure-Rust ECMAScript; no V8/QuickJS/SpiderMonkey |

**Rationale (note in the FOOP):** these three were chosen because they are
**pure-Rust implementations of interpreters** — no C/FFI toolchain, no system
library linkage, no `*-sys` build dependency. This keeps the entire test
harness (`einmo` + `zweimomo`) compilable with `rustc` alone, which is the
load-bearing property for **promoting einmo to its own repository** later
without dragging a C toolchain requirement into the consumer's build. A
non-pure-Rust interpreter (e.g. one binding CPython or V8) would couple the
test harness to a system C library and break repo portability. Lua (`mlua`)
and Rune were considered and rejected for this reason — `mlua`'s default
backend links LuaJIT (C), and while Rune is pure-Rust, RustPython + Boa cover
two of the most widely-known languages (Python, JavaScript), making the
parallel-input matrix more legible to reviewers.

### D.2 The `Evaluator` contract each interpreter satisfies

Einmo's trait (no `FirRef` dependency):

```rust
pub trait Evaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String>;
}
```

Each interpreter is wrapped by an `Evaluator` impl in zweimomo that:
1. Takes a source string in that interpreter's language.
2. Evaluates it (catching panics/errors → `Err(String)`; einmo records errors
   in the envelope's `status`/`status-detail` metadata, §4.2).
3. Stringifies the result(s) → `Vec<String>` (one string per top-level result;
   einmo writes one OUTPUT section per string).

**Serialization is zweimomo's responsibility.** Einmo is language-agnostic —
it takes chunks of input/output text and never interprets them. How each
interpreter's values are rendered into those text chunks (which mode to
evaluate in, how to stringify results, what an "output" even is per language)
is designed and owned by the test crate. The rule: **use what is most
colloquial in each language** — idiomatic evaluation and idiomatic
stringification per interpreter, specified per adapter in zweimomo, not
homogenised by einmo.

### D.3 The three `Evaluator` impls (embedding sketches)

**`UbcaEvaluatorAdapter`** (wraps the existing `foolish-ubca::UbcaEvaluator`):

```rust
use foolish_ubca::UbcaEvaluator;   // existing; NOT modified
use foolish_core::FirSequencer;     // for FIR→String formatting
use einmo::Evaluator;

pub struct UbcaEvaluatorAdapter;
impl Evaluator for UbcaEvaluatorAdapter {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        let inner = UbcaEvaluator;
        let firs = inner.evaluate(source)?;          // Vec<FirRef> (foolish-core type)
        Ok(firs.iter()
            .map(|fir| FirSequencer::format(fir))    // FIR → hfssnap String
            .collect())
    }
}
```
*(zweimomo depends on `foolish-ubca` + `foolish-core` for this adapter; einmo
itself does not. The existing `UbcaEvaluator` and `FirSequencer` are used
as-is, never modified.)*

**`RustPythonEvaluator`** (`rustpython-vm` 0.5.0):

```rust
use rustpython_vm::{Interpreter, eval};
use einmo::Evaluator;

pub struct RustPythonEvaluator { /* settings */ }
impl Evaluator for RustPythonEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        let interp = Interpreter::without_stdlib(Default::default());
        interp.enter(|vm| {
            let scope = vm.new_scope_with_builtins();
            let result = eval::eval(vm, source, scope, "<zweimomo>")
                .map_err(|e| e.to_string())?;        // PyResult<PyObjectRef>
            Ok(vec![result.str(vm)?.as_str().to_string()])
        }).map_err(|e| e.to_string())
    }
}
```
*(Pure-Rust: `rustpython-vm` has no `cc`/`cmake`/`*-sys` deps under default
features; `ssl-rustls` is the default SSL backend, not `ssl-openssl`.
`Interpreter` is not `Send` — keep it thread-local or on a dedicated thread.
`without_stdlib` gives a sandbox: no `os`/`sys`/file I/O unless `init_stdlib()`
is called — a sandboxing plus for a test harness.)*

**`BoaEvaluator`** (`boa_engine` 0.21.1):

```rust
use boa_engine::{Context, Source};
use einmo::Evaluator;

pub struct BoaEvaluator { /* settings */ }
impl Evaluator for BoaEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        let mut context = Context::default();
        let result = context.eval(Source::from_bytes(source))
            .map_err(|e| e.to_string())?;            // JsResult<JsValue>
        let s = result.to_string(&mut context)
            .map_err(|e| e.to_string())?
            .to_std_string_escaped();
        Ok(vec![s])
    }
}
```
*(Pure-Rust: `boa_engine` has no `cc`/`cmake`/`*-sys` deps. No `fs`/`network`/
`child_process`/Node APIs by default — a sandboxed ECMAScript core. `Context`
is not `Send` — same thread-local/dedicated-thread advice as RustPython.)*

### D.4 Parallel test-input matrix

For each **concept** row, zweimomo writes the equivalent input in all three
languages, evaluates each via its `Evaluator`, and captures a signed `.einmo`
per language. The matrix is **bounded by what foolish-ubca can do today**
(Foolish is the least capable interpreter; Python and JS are far more capable,
so Foolish sets the ceiling). See Appendix G for the full Foolish capability
envelope.

| Concept | Foolish input | Python input | JS input | Foolish-capable? |
|---|---|---|---|---|
| **Integer arithmetic** | `{2 + 3 * 4 - 5;}` | `2 + 3 * 4 - 5` | `2 + 3 * 4 - 5` | ✅ (int-only; `/` is integer division) |
| **Nested expressions / parsing** | `{((2 + 3) * (4 - 1)) / 5;}` | `((2 + 3) * (4 - 1)) // 5` | `Math.floor(((2+3)*(4-1))/5)` | ✅ |
| **Name binding + scope** | `{x = 42; y = x + 8; y;}` | `x = 42; y = x + 8; y` | `let x = 42; let y = x + 8; y` | ✅ (Foolish's strength; forward refs work) |
| **Data structures / nesting** | `{a = 10; b = 20; n = {inner = a + b;}; n.inner;}` | `{"a":10,"b":20,"n":{"inner":30}}["n"]["inner"]` | `({a:10,b:20,n:{inner:30}}).n.inner` | ✅ (brane = object/dict) |
| **Function application** | `{fn = {result = a+b;}; r =$ {a=10,b=-3} fn}` | `def fn(a,b): return a+b\nfn(10,-3)` | `function fn(a,b){return a+b} fn(10,-3)` | ✅ (see D.5) |
| **Division-by-zero / error** | `{10 / 0;}` | `10 / 0` (→ ZeroDivisionError) | `10 / 0` (→ `Infinity`) | ✅ (Foolish → NK + alarm) |
| **Search/query (Foolish-specific)** | `{d = {x=10;y=20}; d?y;}` | `d["y"]` | `d.y` | ✅ Foolish; asymmetric (search vs dict-access) |
| **SF/SFF laziness (Foolish-specific)** | `{a=1,b=2; c=<<a+b>>; c;}` | *(no equivalent; skip)* | *(no equivalent; skip)* | ✅ Foolish-only |

**Concepts deliberately excluded** (no Foolish representation today; see
Appendix G): string operations (no string type), floats (int-only), booleans,
recursion, loops/iteration, `if/then/else` (parsed but rejected at compile
time), closures/higher-order functions. These may appear in the Python/JS
columns as **Foolish-unsupported** markers (the test documents the asymmetry
rather than forcing a fake parallel).

### D.5 Function application in Foolish (the emergent calling semantics)

Foolish has no `fn(args)` syntax and no `FnDef`/`CallExpr` AST node. Function
application is **emergent from concatenation + tail extraction**:

- **Concatenation**: juxtaposing two branes merges them — `{args} fn` makes
  `args`'s ordinates visible inside `fn`.
- **Tail (`$`)**: `x$` yields the last value of `x`.
- **Bind-tail (`=$`)**: `a =$ b` ≡ `a = b$` — "bind the value of the last
  statement of `b` to the name `a`."

So a "function call" is:

```foolish
{
    fn = {part1 = a+b; part2 = a-b; result = part1+part2;};
    call_result =$ {a=10, b=-3} fn
}
```

`{a=10, b=-3} fn` concatenates the argument brane with `fn` (so `a` and `b`
resolve inside `fn`), and `=$` binds `fn`'s tail (`result`) to `call_result`.
The output is correct when `call_result` is correct. This is confirmed by
existing snap inputs (e.g. `regression_disappearing_brane_statements.foo` uses
`d =$ 4`).

The Python/JS equivalents use ordinary call syntax (`fn(10, -3)`), so the
**function application** concept row is parallel across all three languages —
the *mechanism* differs (structural vs syntactic) but the *contract* (bind
arguments, evaluate, yield a result) is identical.

### D.6 Use existing snaps for inspiration

When designing the parallel-input corpus, **use the existing `.foo` snap inputs
under `foolish-ubca/snapshot_tests/input/` for inspiration and syntax guidance**
(~145 inputs demonstrating arithmetic, branes, search, SF/SFF, alarms, unicode
identifiers, etc.). The Foolish column of the matrix should mirror idioms
already proven in that corpus; the Python/JS columns are then written to match.
This keeps the Foolish inputs realistic (within the interpreter's demonstrated
capability) rather than speculative.

### D.7 Zweimomo's scope (v1) and what's out

**In scope (v1):**
- Three `Evaluator` impls (`UbcaEvaluatorAdapter`, `RustPythonEvaluator`,
  `BoaEvaluator`).
- Parallel test-input corpus for the concept rows in §D.4.
- One `EinmoSuite` per language, each with its own `input/` → `output/` →
  `checked/` tree, gated by `compare output checked`.
- Tests written using einmo asserting per-language correspondence.

**In scope (later in development — planned as a dedicated plan phase):**
- **Exhaustive algorithm coverage.** Port algorithm implementations
  exhaustively from [TheAlgorithms](https://github.com/TheAlgorithms) (and
  other well-known collections) into the zweimomo corpus — sorting, searching,
  math, dynamic programming, string and graph algorithms. TheAlgorithms carries
  parallel Python and JavaScript implementations directly; Foolish equivalents
  are written where the language allows (Appendix G bounds this). The goal is
  to **test the test framework as thoroughly as possible**: einmo must be
  exercised at realistic corpus scale (hundreds of inputs, deep hierarchical
  trees, batch promotion, `--filter` and `--root-cause` under load), not just
  by a handful of hand-picked examples. See the plan's algorithm-coverage
  phase.

**Out of scope (v1):**
- Cross-language correspondence (e.g. "Foolish output == Python output for the
  same concept") — the three languages produce *different* output formats
  (hfssnap vs Python `repr` vs JS `toString`), so byte-identity across languages
  is not meaningful. Each language is gated independently.
- Per-agent cryptographic identity in `test` entries (agents share the computer
  key — that is Use Case B's v1 scope).
- Fuzzing / property-based input generation (Use Case A §A.1 future).

## Specification

### 1. Crate structure

The library is a **standalone** workspace member **`einmo`** (path `einmo/` at
the workspace root). It does **not** depend on `foolish-core` or any other
existing crate; it reimplements the signing/format machinery from scratch. It is
structured so it can be promoted to its own repository later.

```
einmo/
├── Cargo.toml             # standalone: ed25519-dalek, argon2, base64, hex, clap, serde, serde_json, toml, time, thiserror
└── src/
    ├── lib.rs             # re-exports
    ├── main.rs            # the `einmo` CLI binary: promote, flag, compare, verify, verify-signatures, confirm-signatures, show, console-review, serve, self-check
    ├── bin/
    │   └── cargo_einmo.rs # one-line alias binary `cargo-einmo` → same CLI (enables `cargo einmo …`)
    ├── config.rs          # TestConfig, StageDirs, MatchSections, Perspective, key/cascade resolution
    ├── stage.rs           # Stage enum + directory operations + transitions
    ├── compare.rs         # per-section stage-to-stage comparison
    ├── format.rs          # .einmo envelope parse/serialize (§4: header line, separator, sections, STAMPS)
    ├── signature.rs       # Compiled/Configured/Stage key model; Ed25519 + Argon2id; stamp create/verify
    ├── snapshot_suite.rs  # EinmoSuite + generalised Evaluator trait (implemented from scratch)
    └── verify.rs          # verify-on-inspect + verify-all (clean submodule; no fs/tty/argon2)
```

**Single CLI app, cargo-installable.** Einmo is one CLI surface; every
operation — including signature verification (`einmo verify-signatures …`) —
is a subcommand of the one app. The crate is published as `einmo` so users run
**`cargo install einmo`**, which installs two binary targets sharing the same
clap parser: `einmo` (canonical) and `cargo-einmo` (a one-line alias whose
name follows the cargo-subcommand convention, so **`cargo einmo …`** also
works). This FOOP writes all examples in the canonical `einmo …` form.

**`.gitignore` fix (blocking, still required):** the root `.gitignore` has a
`bin/` pattern that ignores *any* `bin/` directory including `src/bin/` — the
new `einmo/src/bin/cargo_einmo.rs` alias would be silently ignored. Narrow it
to `/bin/` (repo-root `bin/` only) in Phase 0, *before* creating the crate.
(Note: `foolish-core/src/bin/verify_signatures.rs` *does* exist and is
git-tracked despite the pattern — already-tracked files are unaffected.) A
`.gitattributes` entry `*.einmo -text` is added at the same time so git
eol-normalization can never corrupt signed bytes.

**No `migrate.rs` (scope change).** This FOOP does **not** migrate the existing
`.snap` corpus. Einmo writes its own `.einmo` files from day one via its
`Evaluator`-driven test runner. Migrating the legacy `insta` `.snap` corpus is a
future, separate effort (see §Deferred).

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
    Output,    // Generated by test runner. compiled + configured + stage:output stamps.
    Checked,   // Reviewed — promoted from output by AI agent or human. stage:checked stamp appended.
    Flagged,   // Set aside — any state → flagged via CLI. Move (origin vacated). Advisory `# flagged:` line. No stamp. Terminal sink.
    Verified,  // Promoted from checked (or output) by human with passphrase. stage:verified stamp appended.
}
```

**Transitions** (every promotion appends the destination stage's stamp over
all prior bytes; existing stamps are never touched):
- `output->checked`: copy + append `stage:checked` stamp (checked stage key —
  configured, no prompt).
- `output->verified`, `checked->verified`: copy + append `stage:verified`
  stamp (verified stage key — typically the interactive human passphrase).
- `output->flagged`, `checked->flagged`, `verified->flagged`: move (remove
  from origin, create in `flagged/`); advisory line only, no stamp.
- `console-review` demotion: `verified->checked` (move; all stamps preserved
  as history; re-promotion appends another `stage:verified` stamp).

**Flagged collision handling:** if `flagged/<rel>` already exists when a new
file is flagged to the same path, the new file gets a timestamp suffix. The
common case (one flag per path) keeps the clean mirror; collisions disambiguate.
The flag reason + origin stage are captured in the advisory `# flagged:` line.

### 4. The `.einmo` containment envelope

*(Rewritten 2026-07-03 after the envelope design discussion; supersedes the
earlier `--- SECTION ---` / per-section-progressive-signature draft.)*

Einmo is **language- and format-agnostic**: it takes chunks of input/output
**text** and that is all it is specified to do in this version. How evaluator
results are serialized into those text chunks is the **test crate's job**
(zweimomo's, for this repo — see §D.2); einmo never interprets body content.

#### 4.1 Envelope structure

A `.einmo` file is: a **header line**, then **sections** separated by a
configurable **separator string**, in the order declared by the metadata
section:

```
#einmo 1 encoding=utf-8 separator=①\n
<metadata section>
①
<INPUT body>
①
<OUTPUT body>            (one OUTPUT section per evaluator result chunk)
①
<perspective bodies…>    (zero or more, named in metadata)
①
<COMMENTS body>
①
<STAMPS section — one JSON object per line>
```

- **Header line** (line 1): `#einmo <format-version> encoding=<enc>
  separator=<escaped-string>`. Self-describing: the parser reads line 1, then
  splits the rest on the separator. Format version starts at `1`.
- **Encoding** is configurable per suite; default `utf-8`.
- **Separator** is configurable per suite; default is the character `①`
  (U+2460) followed by LF. **For Foolish suites the separator is configured as
  the string `"!!" + LF`** — `!!` is a Foolish line comment, so the separator
  reads as comment noise to Foolish tooling.
- **Collision rule:** einmo **refuses to serialize** (hard error at write time)
  any section whose content contains the configured separator sequence — the
  suite must then configure a different separator. No escaping mechanism in v1;
  refusal keeps parsing trivially byte-exact.
- Line endings are **LF only**; a `.gitattributes` entry `*.einmo -text`
  protects the signed bytes from git eol normalization.

#### 4.2 Metadata section

Key–value lines, fixed order (canonical — re-serialization is byte-stable):

```
test: stage1/section3/specific.test      # test name = mirror-relative input path
suite: zweimomo/suites/foolish           # suite identity
producer: 4d54ab99                       # commit SHA of the producing tree…
producer-diff: sha256:9f2c…              # …plus SHA of `git diff` when dirty (omitted when clean)
generated: 2026-07-03T15:30:45Z          # time of production (ISO8601 UTC)
status: normal                           # normal | input-error | output-error
status-detail:                           # specifics when status ≠ normal (free text, may be multi-line)
sections: INPUT, OUTPUT, OUTPUT[1], names-perspective, COMMENTS, STAMPS
```

**`status` / `status-detail` (evaluator error semantics).** When the evaluator
cannot parse/accept the input, `status: input-error`; when evaluation fails
abnormally (panic, crash, harness fault), `status: output-error`. In both cases
`status-detail` carries **maximal specifics** — this file is a debugging/repair
artifact, so detail is the point. Distinguish carefully: an *expected* error
result (e.g. an OUTPUT body of `infinite loop detected`, a division-by-zero NK
alarm) is **normal** — tests are allowed, even expected, to pin error
behaviour, and such files are promotable all the way to `verified/`. `status`
marks *the harness's* abnormality, not the program-under-test's.

#### 4.3 Bodies: INPUT, OUTPUT, perspectives, COMMENTS

- **INPUT** — the test trigger text, byte-exact.
- **OUTPUT** — one section per text chunk the evaluator returned
  (`OUTPUT`, `OUTPUT[1]`, `OUTPUT[2]`, … as declared in `sections:`).
  (Called RESULT in earlier drafts of this FOOP.)
- **Perspective bodies** — optional, statically configured views of the input
  or output (see §4.5), one section each, named in `sections:`.
- **COMMENTS** — free text (promotion history, reviewer notes). Always present,
  possibly empty.

#### 4.4 STAMPS — the signature chain (Compiled / Configured / Stage keys)

Einmo is open source, but an organisation incorporating it (or building a
custom einmo) may want secret keys. There are **three named key roles**:

| Key | Origin | Secret? | What its stamp signs |
|---|---|---|---|
| **Compiled Key** | embedded in the `einmo` binary at **compile time** | in a custom build, yes; in the stock open-source build the default key is public knowledge | the **Configured Key's public key** (certification) |
| **Configured Key** | set at **configuration time** (suite/deployment config) | optionally | the **Stage keys' public keys** (certification) |
| **Stage Keys** | one per stage, resolved per §B.5 (config or passphrase prompt) | verified-stage key is typically a human passphrase | **all file bytes before its own stamp** (content + all prior stamps) |

The two secret keys (Compiled, Configured) sign only **public keys** — small,
constant-size certification stamps. Content integrity comes from the Stage-key
stamps, each covering every byte before it, forming the append chain:

> first the Compiled key signs, then the Configured key signs, then the stage
> key signs; each subsequent stage **appends** its stamp after the previous
> stage's stamp.

The STAMPS section is **JSON, one object per line**, appended in order:

```json
{"key":"compiled","pubkey":"<hex>","signs":"pubkey:configured","signature":"<b64>","produced_by":"einmo 0.1.0 sha256:<binhash>","timestamp":"2026-07-03T15:30:45Z"}
{"key":"configured","pubkey":"<hex>","signs":"pubkey:stage:output","signature":"<b64>","produced_by":"einmo 0.1.0 sha256:<binhash>","timestamp":"2026-07-03T15:30:45Z"}
{"key":"stage:output","pubkey":"<hex>","signs":"prior-bytes","signature":"<b64 over all bytes before this line>","produced_by":"einmo 0.1.0 sha256:<binhash>","timestamp":"2026-07-03T15:30:45Z"}
{"key":"stage:checked","pubkey":"<hex>","signs":"prior-bytes","signature":"<b64 over all bytes before this line>","produced_by":"einmo 0.1.0 sha256:<binhash>","timestamp":"2026-07-04T09:12:00Z"}
{"key":"stage:verified","pubkey":"<hex>","signs":"prior-bytes","signature":"<b64 over all bytes before this line>","produced_by":"einmo 0.1.0 sha256:<binhash>","timestamp":"2026-07-05T11:00:00Z"}
```

- **`produced_by`** names the program that generated the signature (name,
  version, binary SHA-256) — signature metadata is just another JSON field, so
  the separate `# produced-by:` advisory line from earlier drafts is
  **dropped**; provenance rides inside each stamp.
- **Timestamps** (generation, each promotion) live inside the corresponding
  stage stamp — signed, attributable per stamp.
- A promotion **appends** the destination stage's stamp; existing stamps are
  never modified or removed. Re-promotion after demotion appends again (the
  history is the chain).
- Later-stage pubkeys are deliberately **not** certified by the Configured key
  when they come from a human passphrase — that is the emergent
  human-attestation property (§B.4): a stamp whose pubkey equals a well-known
  computer key is post-hoc detectable as non-human.
- **Verify-on-inspect** verifies every stamp: certifications check out, each
  stage stamp's signature matches the bytes before it.

*(Interpretation note, to be confirmed by BDFL: "sign the public keys using
these two secret keys" is implemented here as Compiled certifies Configured's
pubkey and Configured certifies the Stage pubkeys, with content integrity
carried solely by the Stage stamps' prior-bytes signatures. If instead the
Compiled/Configured stamps should also cover content, add
`"signs":"prior-bytes"` variants — the envelope shape is unchanged.)*

#### 4.5 Perspectives (statically configured views)

Einmo can be **programmatically configured with static perspectives** of the
input and/or output. A perspective is a pure text→text transform supplied by
the test crate (einmo stays language-agnostic — it never parses body content):

```rust
pub struct Perspective {
    pub name: &'static str,                       // section name in the envelope
    pub of: PerspectiveOf,                        // Input | Output(i)
    pub extract: fn(&str) -> String,              // pure transform
}
```

Example (supplied by zweimomo for Foolish suites): a **brane-name
perspective** — from `{a=1,b=2,c=3}` it extracts `{a=???,b=???,c=???}`, the
name-skeleton of the brane. The perspective body is a first-class signed
section of the envelope.

**Stated goal:** make it easy for human and AI inspectors to look at the key
characteristics of a test at a glance, and to aid debugging/repair when code
breaks (a reviewer can diff the perspective without wading through the full
output).

#### 4.6 Advisory lines

One advisory line kind remains, excluded from all signed bytes:
`# flagged: <reason> <ISO8601>` — appended when a file moves to `flagged/`
(§3). It appears **after the STAMPS section** (never inside a body, so it
cannot collide with content), and the parser strips it before any signature
verification. The old `# produced-by:` advisory is subsumed by the
`produced_by` stamp field (§4.4).

#### 4.7 Dependent einmos (`++` variants)

A test may have **dependent** tests — variants of a reference test that alter
one aspect ("special cases"). A dependent's artifact carries, in addition to
its own output, a **DIFF section**: the diff of the reference's OUTPUT against
the dependent's OUTPUT. A reviewer of a variant reads the *delta*, not the
whole output; and because the DIFF depends on the reference's behavior, a
change in the reference automatically invalidates the dependent's approved
baseline — dependency-aware drift detection at the leaf level.

**Naming.** The dependent-name separator is configurable per suite
(`dependent-separator` in `einmo.toml` / `TestConfig`), default **`++`**:

```
input/arithmetic_precedence.foo                      # the reference test
input/arithmetic_precedence++divisionByZero.foo      # a dependent (special case)
output/arithmetic_precedence++divisionByZero.foo.einmo
```

Convention (documented, not enforced — einmo treats names as opaque): the base
test name is `snake_case`, the special-case description is `camelCase`, so the
two parts read distinctly around the `++`. The **reference** of a dependent is
the input whose name is the dependent's name minus its **last** `++segment`,
in the same directory — so chains (`base++a++b`) are well-defined: `base++a++b`
references `base++a`, which references `base`.

**Generation.** The suite runner evaluates references before their dependents
(topological order within a directory). After evaluating a dependent, einmo
computes the DIFF section itself — a **deterministic unified diff** (fixed
3-line context; labels `reference` and `dependent`; no paths, no timestamps in
the headers) of the reference's canonical OUTPUT section(s) against the
dependent's, both from the **same run** (both freshly written to `output/`).
Diffing is text→text, so einmo stays language-agnostic; the evaluator knows
nothing about dependents. The dependent's metadata gains a `reference:` field
(the reference's mirror-relative name), and `sections:` lists `DIFF`.

**Missing or failed reference.** The dependent still evaluates and its own
OUTPUT is captured; the DIFF section then records
`reference unavailable: <reason>` (missing input, reference `status ≠ normal`,
…). The dependent's own `status` reflects only its own evaluation.

**Signing and promotion — no special cases.** All the normal signatures
apply: the DIFF is ordinary signed body content, covered by the stage-stamp
chain exactly like INPUT/OUTPUT (§4.4), and it is **approved/promoted through
the same process required for all einmos** — the DIFF becomes a baseline only
via `promote`, is compared by `compare`, is demoted and re-reviewed by
`console-review`, and is refused on tamper by verify-on-inspect. There is no
side channel by which a diff enters `checked/` or `verified/`.

**Comparison.** For dependent einmos, DIFF joins the **required** compared
sections (with INPUT and OUTPUT, §5). Consequence: when the *reference's*
behavior changes, the dependent's freshly generated DIFF no longer matches its
`checked/` baseline even if the dependent's own OUTPUT is unchanged — the
dependent surfaces in `compare` as `differing (DIFF)`, forcing a review of the
relationship, not just the endpoints.

### 5. Formal comparison semantics (stage matching)

The `compare <stage-a> <stage-b>` operation walks both stage trees in parallel
and, for each mirror-relative path present in both, applies the **matching
test**, which is **per-section**:

> Two `.einmo` files **match** iff:
> 1. File A **verifies correctly against its own stamps** — every stamp
>    validates (certifications + each stage stamp over its prior bytes).
> 2. File B **verifies correctly against its own stamps** — same.
> 3. The **configured sections** of A and B are **byte-identical**, section by
>    section:
>    - **INPUT** — required (always compared)
>    - **OUTPUT** (all `OUTPUT[i]`) — required (always compared)
>    - **DIFF** — required on dependent einmos (§4.7); a reference-behavior
>      change surfaces here even when the dependent's own OUTPUT is unchanged
>    - **perspective sections** — *optionally required* (configurable; derived
>      from INPUT/OUTPUT, so usually redundant to compare)
>    - **COMMENTS** — *optionally required* (configurable)

Sections not listed are **excluded from content comparison**:
- **STAMPS** — legitimately differs between stages; never compared.
- **metadata** — identical across stages by construction (promotion preserves
  it). If it drifted, verify-on-inspect (steps 1–2) would catch the corruption.

**Why COMMENTS is optionally required:** COMMENTS holds review annotations that
legitimately differ between `output` and `checked`. Some suites want COMMENTS
locked; others treat it as advisory.

```rust
pub fn compare(config: &TestConfig, a: Stage, b: Stage, sections: MatchSections) -> ComparisonResult;

pub enum MatchSections { InputOutput, InputOutputComments }

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
# Promote output -> checked (AI agent or human; appends stage:checked stamp, no prompt)
einmo promote output->checked <work_dir> [--filter <glob>]

# Promote checked -> verified (human, passphrase; defaults to interactive)
einmo promote checked->verified <work_dir> [--passphrase <v> | --stdin-passphrase | --interactive] [--batch]

# Promote output -> verified directly (skipping checked, human only)
einmo promote output->verified <work_dir> [--stdin-passphrase | --interactive]
```

The CLI resolves the destination stage's key via the cascade (§B.5). Promotion
to `verified` warns if the resolved key equals a well-known computer key (the
`stage:verified` stamp's pubkey would match — post-hoc detectable as a
non-human attestation).

### 7. Stage comparison CLI

```bash
einmo compare <stage-a> <stage-b> <work_dir> [--match-sections input,output] [--require-comments-match] [--stale-days N] [--filter <glob>] [--require-match] [--json] [--root-cause]
```

- `--match-sections <list>` — which sections must be byte-identical. Default
  `input,output`. Use `input,output,comments` to require COMMENTS too.
- `--require-match` — exit non-zero if any file is `differing`, `only_in_a`, or
  `only_in_b` (used by the gates).
- `--root-cause` — on a `differing` file, descend its subtree; report the
  deepest `differing` descendants (the candidate root causes). See §Design
  preference.
- `--stale-days N` — warn about files in stage-b whose mtime is older than N
  days relative to stage-a.

### 8. Key resolution cascade and configuration

See §B.5. Implemented in `einmo::config::resolve_stage_key(stage, …)` with
precedence: `--passphrase` > `--stdin-passphrase` > `EINMO_PASSPHRASE` env >
`einmo.toml` `[signing.<stage>] passphrase` > interactive `/dev/tty` prompt.
`--interactive` forces the prompt. `einmo.toml` also carries `[signing]
configured-key` (the Configured Key, §4.4), envelope settings (`encoding`,
`separator`), `parallel` (run parallel or serial), and `[ci]` / `[review]`
sections. The Compiled Key is embedded at build time (stock builds embed the
published default; custom builds may embed a secret).

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
    pub fn stamps(&self) -> &[Stamp];                           // compiled/configured/stage:* in order
    pub fn highest_stage_stamp(&self) -> Option<&Stamp>;        // most recent stage:* stamp
    pub fn stamped_by(&self, pubkey_prefix: &str) -> bool;
    pub fn verify_all(&self) -> Vec<StampVerification>;
}
```

### 10. CI integration — gates implementable in einmo

The commit / merge / tag gates are configurations of einmo commands wrapped in
tiny shell glue. Einmo provides the primitives; the gates are *configurations*,
not separate code.

**Commit gate (pre-commit hook):**
```bash
#!/bin/sh
einmo compare output checked --work-dir . --require-match || {
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
      - run: ./target/release/einmo verify --work-dir . --all
      - run: ./target/release/einmo compare checked verified --work-dir . --require-match
```
`compare checked verified --require-match` = 0 differing, no `only_in_checked`
(new checks that need promotion). Enforces "merging requires all checked einmos
to be verified." A new check appears as `only_in_checked` and blocks merge until
a human runs `einmo promote checked->verified --interactive` on the PR.

**Tag / release gate (pre-tag):**
```bash
#!/bin/sh
set -e
einmo compare checked verified --work-dir . --require-match
einmo confirm-signatures verified --pubkey-prefix "$RELEASE_KEY_PREFIX" --require-all
# both pass → git tag -s "$VERSION"
```
`confirm-signatures verified <prefix> --require-all` = every `verified/*.einmo`
carries a `stage:verified` stamp whose pubkey starts with the release officer's
key. An AI-generated stamp (its pubkey equals a well-known computer key) does
not match → tag blocked.

**Burden-of-correction rules (encoded in gate failure messages):**
- `compare output checked` fails → the producer of the divergent `output`
  repairs (fix code so output matches checked) or escalates (flag the checked
  file, promote new output to checked after review).
- `compare checked verified` fails → the producer of the `checked` version
  corrects (re-promote to match verified) or escalates. Attributable via the
  stamp chain.

### 11. Randomized re-inspection

**The problem:** even when `output==checked` (no new drift), reviewed baselines
can rot — a `checked` or `verified` file may encode a bug locked in at review
time. Kent Beck scores snapshots low on "Inspiring" (reviewers don't think
twice). Statistical re-inspection forces a random sample of already-promoted
files back through review. No surveyed framework does this.

**Specification:** `einmo console-review <work_dir> <from>-><to>
--reexamine-rate <pct> [--reexamine-seed <seed>]` — in addition to demoting
files that genuinely differ, randomly sample `pct`% (default 10) of files
already in `<to>`, demote them back to `<from>` (move; all stamps preserved),
and re-present them for review. `--reexamine-seed <seed>` pins the RNG for
reproducibility (CI can fix a seed per cycle). `--full` is shorthand for
`--reexamine-rate 10`. The re-examined file's re-promotion appends **another
stage stamp** (the re-inspection is itself attributable). A re-examination that
*rejects* a file (flagging it) surfaces baseline rot — the burden of correction
falls on the process that originally produced the checked version.

### 12. CLI self-attestation (binary integrity check)

The `einmo` binary can perform an integrity check on **its own program
file**, adding provenance evidence to the attestation chain: alongside the
cryptographic signatures on `.einmo` content, the *tool that produced/verified*
the artifacts is itself identifiable.

```rust
let exe_path = env::current_exe()?;
let hash = sha256_of_file(&exe_path)?;
```

**`einmo self-check [--expected <sha256>] [--quiet]`** computes the
SHA-256 of `env::current_exe()?`, prints the path and hash, and — if
`--expected <sha256>` is given (or a sidecar `einmo.sha256` ships next to
the binary, or a release-attestation file records it) — exits non-zero on
mismatch. Use cases:

- A release officer runs `einmo self-check --expected <release-hash>`
  before `promote checked->verified`, confirming the signing binary is the
  audited build.
- CI runs `einmo self-check --expected <pinned-hash>` as a gate before
  any `verify`/`compare`, so a tampered or substituted `einmo` cannot
  rubber-stamp a corrupted corpus.
- An auditor records the binary hash alongside a release attestation.

**Producer provenance rides inside each stamp (§4.4).** Every stamp's
`produced_by` JSON field records the program that generated that signature —
`"einmo <version> sha256:<binary-hash>"`. This replaces the earlier drafts'
separate `# produced-by:` advisory line: provenance is just another field of
the signature metadata. Each stamp attributes its own producer, so a corpus
records *which binary generated the output* and *which binary performed each
promotion* — queryable by `einmo show <file>` and surfaced in the UI's
stamp-summary panel.

**Version coupling note:** a stamp's `produced_by` is covered by *subsequent*
stamps' prior-bytes signatures (it is part of the file), but content matching
(`compare`, §5) never compares the STAMPS section — so rebuilding the tool
changes future stamps' `produced_by` without invalidating any existing stamp
or breaking stage correspondence across tool versions. The binary's integrity
is additionally attested *out-of-band* via `self-check` against a
pinned/recorded hash. The two together — per-stamp producer provenance +
out-of-band binary self-attestation — constitute "more evidence to the
attestation" without sacrificing comparability.

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

A Rust HTTP/WebSocket server (`einmo serve`) wraps the library and
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
| **Approve / promote** — promote output->checked (no passphrase) or checked->verified (prompts in-UI) | `promote(from, to, key)` | agents; humans |
| **Flag** — move file to flagged/ with reason | `flag(stage, filter, reason)` | any actor |
| **Verify all** | `verify(all)` | CI, reviewers |
| **Confirm-signatures** — filter by pubkey prefix | `confirm_signatures(path, prefix)` | release officer |
| **Console-review** — guided review with vimdiff/in-UI diff, @agent handling, randomized re-inspection | `console_review(from, to, opts)` | reviewers |
| **Stamp inspection** — per-file stamp chain (keys, produced_by, timestamps) | `stamps()` | auditors |

### Alert process

| Alert | Trigger | Severity | Burden |
|---|---|---|---|
| **Output drift** | `compare output checked` ≠ 0 | error (blocks commit) | the agent that produced the divergent output repairs or escalates |
| **Unverified checked** | `checked` file with no corresponding `verified` | error (blocks merge) | a human must promote checked->verified, or the checked producer corrects/escalates |
| **Flagged file** | any file in `flagged/` | warning | the flagger resolves or regenerates |
| **Stale baseline** | `compare --stale-days N` flags an old file | warning | scheduled re-inspection picks it up |
| **Signature failure** | verify-on-inspect refuses a tampered file | critical | investigate via the stamp chain |
| **Non-human verified stamp** | `confirm-signatures verified <release-key>` finds a `stage:verified` stamp whose pubkey is a well-known computer key | critical | an AI agent bypassed the human gate; attributable to the computer key |

### Frontend forms (all talk to `einmo serve` or embed the library)

1. **CLI** (`einmo …`) — primary frontend for agents and terminal-native humans.
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
| **CLI** (`einmo`) | Rust | all verbs: promote/flag/compare/verify/confirm-signatures/show/console-review/serve | Thin wrapper over the core; the stable scriptable surface (`--json` on every verb). |
| **Git hooks / CI glue** | Shell | pre-commit, pre-tag scripts; GH Actions `run:` steps | 3–5 lines each, pure `einmo …` invocation. |
| **Presentation (optional)** | Python (or Rust) | rich CI reports; optional `textual` TUI | Reads `einmo compare --json`. Pure presentation — never touches files. |
| **UI frontends** | varies | web SPA / TUI / desktop app | Talk to `einmo serve` (REST/WS) or embed the core. Never hold keys. |

**The discipline (invariant):** Shell, Python, and UI are **frontends** that
call the Rust core (subprocess CLI, HTTP to `serve`, or PyO3 in Proposal B).
They may **never** read/write/parse/serialize `.einmo`, derive keys, or
implement the passphrase cascade. Any logic that is an invariant lives in the
Rust core and is *invoked, not re-implemented*.

### 2. Proposals

#### Proposal A — Rust monolith + embedded `serve` (recommended)

Everything that touches `.einmo` or keys is Rust; shell is git-hook/CI glue
only; the web UI is `einmo serve` (axum) inside the same binary.
`console-review` (vimdiff + diff -I + randomized re-inspection) is Rust — the
demote-random-sample is an invariant (`->flagged` move) and must not live outside
the core. UI: static SPA (SvelteKit/Vite+React) served by `einmo serve`
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
signing server (`einmo serve`) holds keys and is the only mutation path.
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

### 5. Build order (do not start UI until the core passes its own gates)

**Scope change:** einmo is built from scratch; no existing code is migrated.
The existing `foolish-core/src/signature.rs` and `snapshot_suite.rs` are design
references, not files to move.

1. Fix the `.gitignore` `bin/` pattern (narrow to `/bin/` or `target/**/bin/`)
   so a fresh `einmo/src/bin/` is not gitignored.
2. **Implement `einmo::signature` from scratch** per FOOP-22 append-chain. **Write
   tamper/forgery tests first** — highest-risk step.
3. **Implement `einmo::snapshot_suite` from scratch** with `Evaluator` returning
   `Vec<String>` (no `FirRef` dependency).
4. Implement `einmo::format` (`.einmo` parse/serialize, per-section canonical).
5. Implement stage directories + hierarchical mirroring.
6. Implement verify-on-inspect + `verify`.
7. Implement promotion + flagging (move/copy semantics).
8. Implement `compare` (per-section matching, verify-both-then-identical).
9. Implement passphrase cascade.
10. Implement CLI (`einmo` binary).
11. Implement gates (shell glue).
12. **Build `zweimomo`** — the companion test crate (three pure-Rust `Evaluator`
    impls + parallel test-input corpus). See §Use Case D.
13. **Einmo's own CI uses its own gates** (commit: `compare output checked`;
    merge: `compare checked verified`) — the library eats its own dog food via
    zweimomo before any UI lands.
14. Only then: `einmo serve` + SPA (Proposal A's UI layer).
15. (Future, separate effort) migrate the legacy `.snap` corpus to `.einmo`.

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
`SnapshotSuite` in `foolish-core`/`foolish-ubca` `*_snapshot_tester.rs`, parser
unit tests in `foolish-parser`. **No Rust CI exists today**; **no CLI/trycmd
tier**; **no bench tier**. The existing `foolish-ubcb` crate referenced in
earlier drafts has been removed from the workspace (FOOP-03 cleanup); einmo
targets `foolish-core` + `foolish-ubca` only.

**Note on JVM removal (FOOP-03):** Per **FOOP-03** (deprecating JVM
implementations), the cross-validation tier (Java/Scala/Rust output comparison)
is **dropped from Einmo's scope**. The current `.github/workflows/tests.yml`
"Cross Validation" workflow (Java/Scala/Maven only, no Rust) is not migrated
into Einmo. Einmo's scope is the Rust implementation's signed-snapshot lifecycle
only. If JVM implementations are un-deprecated later, the per-impl stage-dir
model (`output-rust/`, `output-java/`, etc. with cross-`compare`) is the
structure that would host cross-validation — out of scope today.

### Tier table

| Tier | Inputs | Evaluator | Stages | `require_correspondence` | Gate |
|---|---|---|---|---|---|
| **1. Unit (inlined)** | inlined strings (`evaluate_inline`) | function under test | `output`, `checked` | `[(Output, Checked)]` | commit: `output==checked` |
| **2. Approval / snapshot (VM gate)** | `.foo` in `input/` (hierarchical) | foolish-ubca VM + humanizing sequencer | full pipeline | CI: `[(Output, Checked)]`; release: `+[(Checked, Verified)]` | commit: `output==checked`; merge: `checked==verified` |
| **3. Cross-language (zweimomo)** | parallel inputs in Foolish/Python/JS (`input/` per language) | three `Evaluator` impls (ubca, rustpython-vm, boa_engine) | `output`, `checked` per language | `[(Output, Checked)]` per language | commit: `output==checked` per language |
| **4. Integration** | `.foo` exercising parser+VM+sequencer | end-to-end | full pipeline | same as approval | same; `--stale-days 30` |
| **5. CI (automated)** | re-runs tier 2/3 | same | `output`, `checked` | `[(Output, Checked)]` + `verify --all` | push/PR: verify + compare |
| **6. Release / deployment** | re-runs 2/3 | same | `verified` | `[(Checked, Verified)]` + `confirm-signatures verified <release-key> --require-all` | tag gate |
| **7. Regression / re-inspection** | existing `input/` | same | demotes `verified->checked` sample | ad-hoc | scheduled: `console-review checked->verified --reexamine-rate 10 --reexamine-seed $WEEK` |
| **8. Performance (optional)** | `.foo` + timing | timing→formatted string | `output`, `checked` | `[(Output, Checked)]` | commit; redact volatile values before signing |

**Tier 3 (cross-language, new)** is what `zweimomo` exercises. It is the tier
that proves the `Evaluator` trait is language-agnostic: three interpreters
produce signed outputs for parallel inputs, and each is gated independently.

### Per-tier `TestConfig` example (approval, CI profile)

```rust
let config = TestConfig::default("zweimomo/suites/foolish")
    .require_correspondence(Stage::Output, Stage::Checked);   // CI gate
let suite = EinmoSuite::new(config);
let results = suite.evaluate_all(num_cpus::get(), &UbcaEvaluatorAdapter);
assert!(results.all_written_and_correspondence_holds());
```

## Idealized development flow

### The actors

- **Coding agents** (many, parallel) — write code on feature branches, run
  tests, generate `output/`, promote `output->checked`, commit.
- **Human reviewers** — review diffs, run `console-review`, promote
  `checked->verified` (interactive passphrase), merge.
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
     `output->checked`), or escalate (flag and surface to human).
3. The agent commits. Pre-commit hook runs `einmo compare output checked
   --require-match`; blocked unless `output==checked`.
4. The agent pushes and opens a PR. CI runs `einmo verify --all` (Layer 1) and
   `einmo compare output checked --require-match` (Layer 2).
5. A human reviews: `einmo console-review checked->verified --interactive`
   (verified-stage key not in repo config → prompt appears; human types it).
   Promotes accepted baselines (appends the `stage:verified` stamp).
   Rejected → `einmo flag checked --reason …`.
6. Merge gate: `einmo compare checked verified --require-match` = 0 differing.
   New checks (promoted `output->checked` but not `checked->verified`) block
   merge until a human verifies them.
7. `checked ≠ verified` at merge → the producer of the `checked` version has the
   burden: re-promote or flag-and-escalate. Attributable via the stamp chain.
8. Release: `einmo compare checked verified` + `einmo confirm-signatures
   verified <release-key> --require-all`. Both pass → tag.
9. Continuous re-inspection: weekly cron `einmo console-review checked->verified
   --reexamine-rate 10 --reexamine-seed $WEEK`. A rejection surfaces baseline
   rot; re-promotion appends another `stage:verified` stamp.

### The invariant this flow enforces

No code change reaches `main` without: (a) its outputs matching a reviewed
`checked` baseline, AND (b) that baseline carrying a human `stage:verified`
stamp in `verified/`. The stamp chain attributes every state to an actor. An AI
agent that bypasses the human gate (`--passphrase ""`) produces a verified
stamp under the well-known computer key — `confirm-signatures verified
<release-key>` catches it.

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
- **Per-section matching:** `compare` matches INPUT+OUTPUT (required) +
  perspectives/COMMENTS (optional); both files independently verify first.
- **Randomized re-inspection:** first-class feature (`--reexamine-rate`,
  `--reexamine-seed`).

### Resolutions from the 2026-07-03 pre-MVP design review (BDFL)

- **Output churn accepted.** Signed timestamps mean every run rewrites
  `output/` bytes; tolerated for now, redesign only when it becomes a real
  problem (§B.3).
- **Three-role key model.** Compiled Key (compile time, secret in custom
  builds) + Configured Key (configuration time) + per-stage Stage Keys; the
  two secret keys sign public keys (certification), stage stamps sign all
  prior bytes and append stage after stage (§4.4). Supersedes the `test`/`util`
  signer roles of earlier drafts.
- **Envelope**: header line + configurable encoding (default `utf-8`) +
  configurable section separator (default `①`+LF; Foolish suites use
  `"!!"`+LF — a Foolish line comment); separator collision = hard error at
  write time; STAMPS section is JSON, one object per line, with `produced_by`
  metadata per stamp (§4). Serialization of evaluator results into text
  chunks is the test crate's (zweimomo's) job — einmo is language-agnostic.
- **Perspectives**: statically configured text→text views of input/output as
  first-class signed sections; goal is at-a-glance inspection for human/AI
  reviewers and debugging aid (§4.5).
- **Error semantics**: `status: normal | input-error | output-error` +
  detailed `status-detail` in metadata; expected error *outputs* (e.g.
  "infinite loop detected") are `normal` and promotable to `verified/` (§4.2).
- **Single CLI app, cargo-installable**: one CLI surface; crate `einmo`
  installs via `cargo install einmo` providing `einmo` + the `cargo-einmo`
  alias so `cargo einmo …` works; `verify-signatures` is a subcommand, not a
  separate tool (§1, §B.6). Parallelism is configurable (parallel or serial).
- **CLI arrows are ASCII** `->`; stage names match `[A-Za-z0-9_-]+` (§B.6).
- **Per-stage keys in config**: `[signing.<stage>]` is part of v1 (no longer
  deferred) — it is how "each stage has different configured keys" is
  expressed; the verified stage is deliberately left unset (§B.5).
- **First-run bootstrap**: no special-case messaging; the correspondence test
  simply fails until someone promotes — humans/AI learn the flow from project
  documentation.
- **Interpreter pins**: zweimomo pins `rustpython-vm = "=0.5.0"` and
  `boa_engine = "=0.21.1"` exactly; zweimomo itself is semantically versioned,
  and bumping an interpreter pin is at least a minor version bump with an
  expected re-review of the corpus.
- **Algorithm-corpus licensing**: TheAlgorithms/Python and /Rust are MIT
  (usable, with attribution); TheAlgorithms/JavaScript is **GPL-3.0** — its
  code is **not** copied into this repo. JavaScript inputs are written
  ourselves (translated from the MIT Python implementations); Foolish inputs
  are written ourselves by definition.
- **crates.io names**: `einmo` and `zweimomo` verified unclaimed (2026-07-03);
  reservation/publishing is handled by the BDFL personally.

## Open Questions

- **Stamp-certification semantics (confirm §4.4 interpretation).** Written as:
  Compiled certifies the Configured pubkey, Configured certifies Stage
  pubkeys, content integrity carried solely by Stage stamps' prior-bytes
  signatures. If Compiled/Configured stamps should *also* cover content, add
  `"signs":"prior-bytes"` variants — envelope shape unchanged. Awaiting BDFL
  confirmation.
- **Desktop app framework (Tauri vs egui).** Deferred — both reuse `einmo
  serve`'s REST API unchanged; nothing to decide now.
- **WASM verify in browser (Proposal C).** Deferred — `einmo::verify` clean
  submodule keeps this available as a future enhancement.

## MVP — what the first shippable version supports

We therefore support, as the MVP, the use cases described in **Use Case A**
(constructing tests: text in → signed `.einmo` out, `EinmoSuite` +
`Evaluator`), **Use Case B** (the three-role stamp chain and the core CLI),
and **Use Case D** (zweimomo's three pure-Rust evaluators). Concretely:

**MVP includes:**
- The `.einmo` envelope (§4): header line, configurable encoding + separator,
  metadata with `status`/`status-detail`, INPUT/OUTPUT/perspective/COMMENTS
  sections, JSON STAMPS chain (Compiled / Configured / Stage keys).
- Stage directories with hierarchical mirroring; promote / flag / demote
  transitions (§2, §3).
- `compare` with per-section matching (§5), `verify` with verify-on-inspect,
  `confirm-signatures`.
- The single `einmo` CLI (installable via `cargo install einmo`; `cargo einmo`
  alias) with: `promote`, `flag`, `compare`, `verify`, `verify-signatures`,
  `confirm-signatures`, `show`, `self-check`. Configurable parallel/serial.
- The `Perspective` API (§4.5) with at least one working example (the Foolish
  brane-name perspective, supplied by zweimomo).
- **zweimomo** with its three `Evaluator` impls (foolish-ubca, RustPython,
  Boa) implementing tests over a subset of: **integer arithmetic, grouping,
  precedence, name binding, and function calling with integer inputs and
  outputs** — parallel inputs per §D.4, each language's suite gated by
  `compare output checked`.

**MVP excludes** (specified in this FOOP, built after MVP): the `serve` web
service and SPA (Use Case C), the MCP server, `console-review` (vimdiff
driving, `@agent` handling, randomized re-inspection), the CI gate scripts/
workflow, the exhaustive TheAlgorithms corpus (§D.7 later-phase), and the
legacy `.snap` migration (Deferred).

The plan file marks the MVP boundary: plan phases 0–10 plus the zweimomo
phases (evaluators + §D.4 concept corpus) are MVP; gates, console-review,
serve, and algorithm-corpus phases follow.

## References

- Prior FOOPs:
  - **FOOP-12** — Signature scheme (Ed25519/Argon2id, canonicalization,
    dual-signing, `verify_signatures`). The cryptographic foundation. Einmo
    reimplements this from scratch (does not depend on the existing
    `foolish-core/src/signature.rs`). Status: Final.
  - **FOOP-22** — Multi-signer append format (`test`/`util` roles, "Entire
    file" integrity). Einmo **generalises** it into the three-role
    Compiled/Configured/Stage stamp chain (§4.4); the append principle is
    inherited, the role names are not. Status: Draft.
  - **FOOP-02** — `SnapshotSuite` current home in `foolish-core`, generalised
    over `Evaluator`. Einmo defines its own `EinmoSuite` from scratch (does not
    move the existing file). Status: Draft.
  - **FOOP-42** — Humanizing FIR Sequencer (HFS) output byte format
    (`hfssnap`). The signed body must conform when foolish-ubca is the
    evaluator; einmo itself is format-agnostic (`Vec<String>`). Status: Draft.
  - **FOOP-62** — UBCa; snapshots are the hard acceptance gate; `catch_unwind`
    panic-capture contract. Status: Final.
  - **FOOP-21** — Alarms emitted into snapshot output. Status: Brewing.
  - **FOOP-03** — Repository cleanup / workspace flattening. Cross-validation
    tier dropped from Einmo scope; `foolish-ubcb` removed from workspace.
    Status: Implementing (workspace already flattened).
- External docs (verified mid-2026; full citation in Appendix D):
  - insta (v1.48.0, commit `7f23d2e`) — https://insta.rs/docs/, https://docs.rs/insta
  - in-toto Attestation Framework — https://github.com/in-toto/attestation
  - sigstore-rs — https://github.com/sigstore/sigstore-rs
  - SLSA attestation model — https://slsa.dev/attestation-model
  - GitHub Artifact Attestations — https://github.com/actions/attest
  - jlevy/tbd Golden Sessions — https://github.com/jlevy/tbd/blob/main/packages/tbd/docs/guidelines/golden-testing-guidelines.md
  - insta issue #792 (TOFU/immutable snapshots, OPEN) — https://github.com/mitsuhiko/insta/issues/792
  - insta PR #815 (non-interactive review for LLMs/CI, shipped 1.44) — https://github.com/mitsuhiko/insta/pull/815
  - **rustpython-vm** v0.5.0 — https://github.com/RustPython/RustPython (pure-Rust Python)
  - **boa_engine** v0.21.1 — https://github.com/boa-dev/boa (pure-Rust ECMAScript)
- Code locations (design references, verified mid-2026; paths reflect the
  post-FOOP-03 flattened workspace — `foolish-core/`, `foolish-ubca/` at the
  workspace root, no `foolish/` wrapper):
  - `foolish-core/src/signature.rs` (680 lines, FOOP-12; currently
    REPLACE-based single-signer). **Design reference only** — einmo reimplements
    per FOOP-22; this file is **not modified**.
  - `foolish-core/src/snapshot_suite.rs` (289 lines, FOOP-02; `Evaluator`
    returns `Vec<FirRef>`). **Design reference only** — einmo defines its own
    `EinmoSuite` with `Evaluator → Vec<String>`; this file is **not modified**.
  - `foolish-core/src/bin/verify_signatures.rs` — **exists and is git-tracked**
    (the `.gitignore` `bin/` pattern does not affect already-tracked files). It
    is a working clap binary. Einmo ships its own `verify_signatures.rs` in
    `einmo/src/bin/`; the `.gitignore` must be narrowed so the new file is not
    ignored.
  - `foolish-ubca/src/ubca_snapshot_tester.rs` — the one existing `Evaluator`
    adapter (`UbcaEvaluator`). Zweimomo wraps it (does not modify it).
  - `foolish_review.sh` (98 lines), `accept_approved.sh` (53 lines) —
    worktree-local; replaced by `einmo` subcommands (zweimomo builds its
    own corpus; existing scripts untouched).
  - 276 committed `.snap` files (131 core + 145 ubca + 0 ubcb) + 285 `.snap.new`
    files across `foolish-core`/`foolish-ubca` snapshot_tests dirs. **Not
    migrated** by this FOOP; migration is a future, separate effort.

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

# Appendix F — Codebase state (supplemental, verified mid-2026, post-FOOP-03 flattening)

## F.1 Test-bearing files (the real tiers)

- `foolish-core/src/`: `fir.rs` (22 tests), `signature.rs` (27 tests),
  `sequencer_tests.rs` (21), `unit_tests.rs` (14).
- `foolish-ubca/src/`: `fir_kinds.rs` (88 tests, incl. all 16
  `*_nyes_transitions`), `fir_trait.rs` (30, incl.
  `step_leaf_through_nyes_transitions`), `proto_brane.rs` (5), `nyes_ext.rs` (5),
  `ubca_snapshot_tester.rs` (1).
- `foolish-parser/src/`: `parser.rs` (14), `lexer.rs` (8).
- `foolish-cli/src/`: CLI binary; no snapshot tests.

No `tests/` integration dirs. No CLI/trycmd tests. No benchmarks (`#[bench]`/
criterion). The `*_nyes_transitions` tests live in `foolish-ubca/src/fir_kinds.rs`
(inline `#[cfg(test)] mod tests`); AGENTS.md mandates extending them when FIR
kinds/NYES states change.

**Note:** `foolish-ubcb` was removed from the workspace by FOOP-03. Any earlier
draft referencing `foolish-ubcb` test files, `UbcbEvaluator`, or
`ubcb_snapshot_tester.rs` is stale.

## F.2 Snapshot directories

Two crates have `snapshot_tests/{input,approved}/`:
- `foolish-core/snapshot_tests/approved/` — 131 `.snap` + `.foo` inputs; some `.snap.new`.
- `foolish-ubca/snapshot_tests/approved/` — 145 `.snap`; stray `.snap.approved`, editor `.swp/.swo`.

276 committed `.snap` + 285 `.snap.new` files total. **Not migrated by this
FOOP** — einmo builds its own corpus via zweimomo; legacy `.snap` migration is a
future, separate effort.

## F.3 CI (no Rust)

`.github/workflows/tests.yml` ("Cross Validation") runs Java/Scala/Maven only.
**No `cargo test` in any workflow.** Einmo's CI gates are greenfield (Rust).

## F.4 Test-harness helpers (design references; NOT migrated)

- `foolish-core/src/snapshot_suite.rs` — the shared `SnapshotSuite` +
  `Evaluator` trait (returns `Vec<FirRef>`). **Design reference**; einmo defines
  its own `EinmoSuite` returning `Vec<String>`.
- `foolish-ubca/src/ubca_snapshot_tester.rs` — the one existing `Evaluator`
  adapter (`UbcaEvaluator`). Zweimomo wraps it (does not modify it).
- `foolish-core/src/signature.rs` — Ed25519/Argon2id (currently REPLACE-based).
  **Design reference**; einmo reimplements per FOOP-22.
- `foolish-core/src/bin/verify_signatures.rs` — exists and is git-tracked
  (dangling-`[[bin]]` premise was wrong; the file is present). Einmo ships its
  own `verify_signatures.rs` in `einmo/src/bin/`.
- `foolish_review.sh` (98 lines), `accept_approved.sh` (53 lines) —
  worktree-local; left untouched.

Insta workspace dep: `insta = { version = "1", features = ["yaml"] }`; dev-dep in
foolish-core, foolish-ubca, foolish-parser. Einmo does **not** depend on insta;
zweimomo does not either (both write `.einmo` directly).

---

# Appendix G — Foolish-ubca capability envelope & syntax reference

*(Verified mid-2026 against `foolish-ubca` source + the ~145 `.foo` snap inputs
under `foolish-ubca/snapshot_tests/input/`. This appendix bounds what the
parallel-input matrix (§D.4) can express in the Foolish column.)*

## G.1 Language features that work today

| Feature | Syntax / example | Notes |
|---|---|---|
| Integer literals | `{5;}` `{42;}` | `u64`; no floats, no strings |
| Unary minus | `{-42;}` | |
| Arithmetic | `{2 + 3 * 4 - 5;}` | `+ - * /`; `/` is **integer** division |
| Precedence + parens | `{((2 + 3) * (4 - 1)) / 5;}` | |
| Name binding (SSA) | `{x = 42; y = x + 8;}` | static single assignment; reuse allowed (`a=1; a=2`) |
| Forward references | `{fwd = later; later = 99;}` | use-before-define works |
| Branes (nested) | `{a = 10; b = 20; n = {inner = a+b;};}` | arbitrarily deep |
| Ordinate access | `n.inner` | DotSearch |
| Seek (indexed) | `data#0`, `data#-1` | positional; unanchored `#-N` works |
| Head / Tail | `data^`, `data$` | first / last element |
| Regex search | `anchor?pattern`, `anchor~pattern` | returns match or NK |
| Upward search | `↑` | parent-context search |
| Concatenation | `{p=1;q=2} ⨃ {r=3;s=4}` | brane merge |
| Bind-tail | `a =$ b` ≡ `a = b$` | bind last value of `b` to `a` |
| Function application | `r =$ {a=10,b=-3} fn` | emergent (see §D.5); no `fn()` syntax |
| Detachment brane | `[...]{...}` | scope control |
| StayFoolish | `<expr>` | lazy/captured evaluation |
| StayFullyFoolish | `<<expr>>` | fully lazy |
| Named brane tag | `name'{...}` | characterization |
| NK literal | `???` | "not knowable" |
| Comments | `!! line` / `!!! block !!!` | |
| Unicode identifiers | `π`, `привет`, `名前` | diverse scripts supported |

## G.2 What does NOT work (sets the matrix ceiling)

- **No string type** — `"hello"` parses in the README example but `Astn` has no
  `StringLit`; there is no string literal in the AST. → string-ops concept row
  dropped from the matrix.
- **No floats** — only `IntLit(u64)`. → Python/JS arithmetic columns use integer
  arithmetic to match outputs.
- **No booleans** — no `true`/`false`.
- **No function definitions** (as an AST node) — no `fn`, `=>`, `\`, no `FnDef`/
  `Lambda`/`CallExpr`. Function application is *emergent* via concatenation +
  `$`/`=$` (§D.5), not syntactic.
- **No recursion / loops / iteration** — pure SSA, no control flow.
- **`if/then/elif/else/fi`** — parsed by the lexer but **rejected at compile
  time** (`compiler.rs`: `"if-then-else: not supported (FOOP=2)"`).
- **No closures / higher-order functions.**

## G.3 Alarms and NK (error-as-value)

- Division by zero: `10 / 0` → the FIR becomes NK and an **alarm** is emitted
  into the output (FOOP-21). The signed `.einmo` captures the alarm — a
  panicking evaluation still produces a signed, reviewable artifact.
- `???` is the explicit NK literal.

## G.4 The `hfssnap` output format

`UbcaEvaluator::evaluate` returns `Vec<FirRef>` (one per top-level statement).
Each FIR is rendered by `FirSequencer::format(fir) → String` into an
indented, line-budgeted tree — the `hfssnap` body. The assembled `.snap` is
INPUT + one RESULT block per FIR + COMMENTS + signature footer. Einmo's
`UbcaEvaluatorAdapter` (§D.3) calls the same `FirSequencer::format` and collects
the strings into `Vec<String>`.

## G.5 Representative `.foo` inputs (from the existing corpus)

```
{5;}                                                    # simple integer
{-42;}                                                  # unary minus
{2 + 3 * 4 - 5;}                                        # precedence
{x = 42; y = x + 8; y;}                                 # name binding + scope
{fwd = later; later = 99;}                             # forward reference
{5; {10; 15}; 20;}                                      # nested branes
{a = 10; b = 20; nested = {inner = a + b}; nested.inner;}   # ordinate access
{data = {a=10; b=20; c=30}; first = data#0;}            # seek
{a = 10 / 0; b = 20 / 4;}                              # division-by-zero alarm
{answer = ???;}                                         # NK literal
{a=1,b=2; c=<<a+b>>; c; c;}                             # StayFullyFoolish
{d =$ 4; e = 5;}                                        # bind-tail (calling form)
```

*(Use these for inspiration when writing the Foolish column of the parallel-input
matrix — see plan task "use existing snaps for inspiration".)*

---

# Appendix H — RustPython embedding reference

*(Verified mid-2026 against `rustpython-vm` 0.5.0, commit
`f196dc401c98a1c1e8fc2f308ed038d17f859076` on
`github.com/RustPython/RustPython`.)*

## H.1 Crate identity

- **crates.io name**: `rustpython-vm` — v0.5.0 (published 2026-03-31).
- The top-level `rustpython` crate is the *binary*; its lib docs say "If you're
  looking to embed RustPython into your application, you're likely looking for
  the [`rustpython_vm`] crate."
- Embedding entry point: `rustpython_vm::Interpreter`.

## H.2 Pure-Rust confirmation ✅

- `crates/vm/Cargo.toml`: no `links =`, no `cc`, no `cmake`, no `*-sys`.
- Only "system-ish" dep is `libc` (pure-Rust FFI binding, not a C compiler dep).
- `openssl-sys` is gated behind the **off-by-default** `ssl-openssl` feature;
  the default SSL backend is `ssl-rustls` (pure-Rust). Default features:
  `["threading","stdlib","stdio","importlib","ssl-rustls","host_env"]` — **no C
  toolchain required with defaults.**
- No CPython linkage; RustPython is a from-scratch bytecompiler + VM in pure
  Rust, sharing no code with CPython.

## H.3 Embedding API surface

| What | API | Path |
|---|---|---|
| Bare interpreter (no stdlib) | `Interpreter::without_stdlib(Default::default())` | `crates/vm/src/vm/interpreter.rs` |
| Builder (preferred) | `Interpreter::builder(settings)` | same |
| Run code in VM context | `interp.enter(\|vm\| { ... })` | same |
| Fresh scope | `vm.new_scope_with_builtins()` → `Scope` | vm_new.rs |
| Compile | `vm.compile(source, Mode::Exec, "<path>")` → `PyResult<CodeObject>` | |
| Run compiled | `vm.run_code_obj(code, scope)` → `PyResult<PyObjectRef>` | |
| Eval single expr | `rustpython_vm::eval::eval(vm, source, scope, path)` → `PyResult<PyObjectRef>` | `crates/vm/src/eval.rs` |
| Stringify result | `obj.str(vm)?.as_str().to_string()` | `protocol/object.rs`, `builtins/str.rs` |
| Repr | `obj.repr(vm)?.as_str().to_string()` | `protocol/object.rs` |

**Compile modes** (`rustpython_vm::compiler::Mode`): `Exec` (script), `Eval`
(single expression → value), `Single` (REPL), `BlockExpr`.

## H.4 Minimal snippet

```rust
use rustpython_vm::{Interpreter, eval};

let interp = Interpreter::without_stdlib(Default::default());
let out: String = interp.enter(|vm| {
    let scope = vm.new_scope_with_builtins();
    let result = eval::eval(vm, "1 + 2", scope, "<zweimomo>")?;
    Ok(result.str(vm)?.as_str().to_string())
}).unwrap();
assert_eq!(out, "3");
```

## H.5 Caveats for a test-harness evaluator

- **`without_stdlib` is genuinely bare**: arithmetic, function defs, classes,
  comprehensions, f-strings all work without stdlib; but `import` of stdlib
  modules (`os`, `sys`, `json`, `re`) fails until `init_stdlib()` is called.
  → *Sandboxing plus:* no `os`, no file I/O, no `subprocess` unless explicitly
  wired. Recommended for zweimomo: `without_stdlib` (keep it sandboxed).
- **`Interpreter` is not `Send`** (thread-local VM context). A
  `RustPythonEvaluator` impl must hold the `Interpreter` on one thread or use a
  `thread_local!` / dedicated-thread-with-channel pattern.
- Recursion limit is CPython-like; tunable via `Settings`.
- stdlib is large but incomplete vs CPython — irrelevant for an expression
  evaluator.

---

# Appendix I — Boa embedding reference

*(Verified mid-2026 against `boa_engine` 0.21.1, commit
`8a1e8fe07f626f7a067afc2c9885d5d87de4bb5d` on
`github.com/boa-dev/boa`.)*

## I.1 Crate identity

- **crates.io name**: `boa_engine` — v0.21.1 (published 2026-03-29). 3.55M
  downloads — widely used.
- Embedding entry point: `boa_engine::{Context, Source}`.

## I.2 Pure-Rust confirmation ✅

- `core/engine/Cargo.toml`: no `links =`, no `cc`, no `cmake`, no `*-sys` in the
  entire Boa workspace.
- All deps pure-Rust: `regress` (regex), `icu_*` (i18n, compiled-data),
  `num-bigint`, `ryu-js`, `fast-float2`, `hashbrown`, etc.
- The `ffi/` directory is an *outbound* C-API Boa exposes (optional, separate
  crate), **not** a system dependency Boa consumes.
- No V8, no QuickJS, no SpiderMonkey — Boa is a from-scratch ECMAScript
  lexer+parser+bytecompiler+VM in pure Rust.

## I.3 Embedding API surface

| What | API | Path |
|---|---|---|
| Create interpreter | `Context::default()` (also `Context::new(...)`, `Context::builder()`) | `core/engine/src/lib.rs` |
| Wrap source bytes | `Source::from_bytes(&str)` (`Source` re-exported from `boa_engine`) | |
| Eval | `context.eval(source)` → `JsResult<JsValue>` | `Context::eval` |
| Stringify value | `value.to_string(&mut context)?.to_std_string_escaped()` | `value/mod.rs:974`, `core/string/src/lib.rs:183` |
| Error type | `boa_engine::JsError` (`.to_string(&mut context)` for message) | |

## I.4 Minimal snippet

```rust
use boa_engine::{Context, Source};

let mut context = Context::default();
let result = context.eval(Source::from_bytes("1 + 2")).unwrap(); // JsValue
let out = result.to_string(&mut context).unwrap().to_std_string_escaped();
assert_eq!(out, "3");
```

*(Verbatim pattern from the crate-level doc example.)*

## I.5 Caveats for a test-harness evaluator

- Boa targets ECMAScript spec (test262); passes a large majority. Core language
  (ES2020+ incl. classes, generators, async, destructuring, template literals,
  `Map`/`Set`/`Promise`) is reliable.
- **No file I/O, no `fs`, no `child_process`, no `require`/Node APIs by default**
  — pure language core. → *Sandboxing plus:* a Boa `Context` cannot touch the
  filesystem or network unless host functions are explicitly injected. Ideal
  for a snapshot test evaluator.
- **`Intl`** is behind the off-by-default `intl` feature (pulls ICU, still
  pure-Rust). Skip if locale formatting isn't needed.
- **`Context` is not `Send`** (GC + realm are thread-affine). Same
  thread-local / dedicated-thread advice as RustPython.
- Boa is slower than V8 but that is irrelevant for a correctness test harness.


## Last Updated

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.199 (Claude Code); Fable 5
**Changes**: Folded in the pre-MVP design review resolutions (BDFL). (1)
**Envelope rewritten (§4)**: header line + configurable encoding/separator
(default `①`+LF; Foolish suites `"!!"`+LF), metadata with producer commit SHA
and `status`/`status-detail` error semantics, INPUT/OUTPUT(+perspectives)/
COMMENTS sections, **STAMPS as JSON** (one object per line, `produced_by`
provenance per stamp — the `# produced-by:` advisory is subsumed). RESULT
renamed OUTPUT. (2) **Three-role key model (§4.4)**: Compiled Key + Configured
Key certify public keys; per-stage Stage Keys sign all prior bytes and append
per promotion — supersedes `test`/`util`; every promotion now appends a stamp.
Interpretation note added to Open Questions for BDFL confirmation. (3)
**Perspectives (§4.5)**: statically configured text→text views as signed
sections (e.g. Foolish brane-name perspective). (4) **Single CLI app**: crate
`einmo`, `cargo install einmo`, binaries `einmo` + `cargo-einmo` alias; ASCII
`->` stage arrows; stage names `[A-Za-z0-9_-]+`; `verify-signatures` a
subcommand; configurable parallelism. (5) **New MVP section** (final normative
section) defining MVP in/out. (6) Output-churn tradeoff accepted (§B.3);
per-stage `[signing.<stage>]` keys now v1; `.gitattributes *.einmo -text`;
zweimomo owns result serialization ("most colloquial in each language");
interpreter pins `=0.5.0`/`=0.21.1` + zweimomo semver; TheAlgorithms licensing
resolved (JS repo GPL-3.0 → write JS inputs ourselves from MIT Python).
Resolved Decisions extended accordingly.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.199 (Claude Code); Fable 5
**Changes**: Added the **exhaustive algorithm coverage** later-phase scope to
§D.7 (port implementations from TheAlgorithms and similar collections into the
zweimomo corpus, to stress-test einmo at realistic scale — "test the test
framework as thoroughly as possible"), matching the new algorithm-coverage
phase in FOOP-92.plan.md. The plan file was rewritten in the same session to
match this spec's standalone scope (see its Last Updated entry).

**Date**: 2026-07-03
**Updated By**: Sisyphus / z-ai/glm-5.2
**Changes**: Major refresh. (1) **Standalone-einmo scope**: einmo reimplements
signed-snapshot machinery from scratch — NO edits to `foolish-core`/`foolish-ubca`/
any existing crate; promotable to its own repo. Dropped all "migrate
signature.rs/snapshot_suite.rs/.snap corpus" language. (2) **New companion crate
`zweimomo`** (Use Case D): three **pure-Rust** interpreters as `Evaluator` impls —
foolish-ubca (Foolish), rustpython-vm v0.5.0 (Python), boa_engine v0.21.1
(JavaScript). Parallel test-input matrix (arithmetic, parsing, name-binding,
data-structures, function-application, errors, search, SF/SFF) bounded by
foolish-ubca's capability ceiling. Pure-Rust rationale noted (no C/FFI toolchain
→ portable + repo-promotable). Function-application in Foolish documented as
emergent via concatenation + `$`/`=$` tail-binding (§D.5). (3) **Refresh pass**:
flattened paths (foolish/foolish-core/ → foolish-core/, post-FOOP-03), dropped
all `foolish-ubcb` references (removed by FOOP-03), fixed the verify_signatures.rs
premise (file EXISTS + tracked, not missing — but `.gitignore bin/` fix still
needed for new `einmo/src/bin/`), fixed line counts (680/289), fixed `.snap`
counts (276+285; 131 core + 145 ubca + 0 ubcb), fixed FOOP-62 status (Final).
(4) **New appendices G/H/I**: per-language research (Foolish capability envelope
+ syntax reference; RustPython embedding API; Boa embedding API) with verified
crate SHAs and minimal snippets. (5) Added Tier 3 (cross-language) to the
test-tier table. (6) Rewrote §1 (crate structure) and §5 (build order) for
standalone scope.

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
