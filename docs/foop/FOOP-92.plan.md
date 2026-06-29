# FOOP-92 Plan — Signed-snapshot testing library with separated review and promotion lifecycle

This plan executes [FOOP-92](FOOP-92.md). Read the specification first — the
plan assumes its context. The design is frozen (all blocking OQs resolved;
OQ-5 deferred and non-blocking).

Branch and worktree lifecycle (per `foop.md`):

```
WORKTREE_ORIGIN_BRANCH=alpha
WORKTREE_ORIGIN_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo
WORKTREE_BRANCH_NAME=foop-92-signed-snap
WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-92-signed-snap
```

- [ ] begun
      ( ) <!-- timestamp when work commences -->

- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-92-signed-snap with branch `foop-92-signed-snap` off `alpha`
      ```bash
      cd /home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo
      git worktree add -b foop-92-signed-snap /home/hcbusy/tmp/foolish-worktrees/foop-92-signed-snap
      ```

## Phase 1 — Crate scaffolding

- [ ] Create `foolish/foolish-snap/Cargo.toml` (workspace member; dev-deps: insta, rayon, num_cpus, argon2, ed25519-dalek, base64, hex, clap; features: `parallel-snapshot` gating rayon)
- [ ] Create `foolish/foolish-snap/src/lib.rs` with module declarations (`mod lifecycle; mod signature; mod snapshot_suite; mod format;`) and public re-exports (`SignedSnapshot`, `SnapshotLifecycle`, `SnapshotLifecycleOps`, `SignerEntry`, `SnapshotSuite`, `Evaluator`, `SnapError`)
- [ ] Add `foolish-snap` to the workspace `members` in `foolish/Cargo.toml`
- [ ] Add `foolish-snap = { path = "../foolish-snap" }` as a dev-dependency in `foolish-core/Cargo.toml` and `foolish-ubcb/Cargo.toml`
- [ ] `cargo check --workspace` passes (empty crate compiles; no code moved yet)

## Phase 2 — Move existing code into `foolish-snap`

- [ ] Move `foolish-core/src/signature.rs` → `foolish-snap/src/signature.rs`; update `foolish-core/src/lib.rs` to re-export from `foolish_snap` (preserve public API: `derive_keypair`, `sign_snapshot`, `verify_snapshot`, `parse_snapshot_footer`, `SnapshotSignature`, `SnapshotVerification`, canonicalization fns)
- [ ] Move `foolish-core/src/snapshot_suite.rs` → `foolish-snap/src/snapshot_suite.rs`; **generalise the `Evaluator` trait** to return `Vec<String>` (formatted output blocks) instead of `Vec<FirRef>`. Move the FIR→String formatting (`FirSequencer::format`) into the Foolish adapters (`UbcEvaluator`, `UbcbEvaluator`) so `foolish-snap` has no `FirRef` dependency.
- [ ] Update `foolish-core/src/ubc_snapshot_tester.rs` `UbcEvaluator` to format FIRs internally and return `Vec<String>`
- [ ] Update `foolish-ubcb/src/ubcb_snapshot_tester.rs` `UbcbEvaluator` to format FIRs internally and return `Vec<String>`
- [ ] Move `foolish-core/src/bin/verify_signatures.rs` → `foolish-snap/src/bin/verify_signatures.rs`; update `foolish-core/Cargo.toml` to remove the bin target, add it to `foolish-snap/Cargo.toml`
- [ ] `foolish-core` re-exports `SnapshotSuite`, `Evaluator`, `signature::*` from `foolish_snap` so existing test modules compile unchanged
- [ ] `cargo test -p foolish-core --lib` passes (all existing signature + snapshot tests green)
- [ ] `cargo test -p foolish-ubcb --lib` passes

## Phase 3 — Adopt FOOP-22 append-multi-signer format

- [ ] Add `SignerEntry` struct (`role`, `public_key_hex`, `foolish_sig_b64`, `hs_sig_b64`, `comments_sig_b64`, `entire_file_sig_b64: Option<String>`) and `SnapshotSignatures { entries: Vec<SignerEntry> }` to `foolish-snap/src/signature.rs`
- [ ] Implement `parse_snapshot_signatures` (replaces `parse_snapshot_footer`): handles both legacy flat footer (→ single `test` entry) and new indented `* Signed by …` format
- [ ] Implement `SnapshotSignatures::format_footer()` rendering the indented list format
- [ ] **Switch `verify_signatures --write-verified` from replace to append** — a new `util` entry is appended; existing `test`/`util` entries preserved. This is the core behaviour change.
- [ ] `--write-verified` refuses to append if ANY existing signature fails (chain integrity) — verify all entries first
- [ ] Add `# promoted: <ISO8601>` unsigned comment line above `util` entries (OQ-4 resolution; outside signed content, preserves determinism)
- [ ] Unit tests (from FOOP-92 Test Plan): parse legacy, parse new, parse multi-util, format roundtrip, legacy→new migration, verify test entry, verify util entry + entire-file, tamper detection, chain integrity
- [ ] Migrate existing `.snap` corpus: run `verify_signatures --write-verified` over all approved dirs with the computer key (empty passphrase) to re-emit in the new indented format. (No human signature added — these stay computer-only until human review, per the two-layer gate default-OFF.)
- [ ] All existing `.snap` files verify under the new parser
- [ ] `cargo test -p foolish-snap --lib` passes

