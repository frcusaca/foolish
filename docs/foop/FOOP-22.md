---
foop: 22
title: Multi-signer snapshot signatures with appended utility signing and entire-file integrity
author: opencode 1.14.39; Qwen3.6-27B-AWQ-BF16-INT4
status: Draft
type: Standards
created: 2026-06-01
phase: meta
supersedes: []
---

# FOOP-22: Multi-signer snapshot signatures with appended utility signing and entire-file integrity

## Abstract

Extend the snapshot signature format to support multiple independent signers. When a
user signs one or more snapshot files with a passphrase via the `verify_signatures`
utility, the user's signature is **appended** to the existing signatures rather than
replacing them. Each signer entry includes an "Entire file" signature that covers the
complete file content before that signer's entry was added, providing tamper evidence
for the full file at the moment of signing.

## Motivation

**Current state:** A snapshot file has a single `SIGNATURES:` footer with one signer's
progressive triple-signature (Foolish, HS, Comments). Running `verify_signatures
--write-verified` replaces the existing footer with a new one, destroying the previous
signer's attestation.

**Problem:** If an AI agent generates a snapshot and signs it with the default (empty)
passphrase, and then a human reviews and wants to attest "I approve this output," the
human's signing operation overwrites the agent's signature. There is no chain of
attestation — only the last signer remains.

**After this FOOP:** The SIGNATURES section becomes an ordered list of signer entries.
The original test/agent signature is preserved. A human (or any utility signer) appends
their own entry below it. Each utility signer computes an "Entire file" signature over
the complete file bytes as they existed before their entry was appended, making it
impossible to modify any earlier content (including earlier signatures) without
invalidating the entire-file signature.

## Specification

### New Signature Format

The `SIGNATURES:` section changes from a flat key-value block to an indented list of
signer entries.

**Current format (single signer):**

```
SIGNATURES:
Public key: dc5f586cda43023e707697692c78032300a24b1b813cbf1ccb770080d1a7b683
Foolish signature: 12jeoTA6a9yLcnL7xrBQtpBrLZ2d4QsGbVoF9JBZAr6rp7Viww+MwrWd9xX7EKdqGr3jvl0678VpY0B9Bx1xDg==
HS signature: kLCeelGLKF5IYwX0u86CfPQroin5fBnxem+USN70SRtjjGkl8m4gWdaRyWYx9FgGSND72cYZ6WuEjQW2PWRKBg==
Comments signature: e9JTBNpuFfmVxjGOGnM26rwm6tuxRTlSVkvPceSRLwBoaXvYr6qf9zQxwG85W6lilqWegah+mOOmwkQMiI0gCQ==
```

**New format (multiple signers):**

```
SIGNATURES:
  * Signed by test: dc5f586cda43023e707697692c78032300a24b1b813cbf1ccb770080d1a7b683
    * Foolish: 12jeoTA6a9yLcnL7xrBQtpBrLZ2d4QsGbVoF9JBZAr6rp7Viww+MwrWd9xX7EKdqGr3jvl0678VpY0B9Bx1xDg==
    * HS: kLCeelGLKF5IYwX0u86CfPQroin5fBnxem+USN70SRtjjGkl8m4gWdaRyWYx9FgGSND72cYZ6WuEjQW2PWRKBg==
    * Comments: e9JTBNpuFfmVxjGOGnM26rwm6tuxRTlSVkvPceSRLwBoaXvYr6qf9zQxwG85W6lilqWegah+mOOmwkQMiI0gCQ==
  * Signed by util: a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2
    * Entire file: XyZaBcDeFgHiJkLmNoPqRsTuVwXyZaBcDeFgHiJkLmNoPqRsTuVwXyZaBcDeFgHi==
    * Foolish: aAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaAaA==
    * HS: bBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbBbB==
    * Comments: cCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcCcC==
```

### Signer Entry Structure

Each signer entry has:

1. **Label line** (2-space indent + `* Signed by <role>: <hex_pk>`):
   - `<role>` is `test` for the original test/agent signer, or `util` for utility signers.
   - `<hex_pk>` is the 64-character hex-encoded Ed25519 verifying key.

2. **Signature lines** (4-space indent + `* <label>: <base64_sig>`):
   - A `test` entry has: `Foolish`, `HS`, `Comments` (progressive triple, unchanged semantics).
   - A `util` entry has: `Entire file`, `Foolish`, `HS`, `Comments`.

