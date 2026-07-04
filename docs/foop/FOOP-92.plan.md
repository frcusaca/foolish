# FOOP-92 Plan — Einmo: directory-based signed-snapshot testing with staged promotion

This plan executes [FOOP-92](FOOP-92.md). **Read the specification first** —
the plan assumes its context. The design is frozen (all OQs resolved).

**Scope rule (critical, from the spec):** this FOOP creates exactly **two new
crates** — `einmo` and `zweimomo` — and touches **one existing file**
(`.gitignore`, Phase 0). **No other existing file is modified.** `foolish-core`,
`foolish-ubca`, `foolish-parser`, `foolish-cli`, the existing `.snap` corpus,
`foolish_review.sh`, and `accept_approved.sh` are all left untouched. Einmo
reimplements the signing/format machinery from scratch (the existing
`foolish-core/src/signature.rs` and `snapshot_suite.rs` are design references
only) so it can be promoted to its own repository later.

**MVP boundary (spec §MVP):** Phases 0–10 (envelope, stamps, stages, compare,
verify, cascade, CLI) plus Phases 14–15 (zweimomo evaluators + the §D.4 concept
corpus: integer arithmetic, grouping, precedence, name binding, function
calling with integer inputs/outputs) constitute the **MVP**. Phases 11–13
(gates, console-review, serve), 16 (algorithm corpus), and 17 (use-case
validation) are post-MVP.

This plan is written for a smaller implementing LLM. Each task is atomic: one
concrete action, exact file paths, exact commands, and an acceptance check. Do
not combine tasks. Do not skip the acceptance check. Run `cargo fmt`, `cargo
clippy -D warnings`, and the relevant `cargo test` after each phase.

**Never auto-accept snapshots.** Never run `cargo insta accept` or
`INSTA_UPDATE=always`. Einmo has no automated update; promotion is always a
deliberate CLI act. Present `output/` vs `checked/` diffs to a human for review.

Branch and worktree lifecycle (per `foop.md`):

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/home/hcbusy/foolish-rust
WORKTREE_BRANCH_NAME=foop-92-einmo
WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-92-einmo
```

- [ ] begun
      ( ) <!-- timestamp when work commences -->

- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-92-einmo with branch `foop-92-einmo` off `jia`
      ```bash
      cd /home/hcbusy/foolish-rust
      git worktree add -b foop-92-einmo /home/hcbusy/tmp/foolish-worktrees/foop-92-einmo
      cd /home/hcbusy/tmp/foolish-worktrees/foop-92-einmo
      ```

## Phase 0 — Fix the `.gitignore` `bin/` bug (BLOCKING, do first)

The root `.gitignore` has a `bin/` pattern that ignores ANY `bin/` directory
including `src/bin/`. (Note: `foolish-core/src/bin/verify_signatures.rs` exists
and is git-tracked — already-tracked files are unaffected — but NEW files under
a fresh `einmo/src/bin/` WOULD be silently ignored.) Fix before creating
`einmo/src/bin/`.

- [ ] In the repo root `.gitignore`, change the `bin/` line to `/bin/` (anchors to repo-root `bin/` only).
- [ ] Create a repo-root `.gitattributes` (new file) containing `*.einmo -text` — git eol-normalization must never touch signed bytes.
- [ ] Acceptance: `git check-ignore -v einmo/src/bin/cargo_einmo.rs` prints nothing (not ignored), and `git status` shows no previously-tracked file becoming untracked.

## Phase 1 — `einmo` crate scaffolding (standalone)

- [ ] Create `einmo/Cargo.toml`:
      - package name `einmo`, edition 2021, semantic versioning starting `0.1.0`.
      - `[dependencies]`: `ed25519-dalek`, `argon2`, `base64`, `hex`, `clap` (derive), `serde`, `serde_json`, `toml`, `time`, `thiserror`.
      - `[dev-dependencies]`: `tempfile`.
      - Default binary `einmo` from `src/main.rs` (the single CLI app), plus `[[bin]] name = "cargo-einmo" path = "src/bin/cargo_einmo.rs"` — a one-line alias calling the same CLI entry point, so `cargo install einmo` yields both `einmo …` and `cargo einmo …`.
      - **NO dependency on `foolish-core`, `foolish-ubca`, or any workspace crate** — einmo must build standalone (repo-promotable).
- [ ] Create `einmo/src/lib.rs` with module declarations: `mod config; mod stage; mod compare; mod format; mod signature; mod snapshot_suite; mod verify;` and public re-exports: `TestConfig, Stage, StageDirs, MatchSections, Perspective, EinmoSuite, Evaluator, EinmoFile, Stamp, EinmoError, compare, promote, flag, verify, confirm_signatures`. (No `migrate` module — legacy `.snap` migration is Deferred.)
- [ ] Add `einmo` to the workspace `members` in the repo-root `Cargo.toml`. Do NOT add einmo as a dependency of any existing crate.
- [ ] Acceptance: `cargo check --workspace` passes (empty crate compiles; no existing crate changed — `git status` shows only `.gitignore`, `.gitattributes`, `Cargo.toml` members line, and `einmo/`).

## Phase 2 — Implement `einmo::signature` from scratch (three-role stamp chain) — HIGHEST RISK

**Write tamper/forgery tests FIRST.** This is the single highest-risk step.
The existing `foolish-core/src/signature.rs` (680 lines, REPLACE-based
single-signer) is a **design reference for reading only** — do not copy it in,
do not modify it. Einmo implements the **Compiled / Configured / Stage key
model** of FOOP-92 §4.4: the two secret keys (Compiled, Configured) sign
public keys (certification stamps); Stage keys sign all prior file bytes and
append, stage after stage.

- [ ] Create `einmo/src/signature.rs`. Implement Argon2id passphrase→key derivation (parameters pinned by einmo) and Ed25519 sign/verify. Empty passphrase = the well-known computer key.
- [ ] Embed the **Compiled Key** at build time (stock default key in the open-source build; overridable at compile time for custom builds, e.g. via env at build.rs time).
- [ ] Add `Stamp` struct (serde JSON, one object per line in the STAMPS section): `key` (`"compiled"` | `"configured"` | `"stage:<name>"`), `pubkey` (hex), `signs` (`"pubkey:<role>"` | `"prior-bytes"`), `signature` (b64), `produced_by` (`"einmo <version> sha256:<binary-hash>"`), `timestamp` (ISO8601 UTC).
- [ ] Add `Stamps { entries: Vec<Stamp> }` with parse (JSON lines) and serialize (byte-stable).
- [ ] Implement stamp creation: `compiled` stamp signs the Configured Key's pubkey; `configured` stamp signs the output Stage Key's pubkey; `stage:<name>` stamp signs **all file bytes before its own line**.
- [ ] Implement stamp verification: certifications check out; every stage stamp's signature matches the bytes before it; ordering is compiled → configured → stage:output → (appended stage stamps).
- [ ] Appending a stage stamp (promotion): existing stamps preserved byte-for-byte; refuse to append if ANY existing stamp fails verification (chain integrity).
- [ ] Generation/promotion timestamps live inside the corresponding stage stamp (`timestamp` field — signed by any subsequent stamp's prior-bytes coverage).
- [ ] **Tamper tests** (write these FIRST): parse JSON-lines, serialize roundtrip, multi-stage-stamp parse, verify compiled certification, verify configured certification, verify stage stamp prior-bytes, tamper-metadata-detection, tamper-input-detection, tamper-output-detection, tamper-after-promotion-invalidates-later-stamps, chain-integrity-refuses-append-on-broken-stamp, append-preserves-existing-stamps, empty-passphrase-derives-computer-key, custom-compiled-key-changes-certification.
- [ ] Acceptance: `cargo test -p einmo --lib signature` passes (all tamper tests green). `git diff --stat` shows no change to `foolish-core/`.

## Phase 3 — The `.einmo` containment envelope (parse/serialize)

Per FOOP-92 §4: header line, configurable encoding + separator, ordered
sections, JSON STAMPS.

- [ ] Create `einmo/src/format.rs`.
- [ ] Implement the header line: `#einmo <format-version> encoding=<enc> separator=<escaped>` (version `1`; default encoding `utf-8`; default separator `①`+LF).
- [ ] Implement section splitting on the configured separator; section order and names come from the metadata `sections:` field. Bodies: INPUT, one `OUTPUT[i]` per evaluator chunk, named perspective sections, COMMENTS (always present, possibly empty).
- [ ] Implement the metadata section (fixed key order, byte-stable): `test`, `suite`, `producer` (commit SHA), `producer-diff` (git-diff SHA when dirty; omitted when clean), `generated`, `status` (`normal` | `input-error` | `output-error`), `status-detail`, `sections`.
- [ ] Implement the **separator collision rule**: serializing errors (hard, descriptive) if any section's content contains the separator sequence — the suite must configure a different separator. Note the Foolish-suite separator is `"!!"`+LF.
- [ ] Implement `EinmoFile::parse(bytes) -> Result<EinmoFile, EinmoError>` and `EinmoFile::serialize() -> Vec<u8>` (LF-only; byte-exact roundtrip).
- [ ] **Advisory line** `# flagged: <reason> <ISO8601>` (after the STAMPS section) parses as unsigned advisory data, stripped before any verification.
- [ ] Unit tests: parse roundtrip byte-exact; custom separator (incl. `"!!"`+LF); separator-collision refusal; missing-section errors; multiple OUTPUT sections; perspective sections; status/status-detail roundtrip; advisory line excluded from signed bytes; header-line malformed errors.
- [ ] Acceptance: `cargo test -p einmo --lib format` passes.