## Phase 4 — Lifecycle state machine (`lifecycle.rs`)

- [ ] Create `foolish-snap/src/lifecycle.rs` with `SnapshotLifecycle` enum (`Generated`, `Reviewed`, `Flagged`, `Promoted`)
- [ ] Implement `SignedSnapshot::lifecycle(path)` deriving state from file extension + signer list
- [ ] Implement `SignedSnapshot::is_promoted()` (true iff `util` entry present)
- [ ] Implement `SignedSnapshot::computer_signature_ok()` (test entry verifies under empty passphrase)
- [ ] Implement `SnapshotLifecycleOps` trait: `review()` (Generated→Reviewed, pure rename `.snap.new`→`.snap.new.approved`, no signature change), `flag(comment)` (Generated→Flagged, append to `.check` log, remove `.snap.new`, no signature change), `promote(passphrase)` (Reviewed→Promoted, append `util` entry with Entire-file + progressive triple, rename to `.snap`)
- [ ] Enforce invariants: `promote` refuses without `test` entry (always-signed invariant); `promote` refuses if any existing signature fails; `promote` requires `--stdin-passphrase` (not satisfied by env/config tier alone per §9)
- [ ] Unit tests (from FOOP-92 Test Plan): lifecycle derivation, review transition (test entry byte-identical), flag transition, promote transition, promote-refuses-without-test, promote-refuses-on-broken-chain, second promote (re-approval), tamper-after-promotion, legacy parse, is_promoted()
- [ ] `cargo test -p foolish-snap --lib` passes

## Phase 5 — Passphrase resolution cascade (§9)

- [ ] Implement `resolve_passphrase()` with precedence: CLI `--passphrase`/`--stdin-passphrase` > env `FOOLISH_SNAP_PASSPHRASE` > config `.config/foolish-snap.toml` `[signing] passphrase` > default `""`
- [ ] Explicit empty string (`--passphrase ""`) treated as "set to empty" (computer key), not "unset"
- [ ] `promote()` path requires `--stdin-passphrase` specifically — not satisfied by env/config alone (interactive human act)
- [ ] Add config-file parsing (`foolish-snap.toml`: `[signing] passphrase`, `[ci] require_human_promotion`, `[review] staleness_days`)
- [ ] Unit tests: each tier wins over lower; CLI overrides env; env overrides config; config overrides default; empty-vs-unset distinction; promote refuses env/config-only passphrase

## Phase 6 — Two-layer CI gate (§10)

- [ ] **Layer 1 (always-on):** `verify_signatures <dir>` verifies all `.snap`/`.snap.new` — all signer entries valid + content matches. Exit non-zero on any failure. Runs on all branches.
- [ ] **Layer 2 (opt-in):** `verify_signatures --require-human-promotion <dir>` additionally fails if any committed `.snap` lacks a `util` (human) entry. Enabled via `--require-human-promotion` flag, `FOOLISH_SNAP_REQUIRE_HUMAN_PROMOTION=1` env, or `[ci] require_human_promotion = true` config. Default OFF.
- [ ] Layer 2 is a merge-to-main and tag gate, NOT a commit/push gate — documented in `--help` and AGENTS.md
- [ ] Add `SignedSnapshot::is_promoted()` wiring (already from Phase 4) so the CLI check is a one-liner per file
- [ ] Unit tests: Layer 1 fails on tampered/broken/mismatched; Layer 2 fails on computer-only `.snap` when enabled; Layer 2 passes when all `.snap` have `util`; Layer 2 off by default (no enforcement)
- [ ] Add a Rust CI job to `.github/workflows/tests.yml`: `cargo insta test --check` + `cargo insta pending-snapshots` + `verify_signatures <approved-dirs>` (Layer 1, always-on). Layer 2 NOT enabled by default (corpus is computer-only until human review migration).

## Phase 7 — Inline-expected-value refusal (OQ-6)

- [ ] Add a test (in `foolish-snap` or a workspace-level test) that greps test modules under `SnapshotSuite` for `assert_snapshot!(…, @"…")` inline expected values and fails CI if found
- [ ] Error message: "inline expected values bypass the signing lifecycle; use a file snapshot or `SnapshotSuite::evaluate_inline` for inlined inputs"
- [ ] Document in AGENTS.md: expected results must be signed `.snap` files; inputs may be inlined via `evaluate_inline(name, input, evaluator)`
- [ ] Confirm no existing test module uses inline `@"…"` expected values (the Foolish project uses file snapshots only — should be clean)

## Phase 8 — Generalise `SnapshotSuite` API