### Progressive Signatures (unchanged for test entries)

The `test` signer's progressive signatures work exactly as before:
- `Foolish` = sign(`canon_input`)
- `HS` = sign(`canon_input + canon_hs`)
- `Comments` = sign(`canon_input + canon_hs + canon_comments`)

### Utility Signer Signatures

A `util` signer computes:

1. **`Entire file`**: Sign the complete file bytes as they existed **before** the
   utility signer's entry was appended. This includes the file body, the `test` entry
   (and any prior `util` entries), up to but not including the new `util` entry.
   Concretely: the file content from byte 0 up to the newline after the last existing
   signer entry (or after `SIGNATURES:` if no entries exist yet, which shouldn't happen
   in practice).

2. **`Foolish`**, **`HS`**, **`Comments`**: Same progressive triple as the `test`
   signer, computed over the same canonical content. This allows the utility signer to
   independently attest that the INPUT/RESULT/COMMENTS blocks are intact.

### Signing Workflow

When `verify_signatures --write-verified --stdin-passphrase` is invoked:

1. **Read** the file content.
2. **Parse** all existing signer entries from the SIGNATURES section.
3. **Verify** ALL existing signatures (every `test` and `util` entry). If ANY signature
   in ANY entry fails, **refuse to sign** and report the failure. This ensures the
   entire chain of attestation is valid before extending it.
4. **Capture** the current file bytes as `entire_file_content` (for the `Entire file`
   signature).
5. **Compute** canonical content (input, HS outputs, comments) and the progressive
   triple (Foolish, HS, Comments).
6. **Compute** the `Entire file` signature over `entire_file_content`.
7. **Append** a new `util` signer entry to the SIGNATURES section.
8. **Write** the updated file.

If `--write-verified` is used **without** `--stdin-passphrase` (i.e., empty passphrase),
the behavior is the same — a `util` entry is appended with the default computer/AI key.

### Multiple Utility Signers

Multiple `util` entries can accumulate on a single file:

```
SIGNATURES:
  * Signed by test: <pk1>
    * Foolish: <sig>
    * HS: <sig>
    * Comments: <sig>
  * Signed by util: <pk2>
    * Entire file: <sig over file before pk2 was added>
    * Foolish: <sig>
    * HS: <sig>
    * Comments: <sig>
  * Signed by util: <pk3>
    * Entire file: <sig over file before pk3 was added, including pk2's entry>
    * Foolish: <sig>
    * HS: <sig>
    * Comments: <sig>
```

Each `util` signer's `Entire file` signature covers everything that existed before their
entry, including all prior signer entries. This creates a chain: tampering with any
earlier entry invalidates all subsequent `Entire file` signatures.

### Verification Workflow

When `verify_signatures` is invoked **without** `--write-verified`:

1. Parse all signer entries.
2. For each `test` entry: verify the progressive triple (Foolish, HS, Comments).
3. For each `util` entry:
   - Verify the progressive triple (Foolish, HS, Comments) over canonical content.
   - Verify the `Entire file` signature over the file content up to (but not including)
     this entry.
4. Report per-entry results. The CLI output format changes to show per-signer status.

### CLI Output Format (Verification)

Current output:
```
key match: yes, foolish: yes, hs: yes, comments: yes  path/to/file.snap
```

New output (multiple signers):
```
[test] key: yes, foolish: yes, hs: yes, comments: yes  [util] entire: yes, foolish: yes, hs: yes, comments: yes  path/to/file.snap
```

If the passphrase matches a signer's key, that entry shows `key: yes`; others show
`key: no`. The `entire` column appears only for `util` entries.

### Migration of Existing Snapshots

Existing snapshot files with the flat format (no indentation, no `* Signed by` prefix)
are treated as a single `test` entry. The parser handles both formats:

- **Legacy format**: `Public key: ...` / `Foolish signature: ...` / `HS signature: ...` /
  `Comments signature: ...` → parsed as one `test` entry.
- **New format**: Indented `* Signed by ...` entries → parsed as a list of signer entries.

When a legacy-format file is signed with `--write-verified`, the output uses the new
format: the legacy entry is re-emitted as a `test` entry (with identical signatures),
and the new `util` entry is appended.

### Data Structures

**`signature.rs`** changes:

