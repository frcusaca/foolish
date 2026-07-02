# FOOP-92 Plan — Einmo: directory-based signed-snapshot testing with staged promotion

This plan executes [FOOP-92](FOOP-92.md). **Read the specification first** —
the plan assumes its context. The design is frozen (all OQs resolved).

This plan is written for a smaller implementing LLM. Each task is atomic: one
concrete action, exact file paths, exact commands, and an acceptance check. Do
not combine tasks. Do not skip the acceptance check. Run `cargo fmt`, `cargo
clippy -D warnings`, and the relevant `cargo test` after each phase.

**Never auto-accept snapshots.** Never run `cargo insta accept` or
`INSTA_UPDATE=always`. Present `.einmo.new`/diffs to a human for review.

Branch and worktree lifecycle (per `foop.md`):

```
WORKTREE_ORIGIN_BRANCH=alpha
WORKTREE_ORIGIN_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo
WORKTREE_BRANCH_NAME=foop-92-einmo
WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-92-einmo
```

- [ ] begun
      ( ) <!-- timestamp when work commences -->

- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-92-einmo with branch `foop-92-einmo` off `alpha`
      ```bash
      cd /home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo
      git worktree add -b foop-92-einmo /home/hcbusy/tmp/foolish-worktrees/foop-92-einmo
      ```

## Phase 0 — Fix the `.gitignore` `bin/` bug (BLOCKING, do first)

The root `.gitignore` has a `bin/` pattern that ignores ANY `bin/` directory
including `src/bin/`. This is why `foolish-core/src/bin/verify_signatures.rs` is
missing. Fix before creating `einmo/src/bin/`.

- [ ] In the repo root `.gitignore`, change the `bin/` line to `/bin/` (anchors to repo-root `bin/` only) or `target/**/bin/` (only inside target). Verify with `git check-ignore -v foolish/einmo/src/bin/cargo_einmo.rs` (must print nothing — not ignored).
- [ ] Confirm `foolish-core/Cargo.toml` `[[bin]] name="verify_signatures"` no longer points at an ignored path. (The file is still missing — it will be restored in Phase 2.)

## Phase 1 — Crate scaffolding

- [ ] Create `foolish/einmo/Cargo.toml`:
      - package name `einmo`, edition 2021.
      - `[dependencies]`: `ed25519-dalek`, `argon2`, `base64`, `hex`, `clap` (derive), `serde`, `toml`, `time`, `thiserror`.
      - `[dev-dependencies]`: `tempfile`, `num_cpus`, `rayon` (feature-gated under `parallel`).
      - `[[bin]] name = "cargo-einmo" path = "src/bin/cargo_einmo.rs"`.
      - `[[bin]] name = "verify_signatures" path = "src/bin/verify_signatures.rs"`.
- [ ] Create `foolish/einmo/src/lib.rs` with module declarations: `mod config; mod stage; mod compare; mod format; mod signature; mod snapshot_suite; mod migrate; mod verify;` and public re-exports: `TestConfig, Stage, StageDirs, MatchSections, EinmoSuite, Evaluator, EinmoFile, SignerEntry, EinmoError, compare, promote, flag, verify, confirm_signatures`.
- [ ] Add `einmo` to the workspace `members` in `foolish/Cargo.toml`.
- [ ] Add `einmo = { path = "../einmo" }` as a dev-dependency in `foolish-core/Cargo.toml`, `foolish-ubca/Cargo.toml`, `foolish-ubcb/Cargo.toml`.
- [ ] Acceptance: `cargo check --workspace` passes (empty crate compiles; no code moved yet).

## Phase 2 — Migrate `signature.rs` (REPLACE → FOOP-22 append) — HIGHEST RISK

**Write tamper/forgery tests FIRST.** This is the single highest-risk step.

