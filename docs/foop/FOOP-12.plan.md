---
foop: 12
title: Snapshot Canonicalization and Dual-Signing Verification — Implementation Plan
author: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
status: Draft
created: 2026-05-20
---

# FOOP-12: Implementation Plan

## Worktree

Use the existing worktree created for the UBC humanizing sequence round-1
work. No new worktree needed.

```
STARTING_PATH=/home/hcbusy/foolish-rust
STARTING_BRANCH=alpha
WORKTREE_BRANCH_NAME=foop/unified_humannizing_sequencing_foop-12
WORKTREE_FULL_FS_PATH=${HOME}/tmp/foolish-worktrees/unified_humannizing_sequencing_foop-12
```

## Baseline: What Already Exists

Before checking items, understand the current state:

- `foolish-core/src/signature.rs` — Ed25519 infrastructure complete:
  `derive_keypair`, `sign_content`, `verify_signature`, `sign_input_line`
  (old format: `SIG: <hex> <b64>` before INPUT), `verify_input_line`.
- `foolish-core/src/snapshot_suite.rs` — calls `sign_input_line` and places the
  `SIG:` line **before** `INPUT:`. RESULT blocks are bare (no fences).
- 134 approved snapshots in `foolish-core/snapshot_tests/approved/` in the
  OLD format (SIG at top, no `hssnap` fences, no HS signature).
- No `foolish-ubcb-cli` snapshot files yet (directory does not exist).
- Crypto crates (`ed25519-dalek 2.2.0`, `argon2 0.5.3`) already in
  `foolish-core/Cargo.toml`.

FOOP-12 requires:
1. Canonicalization rules applied before signing.
2. **Dual signing**: canonicalized input block AND canonicalized HS output block.
3. Signatures **moved to the bottom** of each snapshot with three labeled lines.
4. RESULT blocks wrapped in ` ```hssnap ` fences.
5. A `verify_signatures` utility to check any passphrase against all snapshots.

## Step 1: Canonicalization helpers in `signature.rs`

- [ ] Add `pub fn canonicalize_input(source: &str) -> String`:
      strip leading and trailing whitespace, append exactly one `\n`.
- [ ] Add `pub fn canonicalize_output_block(output: &str) -> String`:
      strip leading and trailing whitespace, append exactly one `\n`.
- [ ] Add `pub fn canonicalize_all_outputs(outputs: &[String]) -> String`:
      call `canonicalize_output_block` on each, join with `\n`.
- [ ] Unit tests for `canonicalize_input`:
  - empty string → `"\n"`
  - already-trimmed single line → unchanged + `\n`
  - leading/trailing whitespace and newlines removed
- [ ] Unit tests for `canonicalize_output_block`: mirrors above
- [ ] Unit tests for `canonicalize_all_outputs`:
  - single block round-trip
  - multiple blocks joined correctly
- [ ] `cargo test -p foolish-core -- signature` passes

## Step 2: Dual-signing API in `signature.rs`

- [ ] Add `pub fn sign_snapshot(passphrase: &str, input: &str, hs_outputs: &[String]) -> SnapshotSignature`:
  - Canonicalize `input` → `canon_input`
  - Canonicalize all `hs_outputs` → `canon_hs`
  - `derive_keypair(passphrase)` → `(sk, vk)`
  - `sign_content(&sk, &canon_input)` → `foolish_sig`
  - `sign_content(&sk, &canon_hs)` → `hs_sig`
  - Return `SnapshotSignature { public_key_hex, foolish_sig_b64, hs_sig_b64 }`
- [ ] Add `pub struct SnapshotSignature { public_key_hex: String, foolish_sig_b64: String, hs_sig_b64: String }` with a `format_footer() -> String` method that renders:
      ```
      Public key: <hex>
      Foolish signature: <b64>
      HS signature: <b64>
      ```
- [ ] Add `pub fn verify_snapshot(passphrase: &str, input: &str, hs_outputs: &[String], sig: &SnapshotSignature) -> SnapshotVerification`:
  - Returns `SnapshotVerification { key_match: bool, foolish_ok: bool, hs_ok: bool }`
- [ ] Add `pub fn parse_snapshot_footer(snapshot_text: &str) -> Option<SnapshotSignature>`:
      extract the three footer lines from the bottom of a snapshot file.
- [ ] Unit tests:
  - `sign_snapshot` + `verify_snapshot` round-trip (empty passphrase)
  - `verify_snapshot` fails when input tampered
  - `verify_snapshot` fails when hs_outputs tampered
  - `verify_snapshot` fails with wrong passphrase
  - `parse_snapshot_footer` parses well-formed footer
  - `parse_snapshot_footer` returns `None` for missing footer
- [ ] Mark `sign_input_line` and `verify_input_line` as `#[deprecated]`
      with doc comment pointing to `sign_snapshot`.
- [ ] `cargo test -p foolish-core -- signature` passes

## Step 3: New snapshot format in `snapshot_suite.rs`