## Phase 4 — Stage directories + hierarchical mirroring

- [ ] Create `einmo/src/config.rs` and `einmo/src/stage.rs`.
- [ ] Define `Stage` enum (`Output, Checked, Flagged, Verified`), `StageDirs` (output/checked/flagged/verified defaults), `TestConfig` (work_dir, input_dir, stages, require_correspondence, match_sections, encoding, separator, perspectives, parallel), `MatchSections` (`InputOutput`, `InputOutputComments`), `Perspective` (`name`, `of: Input|Output(i)`, `extract: fn(&str) -> String`).
- [ ] Stage names (and any custom stage-dir names) are validated against `[A-Za-z0-9_-]+`.
- [ ] Implement `Stage::dir_name()` and `TestConfig::stage_dir(stage) -> PathBuf`.
- [ ] Implement `mirror_input_path(input_rel_path) -> stage_rel_path`: given `stage1/section3/specific.test`, produce `stage1/section3/specific.test.einmo`. (Append `.einmo` to the input-relative path. Discovery is extension-agnostic — any file under `input/` is a test trigger, so `.foo`, `.py`, and `.js` inputs all work.)
- [ ] Implement `walk_input_tree(config) -> Vec<PathBuf>`: discover all input files under `input/`, return their mirror-relative paths.
- [ ] Implement `ensure_stage_dirs(config)`: create `output/`, `checked/`, `flagged/`, `verified/` (and their mirrored subtrees on demand).
- [ ] Unit tests: flat input → flat stage paths; hierarchical input → mirrored stage paths; same-basename-different-branches coexist; `stage_dir` per stage; non-`.foo` extensions discovered.
- [ ] Acceptance: `cargo test -p einmo --lib stage` passes.

## Phase 5 — Verify-on-inspect + `verify`