- [ ] Copy `foolish/foolish-core/src/signature.rs` (644 lines) to `foolish/einmo/src/signature.rs`. Leave the original in place for now (re-export in Phase 3).
- [ ] Add `SignerEntry` struct: `role: SignerRole` (`Test`/`Util`), `public_key_hex: String`, `input_sig_b64: String`, `result_sig_b64: String`, `comments_sig_b64: String`, `metadata_sig_b64: String`, `entire_file_sig_b64: Option<String>`. (Note: renamed from `foolish_sig`/`hs_sig`/`comments_sig` to `input_sig`/`result_sig`/`comments_sig`/`metadata_sig` per the per-section format in FOOP-92 §4.)
- [ ] Add `SnapshotSignatures { entries: Vec<SignerEntry> }`.
- [ ] Implement `parse_snapshot_signatures` (replaces `parse_snapshot_footer`): handle BOTH the legacy flat footer (→ single `Test` entry, no `Entire file`) AND the new indented `* Signed by …` format.
- [ ] Implement `SnapshotSignatures::format_footer()` rendering the indented format (FOOP-92 §4).
- [ ] **Change `verify_signatures --write-verified` from REPLACE to APPEND**: a new `Util` entry is appended; existing `Test`/`Util` entries preserved byte-for-byte. This is the core behaviour change.
- [ ] `--write-verified` refuses to append if ANY existing signature fails (verify all entries first; chain integrity).
- [ ] Add the `Metadata` signature (over `canon_input + canon_result + canon_comments + canon_metadata`) — the generation timestamp is now signed.
- [ ] Add the promotion timestamp inside the `Util` entry's signed content (signed).
- [ ] **Tamper tests** (write these FIRST, before changing the write path): parse legacy, parse new, parse multi-util, format roundtrip, legacy→new migration, verify `Test` entry, verify `Util` entry + `Entire file`, tamper-input-detection, tamper-result-detection, tamper-comments-detection, tamper-metadata-detection, tamper-after-promotion-invalidates-util, chain-integrity-refuses-on-broken-sig, append-preserves-test-entry.
- [ ] Acceptance: `cargo test -p einmo --lib signature` passes (all tamper tests green). `cargo test -p foolish-core --lib signature` still passes (re-export not yet changed).

## Phase 3 — Generalise `snapshot_suite.rs` and migrate `Evaluator`

- [ ] Copy `foolish/foolish-core/src/snapshot_suite.rs` (258 lines) to `foolish/einmo/src/snapshot_suite.rs`.
- [ ] **Generalise the `Evaluator` trait** to return `Result<Vec<String>, String>` instead of `Result<Vec<FirRef>, String>`. The library now has NO `FirRef` dependency.
- [ ] In `foolish-core/src/ubc_snapshot_tester.rs`, update `UbcEvaluator` to format FIRs to strings internally (via `FirSequencer::format` or equivalent) and return `Vec<String>`.
- [ ] In `foolish-ubca/src/ubca_snapshot_tester.rs`, update `UbcaEvaluator` the same way.
- [ ] In `foolish-ubcb/src/ubcb_snapshot_tester.rs`, update `UbcbEvaluator` the same way.
- [ ] Move `foolish-core/src/bin/verify_signatures.rs` (currently MISSING — recreate it) to `foolish/einmo/src/bin/verify_signatures.rs`. Update `foolish-core/Cargo.toml` to remove its `[[bin]] verify_signatures` (now in `einmo`).
- [ ] In `foolish-core/src/lib.rs`, re-export `EinmoSuite, Evaluator, signature::*` from `einmo` so existing test modules compile unchanged.
- [ ] Acceptance: `cargo test -p foolish-core --lib` passes (all existing signature + snapshot tests green). `cargo test -p foolish-ubca --lib` passes. `cargo test -p foolish-ubcb --lib` passes.

## Phase 4 — The `.einmo` format (parse/serialize, per-section canonical)