- [ ] Add `SnapshotSuite::evaluate_inline(name, input, evaluator)` entry point — input is a string, not a file; still produces a signed `.snap` file (input captured into `INPUT:` block, signed)
- [ ] Keep `SnapshotSuite::evaluate(path, evaluator)` for file-based inputs (Foolish's `.foo`-file model)
- [ ] `SnapshotSuite::evaluate_all(threads, evaluator)` continues to discover `.foo` files; add `evaluate_all_inline(pairs: &[(name, input)], threads, evaluator)` for the inlined-input case
- [ ] Unit tests: `evaluate_inline` produces a valid signed `.snap.new` with the inlined input in the `INPUT:` block; signature verifies
- [ ] Update the `Evaluator` trait doc: "returns formatted output blocks; the library is project-agnostic (no FirRef dependency)"

## Phase 9 — Wrap shell scripts over library calls

- [ ] Update `foolish_review.sh` to call `SignedSnapshot::review()` / `flag()` via `cargo run -p foolish-snap --bin verify_signatures --` (or a new `review` subcommand) instead of raw `mv`/`rm`. Preserve: vimdiff, `@agent` flagging → append-to-`.check`-log, `@agent, skip` defer, 10% reread sampling, `diff -I` ignoring signature lines.
- [ ] Update `accept_approved.sh` to call `SignedSnapshot::promote()` via the library. Replace the hardcoded `target/debug/verify_signatures` path with `cargo run -p foolish-snap --bin verify_signatures --`. Preserve single-passphrase-prompt-for-the-batch behaviour.
- [ ] Merge the updated `foolish_review.sh` and `accept_approved.sh` from the `foop-62-ubca-mimo` worktree into the main branch (they are currently worktree-local).
- [ ] Update AGENTS.md: correct `hssnap`→`hfssnap`, "HS signature"→"HFS signature", add `Comments signature` line, fix `bind_dynamic`→`bind_to_scope`/`bind`/`dynamic_redaction`, reconcile `.snap.new.check` description (append-log, not rename), document the two-layer CI gate and the passphrase cascade.

## Phase 10 — Hardening (Appendix E P0–P3)

- [ ] Add `.snap.new`, `.snap.new.approved`, `.snap.new.check` to `.gitignore` (only `.snap` tracked)
- [ ] Remove `INSTA_UPDATE=always cargo test *` from `foolish/.claude/settings.json` (directly contradicts AGENTS.md hard rule)
- [ ] Add `.gitattributes`: `*.snap linguist-language=YAML`, `*.snap.new linguist-generated=true`, consider `*.snap merge=ours`
- [ ] Add `.config/insta.yaml` with `behavior.require_full_match: true` (pins header metadata) and `test.unreferenced: reject` equivalent
- [ ] Remove dead insta dev-dep from `foolish-parser/Cargo.toml`
- [ ] Add `[profile.dev.package.insta] opt-level = 3` and `[profile.dev.package.similar] opt-level = 3` to workspace `Cargo.toml`
- [ ] Migrate test harness from `Settings::clone_current()` + `set_*` + `bind` to idiomatic `with_settings!` macro
- [ ] Correct README.md: `cargo test -p foolish-ubcb-cli --lib` → `cargo test -p foolish-ubcb --lib`

## Phase 11 — Final verification

- [ ] `cargo test --workspace` passes (all unit + snapshot tests green)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --all --check` passes
- [ ] `verify_signatures` over all approved dirs: all `.snap` files verify (Layer 1 green)
- [ ] `foolish_review.sh` / `accept_approved.sh` produce identical observable behaviour to the worktree versions (diff-ignoring signatures, single passphrase prompt)
- [ ] End-to-end manual test: generate (`.snap.new`, computer-signed) → review (`.snap.new.approved`) → promote (`.snap`, human-signed) → verify (both entries ok) → Layer 2 gate blocks merge if `require_human_promotion` enabled
- [ ] End-to-end: commit computer-only `.snap` to feature branch → push succeeds → Layer 2 (if enabled on target) blocks PR merge to `main`

## Phase 12 — Merge

- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-92-signed-snap and committed to `foop-92-signed-snap`
- [ ] Merge `foop-92-signed-snap` to `alpha`
  - [ ] If merge conflicts arise on `alpha`, repair all tests in /home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo
  - [ ] STOP — ask human to check this box before continuing
- [ ] After merge: confirm `alpha` CI green (Layer 1 always-on; Layer 2 still OFF — corpus is computer-only until a human re-signs it)
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-92-signed-snap
  - [ ] Check that this plan file has all but the Cleanup checkboxes completed
  - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-92-signed-snap
  - [ ] This is the last sub-task checkbox to be checked

## Deferred — OQ-5 (structured `.check` log)

- [ ] TBD — whether `flag()` writes JSON-lines `{timestamp, snapshot_path, agent_comment, flagged_content_digest}` instead of free-text `$(date) cat $x` + content. To be flushed out in a follow-up; not blocking this FOOP.

## Deferred — Layer 2 enablement + human re-signing migration

- [ ] TBD (separate coordination) — once `foolish-snap` is merged and stable, a human runs `verify_signatures --write-verified --stdin-passphrase` over the entire `.snap` corpus with a human passphrase, then enables `require_human_promotion = true` in config. This is a human-coordinated migration, not an agent task.