- [ ] Create `einmo/src/verify.rs` (clean submodule: NO filesystem, NO tty, NO argon2 — only pure verify over parsed `EinmoFile` + `Stamps`). This keeps Proposal C (WASM verify) available later.
- [ ] Implement `verify_all(einmo_file) -> Vec<StampVerification>`: verify every stamp — `compiled`/`configured` certifications against the certified pubkeys, each `stage:*` stamp's signature against all file bytes before its line.
- [ ] Implement `EinmoFile::from_file(path) -> Result<Self, EinmoError>`: read file, parse, **verify all stamps**; return `Err` if any fail (verify-on-inspect invariant).
- [ ] Implement `verify(config, stage: Option<Stage>) -> VerificationReport`: walk a stage (or all stages), verify every file, report per-file status.
- [ ] Unit tests: valid file verifies; tampered input fails; tampered output fails; tampered metadata fails; broken stage-stamp chain fails; multi-stage-stamp chain validates; from_file-refuses-tampered.
- [ ] Acceptance: `cargo test -p einmo --lib verify` passes.

## Phase 6 — `EinmoSuite` + the generalised `Evaluator` (test runner writes signed `.einmo`)

The trait is einmo's own, defined from scratch — `Vec<String>`, no `FirRef`,
no dependency on any Foolish crate:

```rust
pub trait Evaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String>;
}
```

- [ ] Create `einmo/src/snapshot_suite.rs` with `EinmoSuite::new(config)`.
- [ ] Implement `EinmoSuite::evaluate(path, evaluator)`: read the input file, call `evaluator.evaluate(&source)`, compute configured perspective sections (§4.5), assemble the envelope (metadata + INPUT + one OUTPUT section per returned string + perspectives + COMMENTS), stamp with compiled + configured + `stage:output` (§4.4), write to `output/<mirror-path>`. Metadata `producer` = current commit SHA (+ `producer-diff` when the tree is dirty).
- [ ] **Error capture (spec §4.2)**: evaluator `Err(String)` on parse/accept → `status: input-error`; panic (`catch_unwind`) or abnormal evaluation → `status: output-error`; both with maximal `status-detail` and still a stamped, reviewable `.einmo` in `output/`. Expected error *outputs* (e.g. "infinite loop detected", NK alarms) are `status: normal` — promotable to `verified/`.
- [ ] **Output churn (accepted tradeoff)**: the runner rewrites `output/` unconditionally every run (fresh timestamps → fresh stamp bytes). Do not add skip-if-unchanged logic; revisit only if churn becomes a real problem (spec §B.3).
- [ ] Implement `EinmoSuite::evaluate_inline(name, input, evaluator)`: input is a string in code; captured into the INPUT section and stamped; `name` becomes the filename. Inline **expected values** are refused by design (no API for them).
- [ ] Implement `evaluate_all(evaluator)` / `evaluate_all_inline(pairs, evaluator)` returning `TestResults`, running parallel or serial per `TestConfig::parallel`; enforce `require_correspondence` pairs via `compare` (Phase 8) and fail with a per-file diff on mismatch. (Until Phase 8 lands, correspondence enforcement may be a stub that always errors "compare not yet implemented" — wire it in Phase 8.) No special-case bootstrap messaging: an empty `checked/` simply fails correspondence until someone promotes.
- [ ] Unit tests (use a trivial in-test `Evaluator` impl, e.g. an integer-arithmetic echo): output written + stamped + verifies; Err → input-error/output-error status with detail; panic captured; inline input captured; mirror path respected; perspective section emitted; parallel and serial modes agree.
- [ ] Acceptance: `cargo test -p einmo --lib snapshot_suite` passes.

## Phase 7 — Promotion + flagging (move/copy semantics)

- [ ] Add transition functions to `einmo/src/stage.rs` (or a `transitions.rs` module).
- [ ] `promote(config, from, to, key_source) -> Result<PromotionReport>` — every promotion APPENDS the destination stage's stamp over all prior bytes (existing stamps untouched):
  - `output->checked`: copy file `output/<rel>` → `checked/<rel>` + append `stage:checked` stamp (checked stage key — configured, no prompt). Verify-on-inspect the source first.
  - `*->verified`: copy file → `verified/<rel>` + append `stage:verified` stamp (stage key resolved via cascade; promotion timestamp inside the stamp). Warn if the stamp pubkey equals a well-known computer key (non-human attestation).
  - `*->flagged`: same as `flag` below (move, advisory line, NO stamp).
  - Refuse if source file fails verify-on-inspect. Refuse to append if any existing stamp fails (chain integrity).
- [ ] `flag(config, stage, filter, reason) -> Result<FlagReport>`:
  - Move file `<stage>/<rel>` → `flagged/<rel>` (REMOVE from origin, CREATE in `flagged/`).
  - Collision: if `flagged/<rel>` exists, suffix the new file with timestamp: `flagged/<rel-no-.einmo>.<ISO8601>.einmo`.
  - Append advisory `# flagged: <reason> <ISO8601>` line OUTSIDE signed content (so original sigs stay valid; do NOT re-sign).
  - Verify-on-inspect the source before moving.
- [ ] `confirm_signatures(path, pubkey_prefix) -> SignatureReport`: scan all `.einmo` under `path`; report files carrying a signer whose pubkey starts with `prefix`. `--require-all` → non-zero exit if any file lacks a match.
- [ ] Unit tests: promote output->checked appends stage:checked and preserves prior stamps; promote checked->verified appends stage:verified with signed timestamp; promote refuses on tampered source; promote refuses on broken chain; flag moves file (origin vacated); flag collision → timestamp suffix; flag advisory line outside signed bytes (stamps still valid); confirm-signatures matches prefix; confirm-signatures --require-all exits non-zero on missing.
- [ ] Acceptance: `cargo test -p einmo --lib promote flag confirm_signatures` passes.

## Phase 8 — `compare` (per-section matching, verify-both-then-identical)