- [ ] Create `foolish/einmo/src/format.rs`.
- [ ] Define the `.einmo` file structure per FOOP-92 §4: `--- metadata ---` block (generated, suite, input_path), `--- INPUT ---` fenced block, `--- RESULT ---` fenced block, `--- COMMENTS ---` fenced block, `--- SIGNATURES ---` block.
- [ ] Implement `EinmoFile::parse(text) -> Result<EinmoFile, EinmoError>`: extract each section; canonicalise (canonical input, canonical result, canonical comments, canonical metadata).
- [ ] Implement `EinmoFile::serialize() -> String`: render the format.
- [ ] Implement per-section canonicalisation functions: `canon_input`, `canon_result`, `canon_comments`, `canon_metadata` (each returns a `Vec<u8>` ready for signing).
- [ ] Unit tests: parse roundtrip, parse-with-missing-section-errors, parse-legacy-flat-footer (→ single Test entry), parse-indented-multi-signer, serialize-stable, section-boundary-detection.
- [ ] Acceptance: `cargo test -p einmo --lib format` passes.

## Phase 5 — Stage directories + hierarchical mirroring

- [ ] Create `foolish/einmo/src/config.rs` and `foolish/einmo/src/stage.rs`.
- [ ] Define `Stage` enum (`Output, Checked, Flagged, Verified`), `StageDirs` (output/checked/flagged/verified defaults), `TestConfig` (work_dir, input_dir, stages, require_correspondence, match_sections), `MatchSections` (`InputResult`, `InputResultComments`).
- [ ] Implement `Stage::dir_name()` and `TestConfig::stage_dir(stage) -> PathBuf`.
- [ ] Implement `mirror_input_path(input_rel_path) -> stage_rel_path`: given `stage1/section3/specific.test`, produce `stage1/section3/specific.test.einmo`. (Append `.einmo` to the input-relative path.)
- [ ] Implement `walk_input_tree(config) -> Vec<PathBuf>`: discover all input files under `input/`, return their mirror-relative paths.
- [ ] Implement `ensure_stage_dirs(config)`: create `output/`, `checked/`, `flagged/`, `verified/` (and their mirrored subtrees on demand).
- [ ] Unit tests: flat input → flat stage paths; hierarchical input → mirrored stage paths; same-basename-different-branches coexist; `stage_dir` per stage.
- [ ] Acceptance: `cargo test -p einmo --lib stage` passes.

## Phase 6 — Verify-on-inspect + `verify`

- [ ] Create `foolish/einmo/src/verify.rs` (clean submodule: NO filesystem, NO tty, NO argon2 — only pure verify over parsed `EinmoFile` + `SnapshotSignatures`). This keeps Proposal C (WASM verify) available later.
- [ ] Implement `verify_all(einmo_file) -> Vec<SignerVerification>`: verify every signer entry; for each, check `input_sig`, `result_sig`, `comments_sig`, `metadata_sig` against the canonical sections; check `entire_file_sig` (if present) against all file bytes before the entry.
- [ ] Implement `EinmoFile::from_file(path) -> Result<Self, EinmoError>`: read file, parse, **verify all signatures**; return `Err` if any fail (verify-on-inspect invariant).
- [ ] Implement `verify(config, stage: Option<Stage>) -> VerificationReport`: walk a stage (or all stages), verify every file, report per-file status.
- [ ] Unit tests: valid file verifies; tampered input fails; tampered result fails; tampered metadata fails; broken entire-file-chain fails; multi-util chain validates; from_file-refuses-tampered.
- [ ] Acceptance: `cargo test -p einmo --lib verify` passes.

## Phase 7 — Promotion + flagging (move/copy semantics)