```rust
/// A single signer's entry in the SIGNATURES section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerEntry {
    /// Role: "test" (original) or "util" (appended by utility).
    pub role: String,
    /// Hex-encoded 32-byte Ed25519 verifying key.
    pub public_key_hex: String,
    /// Base64-encoded signature over canonical input (all entries).
    pub foolish_sig_b64: String,
    /// Base64-encoded progressive signature over input + HS (all entries).
    pub hs_sig_b64: String,
    /// Base64-encoded progressive signature over input + HS + comments (all entries).
    pub comments_sig_b64: String,
    /// Base64-encoded signature over entire file content before this entry was appended.
    /// Only present for "util" entries.
    pub entire_file_sig_b64: Option<String>,
}

/// Collection of signer entries for a snapshot file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotSignatures {
    pub entries: Vec<SignerEntry>,
}
```

`SnapshotSignature` (current struct) is deprecated in favor of `SnapshotSignatures`.

### Parsing

`parse_snapshot_footer` is replaced by `parse_snapshot_signatures`, which returns
`Option<SnapshotSignatures>`. It handles both legacy and new formats.

### Formatting

`SnapshotSignatures::format_footer()` renders the indented list format. A single `test`
entry with no `util` entries still renders in the new indented format (no legacy
re-emission after first utility signing).

## FIR Impact

None.

## UBC Step Impact

None.

## Test Plan

### Unit tests (`signature.rs`)

- **Parse legacy format** → single `test` entry in `SnapshotSignatures`.
- **Parse new format** (one `test` + one `util`) → two entries, correct roles.
- **Parse new format** (one `test` + two `util`) → three entries, chain intact.
- **Format roundtrip** → parse(format(entries)) == entries.
- **Legacy → new migration** → parse legacy, format new, parse again → same data.
- **Verify test entry** → progressive triple validates.
- **Verify util entry** → progressive triple + entire file signature validate.
- **Verify util entry with tampered prior content** → entire file signature fails.
- **Verify chain** (two util entries) → second util's entire file covers first util's entry.

### Integration tests (`verify_signatures` binary)

- **Sign legacy file** → legacy entry preserved as `test`, new `util` appended.
- **Sign already-signed file** → existing `util` entry preserved, new `util` appended.
- **Verify file with failed prior signature** → refused, error reported.
- **Verify chain** (test + util + util) → all entries report correct status.
- **Tamper with file body** → all `util` entire-file signatures fail, test signatures fail.
- **Tamper with a prior util entry** → subsequent util entire-file signatures fail.

### Regression

- Existing `cargo test -p foolish-core -- signature` must pass (all current tests).
- `cargo test --workspace` must pass after changes.

## Rejected Alternatives

### A. Replace signatures (current behavior)

The current `--write-verified` replaces the footer. This loses the original signer's
attestation. Rejected because it provides no chain of trust — only the last signer
remains, and there's no way to know who signed before.

### B. Append raw signature lines without restructuring

Simply appending `Public key: ...` lines below the existing footer would create ambiguity
about which key belongs to which signatures. The flat format doesn't support grouping.
Rejected because it requires a structural change anyway; might as well do it cleanly.

### C. Use a separate `.sig` file per signer

Storing signatures externally (e.g., `file.snap.signer1.sig`) decouples the signature
from the file. Rejected because it complicates the workflow — signers must manage
multiple files, and the signature file can be lost or mismatched.

### D. Cryptographic chaining (each signer signs the previous signer's signature)

Each `util` signer could sign a hash that includes the previous signer's entry. This is
more cryptographically elegant but harder to verify incrementally — you need the full
chain to verify any entry. The `Entire file` approach is simpler: each `util` entry is
independently verifiable against the file state at signing time.

## Open Questions

- Should `util` entries include a timestamp? (Currently no — signatures are
  deterministic and timestamps add complexity without clear benefit.)
- Should there be a limit on the number of `util` entries? (Proposed: no limit.)
- Should `--write-verified` without `--stdin-passphrase` (empty passphrase) still append
  as `util`, or should it be treated differently? (Proposed: always append as `util`.)

## References

- Current implementation: `foolish/foolish-core/src/signature.rs`
- CLI binary: `foolish/foolish-core/src/bin/verify_signatures.rs`
- Snapshot suite: `foolish/foolish-core/src/snapshot_suite.rs`
- Example snapshot: `foolish/foolish-core/snapshot_tests/approved/simple_addition.foo.snap.new`
- AGENTS.md approval test protocol section