- [ ] Create `einmo/src/compare.rs`.
- [ ] Implement `compare(config, a, b, sections) -> ComparisonResult`:
  - Walk both stage trees in parallel (by mirror-relative path).
  - For each path in both: load file A via `EinmoFile::from_file` (verify-on-inspect); load file B same. If either fails verification → add to `tampered` (NOT `differing`); skip content comparison.
  - If both verify: compare configured sections byte-for-byte. `InputOutput` → INPUT + every `OUTPUT[i]`. `InputOutputComments` → also COMMENTS. Perspective sections optionally per config. STAMPS and metadata are NOT compared.
  - Result per path: `matching` (configured sections identical), `differing` (a configured section differs — record which section(s)), `only_in_a`, `only_in_b`, `tampered`.
- [ ] Wire `require_correspondence` enforcement in `EinmoSuite::evaluate_all` (Phase 6 stub) to this `compare`.
- [ ] Implement `--root-cause` support: for each `differing` file, descend its subtree (`--filter <subtree>/*`) and report the deepest `differing` descendants.
- [ ] Implement `--stale-days N`: warn about files in stage-b whose mtime is older than N days relative to stage-a.
- [ ] Unit tests: identical stages → all matching; missing files → only_in_a/only_in_b; content diff in OUTPUT → differing (names OUTPUT[i]); content diff in COMMENTS with InputOutput → matching (COMMENTS not compared); content diff in COMMENTS with InputOutputComments → differing (names COMMENTS); tampered file → tampered (not differing); stamps-only diff → matching (STAMPS excluded); root-cause descends to deepest differing.
- [ ] Acceptance: `cargo test -p einmo --lib compare` passes.

## Phase 9 — Key resolution cascade + configuration

- [ ] Implement `resolve_stage_key(stage, cli_pass, stdin_pass, interactive_flag, env, config) -> Result<KeySource, EinmoError>` in `einmo/src/config.rs`.
- [ ] Precedence: `--passphrase <v>` > `--stdin-passphrase` (read one line from stdin) > `EINMO_PASSPHRASE` env > `einmo.toml` `[signing.<stage>] passphrase` (per-stage; each stage may have a different configured key) > interactive prompt on `/dev/tty` (only if no tier yielded a value).
- [ ] `--interactive` flag forces the prompt (skips tiers 1–4).
- [ ] Explicit empty string (`--passphrase ""` or `EINMO_PASSPHRASE=""`) = "set to empty" (the well-known computer key), NOT "unset". To unset, omit entirely.
- [ ] Config-file parsing: `einmo.toml` with `[signing] configured-key`, per-stage `[signing.<stage>]` tables, envelope settings (`encoding`, `separator`), `parallel`, `[ci]`, `[review]`. Read from `.config/einmo.toml` or repo-root `einmo.toml`. Deployment convention: output/checked stage passphrases set to `""`; verified deliberately unset → prompt.
- [ ] Unit tests: CLI overrides env; env overrides per-stage config; per-stage config used per stage; empty-vs-unset distinction; --interactive forces prompt; no tier → prompt (mock /dev/tty); configured-key parsed.
- [ ] Acceptance: `cargo test -p einmo --lib config` passes.

## Phase 10 — CLI (the single `einmo` app)

- [ ] Create `einmo/src/main.rs` (the `einmo` binary). Use `clap` derive. Create `einmo/src/bin/cargo_einmo.rs` as a one-line alias binary `cargo-einmo` calling the same CLI entry point (cargo-subcommand convention: `cargo install einmo` → both `einmo …` and `cargo einmo …` work).
- [ ] Stage-pair arguments use ASCII `->` (`output->checked`); stage names validated `[A-Za-z0-9_-]+`.
- [ ] Global `--parallel <n>` / `--serial` (default from `einmo.toml`).
- [ ] Subcommands (each calls the library; every subcommand verifies-on-inspect any file it touches):
  - `promote <from>-><to> <work_dir> [--filter] [--passphrase|--stdin-passphrase|--interactive] [--batch]`
  - `flag <work_dir> <stage> [--filter] [--reason]`
  - `compare <stage-a> <stage-b> <work_dir> [--match-sections] [--require-comments-match] [--stale-days] [--filter] [--require-match] [--json] [--root-cause]`
  - `verify <work_dir> [--stage|--all]`
  - `verify-signatures <path> [--write-verified] [--stdin-passphrase]` — signature verification is a subcommand of the one app, NOT a separate binary. (The existing `foolish-core/src/bin/verify_signatures.rs` continues to serve the legacy `.snap` corpus and is NOT touched.)
  - `confirm-signatures <path> <pubkey-prefix> [--require-all]`
  - `show <file>` — envelope summary incl. stamp chain with `produced_by` provenance per stamp
  - `console-review <work_dir> <from>-><to> [--filter] [--full] [--reexamine-rate] [--reexamine-seed] [--vim|--list] [--root-cause]` (Phase 12)
  - `serve <work_dir> [--bind]` (Phase 13)
  - `self-check [--expected <sha256>] [--quiet]` — computes SHA-256 of `env::current_exe()?`, prints path + hash; `--expected` exits non-zero on mismatch; `--quiet` prints only the hash. Also reads an expected hash from a sidecar `einmo.sha256` next to the binary if present.
- [ ] Every verb supports `--json` machine output (stable scriptable surface).
- [ ] Every stamp the CLI writes carries `produced_by: "einmo <version> sha256:<self-hash>"` (§4.4 — provenance is a stamp field; there is no separate advisory line).
- [ ] Acceptance: `cargo build -p einmo --bins` succeeds and produces `einmo` + `cargo-einmo`. Manual: `einmo verify <test-suite>` exits 0 on a clean suite; `cargo einmo verify <test-suite>` behaves identically; `einmo self-check` prints the binary's SHA-256; `einmo self-check --expected <wrong-hash>` exits non-zero; `einmo show` on a generated `.einmo` displays the stamp chain with produced_by fields.

## Phase 11 — Gates (shell glue)