- [ ] Create `foolish/einmo/src/stage.rs` transition functions (or a `transitions.rs` module).
- [ ] `promote(config, from, to, key_source) -> Result<PromotionReport>`:
  - `output→checked`: copy file `output/<rel>` → `checked/<rel>`; `Test` entry preserved; NO new signature. Verify-on-inspect the source first.
  - `*→verified`: copy file → `verified/<rel>`; APPEND `Util` entry (sign with resolved passphrase); add signed promotion timestamp inside `Util`'s signed content. Warn if `Util` pubkey == `Test` pubkey (non-human attestation).
  - `*→flagged`: same as `flag` (Phase 7 flag below).
  - Refuse if source file fails verify-on-inspect. Refuse if `*→verified` and any existing signature fails (chain integrity).
- [ ] `flag(config, stage, filter, reason) -> Result<FlagReport>`:
  - Move file `<stage>/<rel>` → `flagged/<rel>` (REMOVE from origin, CREATE in `flagged/`).
  - Collision: if `flagged/<rel>` exists, suffix the new file with timestamp: `flagged/<rel-no-.einmo>.<ISO8601>.einmo`.
  - Append advisory `# flagged: <reason> <ISO8601>` line OUTSIDE signed content (so original sigs stay valid; do NOT re-sign).
  - Verify-on-inspect the source before moving.
- [ ] `confirm_signatures(path, pubkey_prefix) -> SignatureReport`: scan all `.einmo` under `path`; report files carrying a signer whose pubkey starts with `prefix`. `--require-all` → non-zero exit if any file lacks a match.
- [ ] Unit tests: promote output→checked preserves test entry; promote checked→verified appends util with signed timestamp; promote refuses on tampered source; promote refuses on broken chain; flag moves file (origin vacated); flag collision → timestamp suffix; flag advisory line outside signed content (sigs still valid); confirm-signatures matches prefix; confirm-signatures --require-all exits non-zero on missing.
- [ ] Acceptance: `cargo test -p einmo --lib promote flag confirm_signatures` passes.

## Phase 8 — `compare` (per-section matching, verify-both-then-identical)

- [ ] Create `foolish/einmo/src/compare.rs`.
- [ ] Implement `compare(config, a, b, sections) -> ComparisonResult`:
  - Walk both stage trees in parallel (by mirror-relative path).
  - For each path in both: load file A via `EinmoFile::from_file` (verify-on-inspect); load file B same. If either fails verification → add to `tampered` (NOT `differing`); skip content comparison.
  - If both verify: compare configured sections byte-for-byte (canonical). `InputResult` → compare `canon_input` + `canon_result`. `InputResultComments` → also `canon_comments`. SIGNATURES and metadata are NOT compared.
  - Result per path: `matching` (configured sections identical), `differing` (a configured section differs — record which section(s)), `only_in_a`, `only_in_b`, `tampered`.
- [ ] Implement `--root-cause` flag: for each `differing` file, descend its subtree (`--filter <subtree>/*`) and report the deepest `differing` descendants.
- [ ] Implement `--stale-days N`: warn about files in stage-b whose mtime is older than N days relative to stage-a.
- [ ] Unit tests: identical stages → all matching; missing files → only_in_a/only_in_b; content diff in RESULT → differing (names RESULT); content diff in COMMENTS with InputResult → matching (COMMENTS not compared); content diff in COMMENTS with InputResultComments → differing (names COMMENTS); tampered file → tampered (not differing); signature-only diff → matching (SIGNATURES excluded); root-cause descends to deepest differing.
- [ ] Acceptance: `cargo test -p einmo --lib compare` passes.

## Phase 9 — Passphrase cascade

