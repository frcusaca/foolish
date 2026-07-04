---
foop: 22
title: FOOP-22 Implementation Plan — Multi-signer snapshot signatures
status: Draft
created: 2026-06-01
---

# FOOP-22 Implementation Plan

## Worktree

```
WORKTREE_BRANCH_NAME=multi-signer-foop-22
WORKTREE_FULL_FS_PATH=${HOME}/tmp/foolish-worktrees/multi-signer-foop-22
```

## Scope

This FOOP changes the snapshot signature format and the `verify_signatures` binary.
It does NOT change the Foolish language, FIR, or evaluation semantics.

**Files to modify:**
- `foolish/foolish-core/src/signature.rs` — core signature data structures, parsing, formatting, verification
- `foolish/foolish-core/src/bin/verify_signatures.rs` — CLI binary, signing workflow
- `foolish/foolish-core/src/snapshot_suite.rs` — may need minor update for new format

**Files NOT touched:**
- Any `.snap` or `.snap.new` files in `snapshot_tests/approved/` (migration happens at sign-time)
- Java/Scala implementations
- UBC evaluation engine

---

## Phase A: Redesign `signature.rs` data structures

Replace `SnapshotSignature` with `SignerEntry` + `SnapshotSignatures`.

- [x] Canceled. Superseded by FOOP-92 (Einmo — directory-based signed-snapshot testing with staged promotion).
      (2026-07-03 18:23)
- [-] Define `SignerEntry` struct:
      (2026-06-01 XX:XX)
  - Fields: `role: String`, `public_key_hex: String`, `foolish_sig_b64: String`,
    `hs_sig_b64: String`, `comments_sig_b64: String`, `entire_file_sig_b64: Option<String>`
  - `role` is `"test"` or `"util"`
  - `entire_file_sig_b64` is `Some(...)` only for `"util"` entries

- [-] Define `SnapshotSignatures` struct:
  - Field: `entries: Vec<SignerEntry>`
  - Implement `format_footer()` — renders the new indented list format

- [-] Deprecate `SnapshotSignature` (mark `#[deprecated]`, keep for backward compat during transition)

- [-] Update `sign_snapshot` to return `SnapshotSignatures` with a single `test` entry
  (behavior unchanged for the test signer path)

- [-] Add `sign_util_entry` function:
  - Takes: `passphrase`, `entire_file_bytes`, `canonical_input`, `canonical_hs`, `canonical_comments`
  - Returns: `SignerEntry` with role `"util"`, all four signatures

- [-] Add `verify_signer_entry` function:
  - Takes: `passphrase`, `entry`, canonical content, and for `util` entries, the file bytes up to that entry
  - Returns: per-entry verification result (key_match, foolish_ok, hs_ok, comments_ok, entire_file_ok)

- [-] Verify `cargo check -p foolish-core` compiles

## Phase B: New parsing — `parse_snapshot_signatures`

Replace `parse_snapshot_footer` with dual-format parsing.

- [-] Implement `parse_snapshot_signatures(text: &str) -> Option<SnapshotSignatures>`:
  - Detects legacy format (flat `Public key:` / `Foolish signature:` lines after `SIGNATURES:`)
  - Detects new format (indented `* Signed by <role>: <pk>` entries)
  - Legacy → single `test` entry
  - New → list of entries with correct roles

- [-] Deprecate `parse_snapshot_footer` (delegate to `parse_snapshot_signatures` internally)

- [-] Add unit tests for parsing:
  - Legacy format → single test entry
  - New format, one test entry
  - New format, one test + one util entry
  - New format, one test + two util entries
  - Malformed entries → `None` or partial parse with clear errors

- [-] Verify all existing `signature.rs` unit tests still pass

## Phase C: New formatting — indented list

Update `format_footer()` to render the new format.

- [-] Implement `SnapshotSignatures::format_footer()`:
  ```
  SIGNATURES:
    * Signed by test: <pk>
      * Foolish: <sig>
      * HS: <sig>
      * Comments: <sig>
    * Signed by util: <pk>
      * Entire file: <sig>
      * Foolish: <sig>
      * HS: <sig>
      * Comments: <sig>
  ```