- [ ] Create `einmo/scripts/einmo-pre-commit.sh`:
      ```sh
      #!/bin/sh
      einmo compare output checked . --require-match || {
        echo "einmo: output does not match checked. Promote (review) or repair."
        echo "  burden: the producer of the divergent output must repair or escalate."
        exit 1
      }
      ```
- [ ] Create `einmo/scripts/einmo-pre-tag.sh`:
      ```sh
      #!/bin/sh
      set -e
      einmo compare checked verified . --require-match
      einmo confirm-signatures verified "$RELEASE_KEY_PREFIX" --require-all
      ```
- [ ] Create `.github/workflows/einmo-gates.yml` (merge gate: `verify --all` + `compare checked verified --require-match` on PRs). Scope the workflow to the einmo/zweimomo suites only — it must not gate the legacy `.snap` corpus.
- [ ] Document the burden-of-correction messages in each gate's failure output.
- [ ] Acceptance: install the pre-commit hook locally; commit a divergence → blocked; promote → commit succeeds. The GH Actions YAML is valid (`actionlint` or equivalent).

## Phase 12 — `console-review` (vimdiff, diff -I, randomized re-inspection, @agent handling)

- [ ] Implement `console_review(config, from, to, opts) -> Result<ReviewReport>` in `einmo/src/review.rs` (new module; the CLI subcommand calls this).
- [ ] Demotion: for files that genuinely differ (`compare from to` → `differing`), demote from `to` back to `from` (move; all stamps preserved as history).
- [ ] **Randomized re-inspection**: `--reexamine-rate <pct>` (default 10 with `--full`): also pick `pct`% of files already in `to` (random sample, seeded by `--reexamine-seed`), demote them to `from`, re-present for review. Use a deterministic PRNG seeded by (work-dir-path + date) when no explicit seed. Re-promotion appends ANOTHER stage stamp (attributable re-inspection).
- [ ] Review presentation modes:
  - `--vim` (default): invoke `vimdiff <from-file> <to-file>` with `diff -I` ignoring the STAMPS-section JSON lines (`^\{"key":`).
  - `--list`: print file paths one per line (for shell-script pipelining).
- [ ] In-file annotation handling: scan each reviewed file for `@agent`:
  - `@agent, skip` → defer (skip this round, leave in `from`).
  - `@agent` (without skip) → flag the file (`flag` to `flagged/`).
  - neither → promote (`from->to`) after the human marks acceptable.
- [ ] Final sanity grep: `@agent` in `to/` after review (report any missed).
- [ ] `--root-cause`: on a differing file, descend its subtree; report deepest differing descendants.
- [ ] Unit tests (library-level): demote moves file; reexamine-rate samples N%; reexamine-seed reproduces sample; re-promotion appends another stage stamp; @agent-skip defers; @agent flags.
- [ ] Acceptance: `cargo test -p einmo --lib review` passes. Manual: `einmo console-review <suite> output->checked --list` lists differing files.

## Phase 13 — `serve` (Proposal A: axum + SPA)

- [ ] Add deps to `einmo/Cargo.toml`: `axum`, `tokio`, `tower`, `tower-http`, `rust-embed` (feature-gated under `serve` so the core library stays lean).
- [ ] Create `einmo/src/serve.rs` + the `serve` subcommand in `main.rs`.
- [ ] Endpoints (REST, loopback, auth-gated):
  - `GET /api/tree` — suite overview (input tree + per-file stage badge + signature status).
  - `GET /api/diff?a=<stage>&b=<stage>&rel=<path>` — per-section diff (signature lines hidden).
  - `POST /api/promote` `{from, to, filter, passphrase?}` — promote (passphrase brokered to library, derived, discarded; never stored).
  - `POST /api/flag` `{stage, filter, reason}`.
  - `GET /api/verify` `GET /api/confirm-signatures` `GET /api/show?path=<path>`.
  - `POST /api/console-review` (review-queue state).
  - `WS /ws/alerts` — stream alerts (output≠checked, checked≠verified, flagged, staleness, signature failures).
- [ ] Serve a static SPA bundle (Vite+React or SvelteKit) via `rust-embed` at `/`. Rich-output rendering per FOOP-92 §C.4 (einmo-in-HTML metadata convention; byte-steadiness invariant).
- [ ] The server NEVER holds keys persistently; passphrase arrives via POST body, is derived to a key, used to sign, dropped.
- [ ] Acceptance: `einmo serve <suite> --bind 127.0.0.1:0` starts; `curl /api/tree` returns JSON; a promote via POST appends the destination stage's stamp.

## Phase 14 — `zweimomo` crate: three pure-Rust `Evaluator` impls

Zweimomo is einmo's companion test crate (FOOP-92 §Use Case D). It embeds three
**pure-Rust** interpreters (rationale: no C/FFI toolchain — keeps the harness
portable and einmo repo-promotable). See Appendices G/H/I for the per-language
embedding references gathered during research.

- [ ] Create `zweimomo/Cargo.toml`:
      - package name `zweimomo`, edition 2021, **semantic versioning** starting `0.1.0`.
      - `[dependencies]`: `einmo = { path = "../einmo" }`, `foolish-ubca = { path = "../foolish-ubca" }`, `foolish-core = { path = "../foolish-core" }`, `rustpython-vm = "=0.5.0"`, `boa_engine = "=0.21.1"` (**exact pins** — interpreter output text is part of signed baselines; bumping a pin is at least a minor zweimomo version bump and implies a corpus re-review).
      - (zweimomo depends on the foolish crates; einmo itself never does.)