- [ ] Implement `resolve_passphrase(cli_pass, stdin_pass, interactive_flag, env, config) -> Result<KeySource, EinmoError>` in `foolish/einmo/src/config.rs`.
- [ ] Precedence: `--passphrase <v>` > `--stdin-passphrase` (read one line from stdin) > `EINMO_PASSPHRASE` env > `einmo.toml` `[signing] passphrase` > interactive prompt on `/dev/tty` (only if no tier yielded a value).
- [ ] `--interactive` flag forces the prompt (skips tiers 1–4).
- [ ] Explicit empty string (`--passphrase ""` or `EINMO_PASSPHRASE=""`) = "set to empty" (computer key), NOT "unset". To unset, omit entirely.
- [ ] Config-file parsing: `einmo.toml` with `[signing] passphrase`, `[ci]` section, `[review]` section. Read from `.config/einmo.toml` or repo-root `einmo.toml`.
- [ ] Unit tests: CLI overrides env; env overrides config; config overrides default; empty-vs-unset distinction; --interactive forces prompt; no tier → prompt (mock /dev/tty).
- [ ] Acceptance: `cargo test -p einmo --lib passphrase` passes.

## Phase 10 — CLI (`cargo-einmo` binary)

- [ ] Create `foolish/einmo/src/bin/cargo_einmo.rs`. Use `clap` derive.
- [ ] Subcommands (each calls the library; every subcommand verifies-on-inspect any file it touches):
  - `promote <from>→<to> <work_dir> [--filter] [--passphrase|--stdin-passphrase|--interactive] [--batch]`
  - `flag <work_dir> <stage> [--filter] [--reason]`
  - `compare <stage-a> <stage-b> <work_dir> [--match-sections] [--require-comments-match] [--stale-days] [--filter] [--require-match] [--json] [--root-cause]`
  - `verify <work_dir> [--stage|--all]`
  - `confirm-signatures <path> <pubkey-prefix> [--require-all]`
  - `show <file>`
  - `console-review <work_dir> <from>→<to> [--filter] [--full] [--reexamine-rate] [--reexamine-seed] [--vim|--list] [--root-cause]` (Phase 12)
  - `serve <work_dir> [--bind]` (Phase 13)
  - `self-check [--expected <sha256>] [--quiet]` — computes SHA-256 of `env::current_exe()?`, prints path + hash; `--expected` exits non-zero on mismatch; `--quiet` prints only the hash. Also reads an expected hash from a sidecar `cargo-einmo.sha256` next to the binary if present.
- [ ] Every verb supports `--json` machine output (stable scriptable surface).
- [ ] Binary is named `cargo-einmo` so `cargo einmo …` works.
- [ ] **Advisory `# produced-by:` line**: when the test runner or a promote/flag operation writes a `.einmo`, append `# produced-by: cargo-einmo <version> sha256:<self-hash>` as an UNSIGNED advisory line outside the canonical signed content (the parser must exclude it from `canon_*`, same as `# flagged:`). Byte-steadiness is preserved — rebuilding the binary does not invalidate existing signatures.
- [ ] Acceptance: `cargo build -p einmo --bin cargo-einmo` succeeds. Manual: `cargo einmo verify --work-dir <test-suite>` runs and exits 0 on a clean suite. `cargo einmo self-check` prints the binary's SHA-256. `cargo einmo self-check --expected <wrong-hash>` exits non-zero. A generated `.einmo` contains the `# produced-by:` advisory line and still verifies (the line is excluded from canonical content).

## Phase 11 — Gates (shell glue)

- [ ] Create `scripts/einmo-pre-commit.sh`:
      ```sh
      #!/bin/sh
      cargo einmo compare output checked --work-dir . --require-match || {
        echo "einmo: output does not match checked. Promote (review) or repair."
        exit 1
      }
      ```
- [ ] Create `scripts/einmo-pre-tag.sh`:
      ```sh
      #!/bin/sh
      set -e
      cargo einmo compare checked verified --work-dir . --require-match
      cargo einmo confirm-signatures verified --pubkey-prefix "$RELEASE_KEY_PREFIX" --require-all
      ```
- [ ] Create `.github/workflows/einmo-gates.yml` (merge gate: `verify --all` + `compare checked verified --require-match` on PRs).
- [ ] Document the burden-of-correction messages in each gate's failure output.
- [ ] Acceptance: install the pre-commit hook locally; commit a divergence → blocked; promote → commit succeeds. The GH Actions YAML is valid (`actionlint` or equivalent).

