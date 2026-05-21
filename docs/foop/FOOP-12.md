---
foop: 21
title: Snapshot Canonicalization and Dual-Signing Verification
status: Final
phase: phase-5
author: Sisyphus
created: 2026-05-19
---

# FOOP-12: Snapshot Canonicalization and Dual-Signing Verification

## Problem

Snapshot files currently have inconsistent content signing:
1. The input source is signed but the signature format varies between implementations.
2. There is no canonicalization of the signed content before hashing.
3. Humanizing sequencer output is not signed at all.
4. No utility exists to verify which key signed a snapshot.

## Goals

1. Define snapshot canonicalization rules that ensure deterministic signing.
2. Sign both the input code block and the humanizing sequencer output block.
3. Move signatures to the end of snapshot files with labeled keys.
4. Provide a CLI utility to verify snapshot signatures against any passphrase-derived key.

## Canonicalization Rules

### Input Block Canonicalization
1. Read the `.foo` input file as a single string.
2. Strip leading and trailing whitespace (including final newline).
3. Append exactly one newline (`\n`).
4. This canonicalized string is placed between ` ```foolish ` and ` ``` ` fences.
5. Sign this canonicalized string.

### Output Block Canonicalization
1. Take the humanizing sequencer output for each FIR.
2. Strip leading and trailing whitespace.
3. Append exactly one newline (`\n`).
4. Place between ` ```hssnap ` and ` ``` ` fences.
5. Concatenate all canonicalized output blocks with `\n` between them.
6. Sign the concatenated string.

## New Snapshot Format

```
---
source: <test_module_path>
---
INPUT:
```foolish
<canonicalized input>
```
RESULT:
```hssnap
<canonicalized sequencer output for the FIR compute from input>
```
...
Public key: <hex-encoded verifying key>
INPUT Foolish signature: <base64 signature of canonicalized INPUT block>
RESULT HS signature: <base64 signature of canonicalized RESULT block>
```

## Verification Utility

A new CLI utility `verify_signatures` in `foolish-core` that:

### Arguments
- `--stdin-passphrase`: Read passphrase from stdin (default: empty string)
- `[directories...]`: Directories to scan (default: `snapshot_tests/approved` for both packages)

### Output
One line per `.snap` or `.snap.new` file:
```
<path>: key=<match|no_match|unsigned> foolish=<ok|fail|unsigned> hs=<ok|fail|unsigned>
```

### Behavior
1. Derive verification keypair from passphrase.
2. Scan all `.snap` and `.snap.new` files in specified directories.
3. For each file:
   - Extract public key, foolish signature, and HS signature from the bottom.
   - Extract canonicalized input and output blocks from fences.
   - Verify signatures against both the snapshot's key and the verification key.
   - Report status.

## Implementation Location

## Migration

Existing snapshots will be regenerated with the new format. Check that `SnapshotSuite` has:
1. Applied canonicalization before signing.
2. Generated both signatures.
3. Wrote the new format with signatures at the bottom.

## Verification

The `verify_signatures` utility must be able to:
- Verify all existing snapshots with the default (empty) passphrase.
- Detect snapshots signed by different keys.
- Report unsigned or tampered snapshots.

## Last Updated

**Date**: 2026-05-21
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
**Changes**: Status updated Draft → Final after merge to alpha (d6eaa782).