- [ ] Add `zweimomo` to the workspace `members` in the repo-root `Cargo.toml`.
- [ ] Create `zweimomo/src/lib.rs` + `zweimomo/src/evaluators.rs` with the three impls of `einmo::Evaluator` (embedding sketches in FOOP-92 §D.3):
  - `UbcaEvaluatorAdapter` — wraps the existing `foolish_ubca::UbcaEvaluator` (used as-is, NOT modified) and formats each returned FIR via `foolish_core::FirSequencer::format` → `Vec<String>`.
  - `RustPythonEvaluator` — `Interpreter::without_stdlib(Default::default())` per call (sandboxed: no os/file I/O); `eval::eval(vm, source, scope, "<zweimomo>")`; stringify via `result.str(vm)?.as_str().to_string()`. Note: `Interpreter` is not `Send` — construct inside `evaluate` (per call) so parallel `evaluate_all` threads each own their interpreter.
  - `BoaEvaluator` — `Context::default()` per call (sandboxed: no fs/network/Node APIs); `context.eval(Source::from_bytes(source))`; stringify via `.to_string(&mut context)?.to_std_string_escaped()`. Same not-`Send` note.
- [ ] **Serialization is zweimomo's job** (spec §D.2): each adapter decides how its interpreter's values become text chunks — **use what is most colloquial in each language** (idiomatic evaluation mode, idiomatic stringification). Document each adapter's choice in its rustdoc.
- [ ] Implement the **brane-name perspective** for the Foolish suite (spec §4.5): `{a=1,b=2,c=3}` → `{a=???,b=???,c=???}` — supplied to `TestConfig::perspectives` as a pure `fn(&str) -> String`.
- [ ] Unit tests (per evaluator, inline): `"1 + 2"`-style smoke test for Python (`"3"`) and JS (`"3"`); `{3 + 4;}` for Foolish (hfssnap output non-empty); error paths return `Err(String)` (Python exception, JS throw, Foolish parse error) rather than panicking; brane-name perspective extraction.
- [ ] Acceptance: `cargo test -p zweimomo --lib` passes. `git diff --stat` shows no change to `foolish-ubca/` or `foolish-core/`.

## Phase 15 — Zweimomo parallel test-input corpus + einmo suites

- [ ] **Read the existing snap inputs for inspiration and syntax guidance**: browse `foolish-ubca/snapshot_tests/input/*.foo` (~145 files) before writing any Foolish input. The Foolish column of the matrix must mirror idioms already proven in that corpus (see FOOP-92 §D.6 and Appendix G.5); the Python/JS columns are then written to match. Confirmed corpus syntax includes the `=$` bind-tail calling form (`regression_disappearing_brane_statements.foo`).
- [ ] Create the three suite work-dirs, one `EinmoSuite` per language:
      ```
      zweimomo/suites/foolish/input/      (*.foo)
      zweimomo/suites/python/input/       (*.py)
      zweimomo/suites/javascript/input/   (*.js)
      ```
      (each gains `output/`, `checked/`, `flagged/`, `verified/` siblings; all git-tracked.)
- [ ] Write the parallel inputs for each concept row of FOOP-92 §D.4 — the matrix is bounded by the Foolish capability ceiling (Appendix G): integer arithmetic; nested expressions/parsing; name binding + scope; data structures/nesting; **function application** (Foolish: concatenation + `=$` tail-binding per §D.5; Python/JS: `fn(args)`); division-by-zero/error behaviour; search/query (Foolish-specific, dict/object access on the Python/JS side); SF/SFF laziness (Foolish-only). Integer arithmetic only (Foolish is `u64`, `/` is integer division — Python uses `//`, JS uses `Math.floor` where needed to match).
- [ ] Excluded concepts (no Foolish representation today — Appendix G.2): strings, floats, booleans, recursion, loops, `if/else`, closures. Do not force fake parallels; where a Python/JS-only demonstration is valuable, mark it Foolish-unsupported in the input's comments.
- [ ] Write the zweimomo tests using einmo: one `#[test]` per language calling `EinmoSuite::evaluate_all` with the language's evaluator and `require_correspondence(Output, Checked)`. Each language is gated independently — NO cross-language byte comparison (the three output formats differ by design, FOOP-92 §D.7).
- [ ] Configure the Foolish suite's separator as `"!!"`+LF (a Foolish line comment; spec §4.1); python/javascript suites use the default `①`+LF.
- [ ] Generate the first `output/` corpus; review the diffs; promote via `einmo promote output->checked zweimomo/suites/<lang>` (this is the dog-food moment: einmo's own CLI promotes einmo's own test corpus).
- [ ] Acceptance: `cargo test -p zweimomo` passes with all three suites green (`output==checked` per language). `einmo verify zweimomo/suites/<each> --all` green.

## Phase 16 — Exhaustive algorithm coverage (later in development; stress-test the framework)

The important goal: **test the test framework as thoroughly as possible.** Port
algorithm implementations exhaustively from
[TheAlgorithms](https://github.com/TheAlgorithms) (and other well-known
collections, e.g. Rosetta Code) into the zweimomo corpus, so einmo is exercised
by a large, diverse, realistic body of inputs rather than a handful of
hand-picked examples.