## Phase 12 — `console-review` (vimdiff, diff -I, randomized re-inspection, @agent handling)

- [ ] Implement `console_review(config, from, to, opts) -> Result<ReviewReport>` in `foolish/einmo/src/review.rs` (new module; the CLI subcommand calls this).
- [ ] Demotion: for files that genuinely differ (`compare from to` → `differing`), demote from `to` back to `from` (move; `Util` entry preserved as history).
- [ ] **Randomized re-inspection**: `--reexamine-rate <pct>` (default 10 with `--full`): also pick `pct`% of files already in `to` (random sample, seeded by `--reexamine-seed`), demote them to `from`, re-present for review. Use a deterministic PRNG seeded by (work-dir-path + date) when no explicit seed. Re-promotion appends a SECOND `Util` entry (attributable re-inspection).
- [ ] Review presentation modes:
  - `--vim` (default): invoke `vimdiff <from-file> <to-file>` with `diff -I` ignoring signature lines (`^\s*\* Signed by`, `^\s*\*(Input|Result|Comments|Metadata|Entire file):`).
  - `--list`: print file paths one per line (for shell-script pipelining).
- [ ] In-file annotation handling: scan each reviewed file for `@agent`:
  - `@agent, skip` → defer (skip this round, leave in `from`).
  - `@agent` (without skip) → flag the file (`flag` to `flagged/`).
  - neither → promote (`from→to`) after the human marks acceptable.
- [ ] Final sanity grep: `@agent` in `to/` after review (report any missed).
- [ ] `--root-cause`: on a differing file, descend its subtree; report deepest differing descendants.
- [ ] Unit tests (library-level): demote moves file; reexamine-rate samples N%; reexamine-seed reproduces sample; re-promotion appends second util; @agent-skip defers; @agent flags.
- [ ] Acceptance: `cargo test -p einmo --lib review` passes. Manual: `cargo einmo console-review output→checked --work-dir <suite> --list` lists differing files.

## Phase 13 — `serve` (Proposal A: axum + SPA)

- [ ] Add deps to `einmo/Cargo.toml`: `axum`, `tokio`, `tower`, `tower-http`, `rust-embed`, `serde_json`.
- [ ] Create `foolish/einmo/src/serve.rs` + the `serve` subcommand in `cargo_einmo.rs`.
- [ ] Endpoints (REST, loopback, auth-gated):
  - `GET /api/tree` — suite overview (input tree + per-file stage badge + signature status).
  - `GET /api/diff?a=<stage>&b=<stage>&rel=<path>` — per-section diff (signature lines hidden).
  - `POST /api/promote` `{from, to, filter, passphrase?}` — promote (passphrase brokered to library, derived, discarded; never stored).
  - `POST /api/flag` `{stage, filter, reason}`.
  - `GET /api/verify` `GET /api/confirm-signatures` `GET /api/show?path=<path>`.
  - `POST /api/console-review` (review-queue state).
  - `WS /ws/alerts` — stream alerts (output≠checked, checked≠verified, flagged, staleness, signature failures).
- [ ] Serve a static SPA bundle (Vite+React or SvelteKit) via `rust-embed` at `/`.
- [ ] The server NEVER holds keys persistently; passphrase arrives via POST body, is derived to a key, used to sign, dropped.
- [ ] Acceptance: `cargo einmo serve --work-dir <suite> --bind 127.0.0.1:0` starts; `curl /api/tree` returns JSON; a promote via POST appends a util entry.

## Phase 14 — Migrate the existing `.snap` corpus to `.einmo`