- [-] Add roundtrip tests: `parse_snapshot_signatures(format_footer(sigs)) == Some(sigs)`

- [-] Verify `cargo test -p foolish-core -- signature` passes

## Phase D: Update `verify_signatures` binary

Change the signing workflow from replace-to-append.

- [-] Update imports to use `SnapshotSignatures`, `SignerEntry`, `sign_util_entry`

- [-] Modify `write_verified_file`:
  1. Read file content
  2. Parse existing signatures (legacy or new format)
  3. **Verify ALL existing entries** — if any fails, return error (refuse to sign)
  4. Capture current file bytes as `entire_file_content`
  5. Extract canonical content (input, HS, comments)
  6. Compute new `util` signer entry (entire file + progressive triple)
  7. Append new entry to existing entries
  8. Replace SIGNATURES section with reformatted footer (new indented format)
  9. Write file

- [-] Update `check_file` (read-only verification):
  - Parse all signer entries
  - Verify each entry independently
  - For `util` entries, verify `Entire file` signature against file content up to that entry
  - Collect per-entry results

- [-] Update CLI output format:
  - Single test entry: `[test] key: yes, foolish: yes, hs: yes, comments: yes  <path>`
  - Multiple entries: `[test] key: yes, ...  [util] entire: yes, key: no, ...  <path>`

- [-] Update `replace_comments_and_footer` to handle the new format (cut at last `SIGNATURES:`)

- [-] Verify `cargo build -p foolish-core --bin verify_signatures` compiles

## Phase E: Update `snapshot_suite.rs`

The snapshot suite generates new snapshots — ensure it uses the new format.

- [-] Update `evaluate()` to use `sign_snapshot` (which now returns `SnapshotSignatures`)
  - The generated snapshot should have a single `test` entry in the new indented format
  - This is a breaking change for `.snap.new` generation but NOT for existing approved files

- [-] Verify `cargo test -p foolish-core --lib` compiles

## Phase F: Comprehensive testing

- [-] Run `cargo test -p foolish-core -- signature` — all unit tests pass
- [-] Run `cargo test -p foolish-core --lib` — all lib tests pass
- [-] Manual test: sign a legacy-format snapshot file
  - Verify legacy entry preserved as `test`
  - Verify new `util` entry appended with `Entire file` signature
- [-] Manual test: sign an already-signed (new format) file
  - Verify existing entries preserved
  - Verify new `util` entry appended
- [-] Manual test: verify a file with tampered content
  - Verify `util` entire-file signatures fail
- [-] Manual test: verify a file with tampered prior signer entry
  - Verify subsequent `util` entire-file signatures fail
- [-] Run `cargo test --workspace` — full workspace passes
- [-] Run `cargo clippy -p foolish-core` — no new warnings

## Phase G: Documentation updates

- [-] Update `AGENTS.md` signature verification section:
  - Document new multi-signer format
  - Document signing workflow (append, not replace)
  - Document `Entire file` signature semantics
  - Update CLI output format examples
- [-] Update `foolish-core/src/signature.rs` module-level documentation
- [-] Update both files' "Last Updated" sections

## Phase H: Cleanup and merge

- [-] Verify all work is complete in `${WORKTREE_FULL_FS_PATH}` and committed to `foop/multi-signer-foop-22`
- [-] Merge `foop/multi-signer-foop-22` to alpha
  - [-] If merge conflicts arise:
    - [-] Repair conflicts
    - [-] Re-run `cargo test --workspace`
    - [-] Re-commit
- [-] Cleanup `${WORKTREE_FULL_FS_PATH}`
  - [-] Check that this plan has all but Cleanup checkboxes completed
  - [-] Remove "${WORKTREE_FULL_FS_PATH}"
  - [-] This is the last checkbox to be checked in this plan

---

## Last Updated

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Plan canceled: added [x] Canceled marker (superseded by FOOP-92) and marked all outstanding checkboxes [-]; already-completed checkboxes left as historical record.