- [ ] Update `SnapshotSuite::evaluate()` (`foolish-core/src/snapshot_suite.rs:136`):
  1. Collect HS-formatted output for each FIR:
     ```rust
     let hs_output = crate::Sequencer::format(&fir);
     ```
  2. Build `hs_outputs: Vec<String>` from all FIRs.
  3. Call `crate::signature::sign_snapshot("", &source, &hs_outputs)` →
     `SnapshotSignature`.
  4. Assemble snapshot body in this order:
     - `INPUT:` line
     - ` ```foolish ` fence
     - `source.trim_end()` (canonicalized input content displayed verbatim)
     - ` ``` ` fence
     - For each `(i, hs_output)`:
       - `[{i}] RESULT:` line
       - ` ```hssnap ` fence
       - `hs_output.trim_end()`
       - ` ``` ` fence
     - `sig.format_footer()`
  5. Remove old `sign_input_line` call and `SIG:` line prepend.
- [ ] `cargo check -p foolish-core` passes (no compile errors)
- [ ] `cargo test -p foolish-core -- snapshot_suite` passes (logic-only tests, if any)

## Step 4: Regenerate all 134 snapshots

The existing `.snap` files have the old format. Run with `INSTA_UPDATE=always`
to regenerate them in the new format. (@human: `.snap` files are insta
snapshots, not `.approved.foo` files — they are safe to regenerate this way.)

- [ ] Run: `INSTA_UPDATE=always cargo test -p foolish-core --lib 2>&1 | tail -20`
      Expect all tests to produce `.snap.new` files.
- [ ] Run: `cargo insta accept` to accept all new snapshots.
- [ ] Spot-check 3 snapshots manually:
  - One with a single-statement input
  - One with multi-statement input
  - One with an alarm
  - Confirm each has: no `SIG:` at top; `INPUT:` + ` ```foolish ` fence;
    RESULT blocks in ` ```hssnap ` fences; three footer lines at bottom.
- [ ] Run: `cargo test -p foolish-core --lib` — all tests pass (no `.snap.new` left).

## Step 5: `verify_signatures` utility

Per FOOP-12, implement in `foolish-core` (not a new binary crate). A `verify_signatures`
binary in `foolish-core/src/bin/verify_signatures.rs` is the simplest path.

(@human: The companion spec `UBC_humanizing_sequence_round_1.spec.md` describes a separate
`foolish-sig` binary crate with `list`, `verify`, and `sign` subcommands. FOOP-12 is more
conservative — just `verify_signatures`. A future FOOP can promote to the full `foolish-sig`
crate if needed.)

- [ ] Create `foolish-core/src/bin/verify_signatures.rs`:
  - CLI args: `--stdin-passphrase` flag; zero or more positional directory paths
    (default: `snapshot_tests/approved` relative to crate root).
  - Scan all `.snap` and `.snap.new` files in each directory.
  - For each file:
    - Call `parse_snapshot_footer` to extract `SnapshotSignature`.
    - If none → report `unsigned`.
    - Otherwise derive keypair from passphrase (or empty default) and call
      `verify_snapshot`. Extract `input` and `hs_outputs` by parsing the
      `INPUT:` block and `RESULT:` blocks from the snapshot.
    - Print one line per file:
      `<path>: key=<match|no_match|unsigned> foolish=<ok|fail|unsigned> hs=<ok|fail|unsigned>`
- [ ] Add `[[bin]]` entry to `foolish-core/Cargo.toml`.
- [ ] Manual smoke test against `snapshot_tests/approved/` with empty passphrase:
      all 134 files should report `key=match foolish=ok hs=ok`.
- [ ] `cargo build -p foolish-core --bin verify_signatures` succeeds.

## Step 6: Final verification and merge

- [ ] `cargo test --workspace` — all tests pass.
- [ ] `cargo check --workspace` — no warnings in changed files.
- [ ] Verify no `.snap.new` files remain: `find . -name "*.snap.new" | wc -l` → 0.
- [ ] Commit all changes to `foop/unified_humannizing_sequencing_foop-12` with message:
      ```
      FOOP-12: dual-sign snapshots, move sigs to footer, hssnap fences

      - Add canonicalize_input/output helpers in signature.rs
      - Add sign_snapshot / verify_snapshot / parse_snapshot_footer
      - Update SnapshotSuite::evaluate to write new format
      - Regenerate all 134 core snapshots
      - Add verify_signatures binary

      Claude Code 2.1.119 (Claude Code); Sonnet 4.6
      ```
- [x] STOP. STOP!! STOP!!! ASK HUMAN to review snapshot diff and approve before continuing.
      (2026-05-21 — human approved: "tests are in a state ready to merge")
- [x] Merge `foop/unified_humannizing_sequencing_foop-12` to `alpha`.
      (2026-05-21 — merge commit d6eaa782)
- [x] Update FOOP-12.md status: `Draft` → `Final`.
      (2026-05-21)

## References

- `foolish-core/src/signature.rs` — existing crypto infrastructure
- `foolish-core/src/snapshot_suite.rs:136` — `evaluate()` method to update
- `foolish-core/snapshot_tests/approved/` — 134 snapshots to regenerate
- `docs/foop/FOOP-12.md` — specification
- `docs/foop/UBC_humanizing_sequence_round_1.spec.md` §5 — companion signing spec

## Last Updated

**Date**: 2026-05-20
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
**Changes**: Initial plan created from FOOP-12 spec and codebase analysis.