- [ ] Create `foolish/einmo/src/migrate.rs`: `migrate_snap_to_einmo(snap_path) -> EinmoFile`.
- [ ] Parse the legacy flat footer (single signer) as a single `Test` entry; re-emit in the indented per-section format.
- [ ] For each crate with `snapshot_tests/`: create the Einmo work-dir structure (`input/`, `output/`, `checked/`, `verified/`). Move `.foo` inputs into `input/` (preserving any hierarchy). Migrate existing `.snap` files into `checked/` (they are the reviewed baselines) as `.einmo`. Migrate `.snap.new` files into `output/` (they are pending generated outputs).
- [ ] Run `cargo einmo verify --work-dir <suite> --all` over each migrated suite — all files must verify (the migration preserves the existing computer-key `Test` signature).
- [ ] Run `cargo einmo compare output checked --work-dir <suite>` — for suites with `.snap.new` migrated to `output/` and `.snap` to `checked/`, this shows the pending-review diffs.
- [ ] **Human re-sign pass** (wall-clock time, not coding): a human runs `cargo einmo promote checked→verified --interactive` over the migrated corpus with a human passphrase. This is human-coordinated, not an agent task.
- [ ] Acceptance: `cargo einmo verify --all` green across all three suites. Existing `cargo test -p foolish-core/foolish-ubca/foolish-ubcb --lib` still green (the `*_snapshot_tester.rs` modules now drive `EinmoSuite` and write to `output/`).

## Phase 15 — Use-case enumeration and validation (feeds user docs + design feedback)

Work through every way einmo is used, to (1) prepare user documentation and (2)
validate the design (awkward use cases reveal design gaps before code freezes).

- [ ] Enumerate the use-case matrix: {tier} × {actor} × {operation} × {granularity}. Tiers: unit / approval / integration / CI / release / regression-reinspection / performance-redacted (JVM cross-validation dropped per FOOP-03). Actors: coding agent / human reviewer / release officer / CI / auditor. Operations: evaluate / promote / flag / compare / verify / confirm-signatures / console-review / serve. Granularity: leaf (unit) / mid-tree (integration) / root (whole-program).
- [ ] For each matrix cell, write a worked example: exact `TestConfig`, CLI invocation(s), expected `.einmo` transitions, gate behavior. Ground in real modules (leaf = a `*_nyes_transitions`-shaped snapshot; mid-tree = `foolish-ubca/snapshot_tests`; root = a whole-program `.foo`).
- [ ] Exercise the hierarchical-granularity diagnostic: construct a synthetic tree where a leaf change cascades to a root mismatch; verify `compare --root-cause` identifies the deepest differing descendant.
- [ ] Exercise the gate lifecycle: commit (output==checked) → PR → merge (checked==verified) → tag (confirm-signatures <release-key> --require-all). Verify burden-of-correction messages on each failure mode.
- [ ] Exercise randomized re-inspection: seed-pinned 10% demote-and-re-review on a verified corpus; verify re-promotion appends a second util; verify a rejection (flag) surfaces baseline rot.
- [ ] Exercise per-section matching: a suite with `--require-comments-match` and one without; verify COMMENTS drift is caught/ignored per config; INPUT/RESULT always required.
- [ ] Exercise emergent human-attestation: an AI runs `promote checked→verified --passphrase ""` → confirm util pubkey == test pubkey → `confirm-signatures verified <release-key>` catches it.
- [ ] Exercise the UI flow (`serve` + SPA): list / diff / approve / flag / alert-feed / confirm-signatures over the worked examples; verify crypto boundary (UI never holds keys).
- [ ] Migrate the 289-file corpus; verify the formal matching test holds across output==checked.
- [ ] Draft the user-documentation outline from the worked examples (Getting Started / Configuring a Suite / The Four Stages / Promotion & Flagging / Gates & CI / The Granularity Diagnostic / Review & Re-inspection / UI / Troubleshooting).
- [ ] Feed back: any awkward use case becomes a design-revision item for this FOOP before code freezes.

## Phase 16 — Final verification