**Licensing rule (verified 2026-07-03):** TheAlgorithms/**Python** and
TheAlgorithms/**Rust** are **MIT** — portable with per-file attribution.
TheAlgorithms/**JavaScript** is **GPL-3.0** — its code is **NOT copied** into
this repo (keeps the repo and the future einmo repo licence-clean). JavaScript
inputs are written ourselves, translated from the MIT Python implementations.
Foolish inputs are written ourselves by definition. Re-verify each source
repo's license at port time.

- [ ] Select an initial algorithm set from TheAlgorithms/Python (MIT): sorting, searching, math/number theory, dynamic programming, string algorithms (Python/JS only), graph algorithms. Record source attribution (repo + path + license) in each input's comments.
- [ ] Port the Python implementations into `zweimomo/suites/python/input/algorithms/…` (mirroring TheAlgorithms' own directory taxonomy — einmo's hierarchical stage trees exist precisely for this).
- [ ] Write the JavaScript equivalents into `zweimomo/suites/javascript/input/algorithms/…` — **translated by us from the MIT Python versions, never copied from the GPL-3.0 JavaScript repo**.
- [ ] Write Foolish equivalents into `zweimomo/suites/foolish/input/algorithms/…` **where the language allows** (arithmetic- and data-structure-heavy algorithms; no loops/recursion/conditionals yet — most algorithms will be Python/JS-only until Foolish grows control flow; note the asymmetry per input).
- [ ] Adjust inputs for determinism: RustPython runs `without_stdlib` (no imports), Boa has no Node APIs — inputs must be self-contained expressions/scripts with deterministic output (no randomness, no timing, no I/O).
- [ ] Run the full corpus through `evaluate_all`; review + promote in batches (`--filter algorithms/sorting/*` etc. — this stress-tests filtering, batch promotion, hierarchical mirroring, and `compare --root-cause` at realistic scale).
- [ ] Feed back: every awkwardness found while operating at scale (slow verify, unwieldy diffs, promotion ergonomics, directory-collision edge cases) becomes a design-revision item for this FOOP.
- [ ] Acceptance: corpus of ≥100 algorithm inputs across the two full-capability languages; `cargo test -p zweimomo` and `einmo verify … --all` green at that scale; wall-clock for `evaluate_all` + `compare` recorded in the plan notes.

## Phase 17 — Use-case enumeration and validation (feeds user docs + design feedback)

Work through every way einmo is used, to (1) prepare user documentation and (2)
validate the design (awkward use cases reveal design gaps before code freezes).

- [ ] Enumerate the use-case matrix: {tier} × {actor} × {operation} × {granularity}. Tiers: unit / approval / cross-language (zweimomo) / integration / CI / release / regression-reinspection / performance-redacted. Actors: coding agent / human reviewer / release officer / CI / auditor. Operations: evaluate / promote / flag / compare / verify / confirm-signatures / console-review / serve. Granularity: leaf (unit) / mid-tree (suite subdir) / root (whole-corpus).
- [ ] For each matrix cell, write a worked example: exact `TestConfig`, CLI invocation(s), expected `.einmo` transitions, gate behavior. Ground in the zweimomo suites (leaf = a single concept input; mid-tree = `suites/python/algorithms/sorting/`; root = a whole-language suite).
- [ ] Exercise the hierarchical-granularity diagnostic: construct a synthetic tree where a leaf change cascades to a root mismatch; verify `compare --root-cause` identifies the deepest differing descendant.
- [ ] Exercise the gate lifecycle: commit (output==checked) → PR → merge (checked==verified) → tag (confirm-signatures <release-key> --require-all). Verify burden-of-correction messages on each failure mode.
- [ ] Exercise randomized re-inspection: seed-pinned 10% demote-and-re-review on a verified corpus; verify re-promotion appends another stage stamp; verify a rejection (flag) surfaces baseline rot.
- [ ] Exercise per-section matching: a suite with `--require-comments-match` and one without; verify COMMENTS drift is caught/ignored per config; INPUT/OUTPUT always required.
- [ ] Exercise emergent human-attestation: an AI runs `promote checked->verified --passphrase ""` → confirm the stage:verified stamp pubkey is the well-known computer key → `confirm-signatures verified <release-key>` catches it.
- [ ] Exercise the UI flow (`serve` + SPA): list / diff / approve / flag / alert-feed / confirm-signatures over the worked examples; verify crypto boundary (UI never holds keys).
- [ ] Draft the user-documentation outline from the worked examples (Getting Started / Configuring a Suite / The Four Stages / Promotion & Flagging / Gates & CI / The Granularity Diagnostic / Review & Re-inspection / UI / Troubleshooting).
- [ ] Feed back: any awkward use case becomes a design-revision item for this FOOP before code freezes.

## Phase 18 — Final verification

- [ ] `cargo fmt --all --check` passes.
- [ ] `cargo clippy --workspace -- -D warnings` passes.
- [ ] `cargo test --workspace` passes (all unit + snapshot tests green — including the untouched legacy `insta` suites, proving zero regression in existing crates).
- [ ] `git diff --stat jia...` confirms the scope rule: changes only in `einmo/`, `zweimomo/`, `.gitignore`, `.gitattributes`, root `Cargo.toml` members, `.github/workflows/einmo-gates.yml`, and `docs/foop/FOOP-92*`.
- [ ] `cargo build -p einmo --release` succeeds; `target/release/einmo` exists.
- [ ] `einmo verify zweimomo/suites/<each> --all` green.
- [ ] End-to-end: generate (`output/` with compiled+configured+stage:output stamps) → review → `promote output->checked` (appends stage:checked) → `promote checked->verified --interactive` (human; appends stage:verified) → `verify` (full stamp chain ok) → merge gate `compare checked verified --require-match` passes.
- [ ] End-to-end gate failure: introduce an output divergence → pre-commit hook blocks the commit with the burden-of-correction message.
- [ ] End-to-end AI-bypass: `promote checked->verified --passphrase ""` → `confirm-signatures verified <release-key> --require-all` exits non-zero (stage:verified stamp pubkey is the well-known computer key).
- [ ] End-to-end re-inspection: `console-review checked->verified --reexamine-rate 10 --reexamine-seed 42` → demotes 10% → re-promote appends another stage:verified stamp.

## Phase 19 — Merge

- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-92-einmo and committed to `foop-92-einmo`.
- [ ] Merge `foop-92-einmo` to `jia`.
  - [ ] If merge conflicts arise on `jia`, repair all tests in /home/hcbusy/foolish-rust.
  - [ ] STOP — ask human to check this box before continuing.
- [ ] After merge: confirm `jia` CI green (`einmo verify --all` always-on; merge gate `compare checked verified` enforced on PRs).
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-92-einmo.
  - [ ] Check that this plan file has all but the Cleanup checkboxes completed.
  - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-92-einmo.
  - [ ] This is the last sub-task checkbox to be checked.

## Deferred

- [ ] **Legacy `.snap` corpus migration:** the 276 `.snap` + 285 `.snap.new` files in `foolish-core`/`foolish-ubca` stay on `insta` for now. Once einmo is merged and stable, a future separate effort writes a `.snap → .einmo` converter, migrates the corpus (`.snap` → `checked/`, `.snap.new` → `output/`), retires the insta dev-dependency, and a human runs the re-sign pass (`promote checked->verified --interactive`). Human-coordinated, not part of this FOOP.
- [ ] **MCP server** (`einmo serve --mcp` or `einmo-mcp`, FOOP-92 §C.3) + the AGENTS.md review-flow skills template — after the REST `serve` stabilises (both are frontends over the same library).
- [ ] **OQ (structured flag log):** flag annotation is an in-file advisory `# flagged:` line. If richer querying is later needed, consider a JSON-lines sidecar — not blocking.
- [ ] **Desktop app (Tauri vs egui):** both reuse `einmo serve`'s REST API unchanged; decide when needed.
- [ ] **WASM verify in browser (Proposal C):** `einmo::verify` is already a clean submodule (Phase 5). Compile to `wasm32-unknown-unknown` when the browser-side verify is wanted; the signing server (`serve`) stays as-is.
- [ ] **Per-stage passphrase config:** `[signing.<stage>]` overrides in `einmo.toml` — deferred; the cascade + deployment convention (omit verified passphrase) suffices initially.
- [ ] **Rune as a fourth evaluator:** pure-Rust, viable; deferred — RustPython + Boa already prove language-agnosticism with more widely known languages. `mlua` remains rejected (default backend links C Lua).
- [ ] **JVM cross-validation:** dropped per FOOP-03. If JVM implementations are un-deprecated, the per-impl stage-dir model (`output-rust/`/`output-java/` with cross-`compare`) is the structure.

## Conventions for the implementing agent

- **Never** modify any existing crate. The scope rule at the top of this plan is absolute: `einmo/`, `zweimomo/`, `.gitignore` (Phase 0), root `Cargo.toml` members, and the gates workflow are the only writable locations outside `docs/foop/`.
- **Never** run `cargo insta accept` or set `INSTA_UPDATE=always`. (Einmo has no automated update; the legacy `insta` corpus is untouched by this FOOP — do not auto-accept anything there either.)
- **Never** suppress errors: no `unwrap()` on fallible crypto/format calls; use `?` and `EinmoError`.
- **Always** run `cargo fmt` and `cargo clippy -D warnings` before marking a phase complete.
- **Always** write the tamper/forgery tests BEFORE the signing write path (Phase 2).
- **Always** verify-on-inspect in every code path that reads a `.einmo` file — the library enforces this; do not add a "fast path" that skips verification.
- **Never** re-implement the passphrase cascade outside `einmo::config`. Shell/UI call `einmo` or `serve`; they never derive keys.
- When a task says "Acceptance:", that check MUST pass before the next task. If it fails, fix the failure; do not proceed.
- If a task is ambiguous, read FOOP-92.md (the specification) for the authoritative answer. This plan assumes the spec's context.

## Last Updated

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.199 (Claude Code); Fable 5
**Changes**: Second pass, folding in the pre-MVP design-review resolutions
(BDFL). Phase 2 rewritten for the three-role stamp chain (Compiled/Configured
certify pubkeys; Stage keys sign prior bytes and append). Phase 3 rewritten for
the new envelope (header line, configurable encoding/separator incl. Foolish
`"!!"`+LF, metadata with producer SHA + status/status-detail, JSON STAMPS,
collision refusal). Phase 6: error-status capture, perspectives, accepted
output churn (no skip-if-unchanged). Phase 7: every promotion appends the
destination stage's stamp. Phase 9: per-stage `[signing.<stage>]` cascade +
configured-key. Phase 10: single CLI app `einmo` + `cargo-einmo` alias
(`cargo install einmo`), `verify-signatures` as subcommand, ASCII `->` arrows,
`--parallel`, produced_by as stamp field (advisory line dropped). Phase 0:
`.gitattributes *.einmo -text`. Phase 14: exact interpreter pins
(`=0.5.0`/`=0.21.1`), zweimomo semver, colloquial-per-language serialization,
brane-name perspective. Phase 16: licensing rule (TheAlgorithms Python/Rust
MIT usable; JavaScript repo GPL-3.0 → JS inputs translated by us from the MIT
Python versions). MVP boundary marker added (Phases 0–10 + 14–15 = MVP).
RESULT→OUTPUT and test/util→stamp terminology swept throughout.

**Date**: 2026-07-03
**Updated By**: Claude Code 2.1.199 (Claude Code); Fable 5
**Changes**: Rewrote the plan to match the refreshed (standalone-scope) FOOP-92
spec. (1) Scope rule added: two new crates (`einmo`, `zweimomo`) only; no
existing crate modified; no `.snap` migration (moved to Deferred). (2) Phases 2–6
reframed from "copy/migrate from foolish-core" to "implement from scratch"
(signature → format → stages → verify → suite), matching the spec's §Build
order. (3) Dropped all `foolish-ubcb` tasks and `foolish/`-prefixed paths
(post-FOOP-03 flattened workspace). (4) Fixed the Phase 0 premise
(`verify_signatures.rs` exists and is tracked; the `.gitignore` fix protects
NEW `einmo/src/bin/` files). (5) New Phase 14 (zweimomo crate: three pure-Rust
`Evaluator` impls — UbcaEvaluatorAdapter, RustPythonEvaluator, BoaEvaluator) and
Phase 15 (parallel corpus per §D.4, with the mandatory "use existing snaps for
inspiration" first task). (6) New Phase 16: exhaustive algorithm coverage from
TheAlgorithms (and similar collections) to stress-test the framework at scale.
(7) Worktree block re-anchored: origin branch `jia`, origin path
`/home/hcbusy/foolish-rust` (previous origin path pointed at the stale FOOP-62
worktree; `alpha` branch does not exist).