- [ ] `cargo fmt --all --check` passes.
- [ ] `cargo clippy --workspace -- -D warnings` passes.
- [ ] `cargo test --workspace` passes (all unit + snapshot tests green).
- [ ] `cargo build -p einmo --release` succeeds; `target/release/cargo-einmo` exists.
- [ ] `cargo einmo verify --work-dir <each-suite> --all` green (Layer 1).
- [ ] End-to-end: generate (`output/`, computer-signed) → review → `promote output→checked` → `promote checked→verified --interactive` (human) → `verify` (both entries ok) → merge gate `compare checked verified --require-match` passes.
- [ ] End-to-end gate failure: introduce an output divergence → pre-commit hook blocks the commit with the burden-of-correction message.
- [ ] End-to-end AI-bypass: `promote checked→verified --passphrase ""` → `confirm-signatures verified <release-key> --require-all` exits non-zero (util pubkey == test pubkey).
- [ ] End-to-end re-inspection: `console-review checked→verified --reexamine-rate 10 --reexamine-seed 42` → demotes 10% → re-promote appends second util.

## Phase 17 — Merge

- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-92-einmo and committed to `foop-92-einmo`.
- [ ] Merge `foop-92-einmo` to `alpha`.
  - [ ] If merge conflicts arise on `alpha`, repair all tests in /home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo.
  - [ ] STOP — ask human to check this box before continuing.
- [ ] After merge: confirm `alpha` CI green (`einmo verify --all` always-on; merge gate `compare checked verified` enforced on PRs to `alpha`).
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-92-einmo.
  - [ ] Check that this plan file has all but the Cleanup checkboxes completed.
  - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-92-einmo.
  - [ ] This is the last sub-task checkbox to be checked.

## Deferred

- [ ] **OQ (structured flag log):** flag annotation is an in-file advisory `# flagged:` line. If richer querying is later needed, consider a JSON-lines sidecar — not blocking.
- [ ] **Desktop app (Tauri vs egui):** both reuse `cargo einmo serve`'s REST API unchanged; decide when needed.
- [ ] **WASM verify in browser (Proposal C):** `einmo::verify` is already a clean submodule (Phase 6). Compile to `wasm32-unknown-unknown` when the browser-side verify is wanted; the signing server (`serve`) stays as-is.
- [ ] **Per-stage passphrase config:** `[signing.<stage>]` overrides in `einmo.toml` — deferred; the cascade + deployment convention (omit verified passphrase) suffices initially.
- [ ] **Layer 2 enablement + human re-sign migration:** once `einmo` is merged and stable, a human runs `cargo einmo promote checked→verified --interactive` over the entire `.einmo` corpus with a human passphrase. Human-coordinated, not an agent task.
- [ ] **JVM cross-validation:** dropped per FOOP-03. If JVM implementations are un-deprecated, the per-impl stage-dir model (`output-rust/`/`output-java/` with cross-`compare`) is the structure.

## Conventions for the implementing agent

- **Never** run `cargo insta accept` or set `INSTA_UPDATE=always`. (Einmo has no automated update anyway, but the old `insta` corpus may still be present during migration — do not auto-accept.)
- **Never** suppress type errors (`as any`, `@ts-ignore` — N/A in Rust, but the equivalent: no `unwrap()` on fallible crypto/format calls; use `?` and `EinmoError`).
- **Always** run `cargo fmt` and `cargo clippy -D warnings` before marking a phase complete.
- **Always** write the tamper/forgery tests BEFORE changing the signing write path (Phase 2).
- **Always** verify-on-inspect in every code path that reads a `.einmo` file — the library enforces this; do not add a "fast path" that skips verification.
- **Never** re-implement the passphrase cascade outside `einmo::config`. Shell/Python/UI call `cargo einmo` or `serve`; they never derive keys.
- When a task says "Acceptance:", that check MUST pass before the next task. If it fails, fix the failure; do not proceed.
- If a task is ambiguous, read FOOP-92.md (the specification) for the authoritative answer. This plan assumes the spec's context.
