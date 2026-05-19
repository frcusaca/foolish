---
foop: 101
title: UBC Humanizing Sequencer Round 1
status: Draft
phase: phase-5
---

# UBC Humanizing Sequencer Round 1

## Overview

This spec addresses five interconnected problems in the UBC snapshot testing pipeline: (1) multi-line input truncation, (2) humanizing sequencer formatting overhaul with grouping/alignment distinction, (3) WOConstanic state bug, (4) forward reference evaluation bug, and (5) input file signature verification.

## Problem Statements

### Problem 1: Multi-line Input Truncation

**Current behavior**: `alarm_division_by_zero_deeply_nested.foo` is a 10-line multi-line input. The approved snapshot shows:
"""
INPUT: {l1 = {l2 = {l3 = {bad = 1 / 0; good = 42};};};}
"""
This is a single-line collapsed representation. When the `.snap.new` was generated, it showed:
"""
INPUT: {
[0] PARSED:
"""
The INPUT line is truncated at the first newline — only `{` appears, then the PARSED section starts on the next line. This is because `Sequencer::format_with_header` uses `source.trim()` which collapses multi-line input into a single line that breaks at newlines.

**Root cause**: `Sequencer::format_with_header` at line 22-24:
"""rust
pub fn format_with_header(source: &str, fir: &Fir, steps: u64) -> String {
    let body = Self::format(fir);
    format!("INPUT: {}\nPARSED:\n{}\nSTEPS: {}", source.trim(), body, steps)
}
"""
The `{}` placeholder in the format string cannot handle embedded newlines in `source`.

**Target behavior**: Multi-line input must be preserved ORIGINAL CODE. Use foolish code fence to separate from rest of the file
"""
INPUT:
```foolish
{
  l1={
    l2={
      l3={
        bad=1/0;
        good=42
      }
    }
  }
}
```
[0] PARSED:
...
"""

### Problem 2: Humanizing Sequencer Formatting Overhaul

**Current format** (from `alarm_division_by_zero_in_brane.foo.snap`):
"""
Brane [EMBRYONIC]
  a = 
  Operator(/) [EMBRYONIC]
    Int(10) [INDEPENDENT]
    Int(2) [INDEPENDENT]
  b = 
  Operator(/) [EMBRYONIC]
    Int(10) [INDEPENDENT]
    Int(0) [INDEPENDENT]
"""

**Problems with current format**:
- No distinction between grouping indent (structural hierarchy) and alignment indent (visual continuation)
- State `[EMBRYONIC]` is on the same line as the FIR name, making it hard to read
- Operators don't visually group their operands — they look like separate list items
- Branes use `Brane [STATE]` which doesn't visually convey containment

**New formatting rules**:

#### 2.1 Grouping Indent vs Alignment Indent

Define two indent constants:
- `GROUPING_INDENT = "  "` (2 spaces) — added when entering a new structural level (after `{`, `(`, `[`)
- `ALIGN_INDENT = " "` Alignment indentation, counts the number of characters before the last grouping symbol(from above) this is the number of space to add to lines within the grouping

**Key principle**: When a line ends with a grouping open symbol (`{`, `(`, `[`), the next line starts with an additional `GROUPING_INDENT`, and optionally additional adaptive allignment indent. When the closing symbol appears, the indent returns to the previous level.

#### 2.2 Atomic Values

Atomic FIRs (Int, NK/???) are formatted inline:
"""
Int(10)
"""
because Independent and Constant values do not need their Nyes printed for this purpose.

#### 2.3 Non-Atomic FIRs

Non-atomic FIRs (Operator, Search, Index, HeadTail, StayFoolish, StayFullyFoolish, Concatenation) are formatted as function calls with their content in parentheses:
"""
Operator(/, [EMBRYONIC]
  Int(10),
  Int(2)
)
"""
Note: The state appears on the first line after the operator name, inside the parentheses. Child operands are indented only by GROUPING_INDENT in this case. The closing parenthesis aligns with the opening operator name.

**Format template**:
"""
Name(args, [NYSE STATE if not CONSTANT or INDEPENDENT]
    child1
    child2
)
"""

#### 2.4 Brane Format

Branes use braces, not parentheses:
"""
Brane{
  b = Brane{
             Int(10)
             Int(20)
             Int(30)
  }
}
"""
This example, statement making assignment to brane b is only indented using GROUPING_INDENT.
The body of b's brane is indented several ways:
"""
Brane{
  b = Brane{
             Int(10)
             Int(20)
             Int(30)
^^ Two spaces here for root brane's GROUPING_INDENT
  ^^^^^^^^^ 9 characters were typed before the final grouping '{' symbol, so the body starts 9 more characters inward.
           ^^ Two more spaces for b's own GROUPING_INDENT
             {
               yes_this_can_get_deep_fast = {
                                              we_have_wide_console_windows=1
               }
             }
       }
}
"""
**Format template**:
"""
Brane{[show NYES that are not CONSTANT or INDENPENDENT]
  name = body
  name2 = body2
}
"""

The closing brace aligns with start of its `Brane{`.

#### 2.5 State Placement

State `[STATE]` placement:
- Atomic FIRs: `Int(10)` — None for atomic FIR
- Non-atomic FIRs: `Operator(/, [EMBRYONIC]` — state on first line, after the opening ONLY if this FIR is pre-constant.
- Branes: `Brane{[EMBRYONIC]` — state on first line, after the opening brace if brane is pre-constant.

#### 2.6 Statement Formatting in Branes

Statements in branes:
- Named atomic: `x = Int(10)` — single line
- Named non-atomic:
  """
  sum = Operator( +,
                  Int(10)
                  Int(20)
  )
  """
- Anonymous atomic: `Int(10)` — single line
- Anonymous non-atomic: Same as named non-atomic but without `name = `

### Problem 3: WOConstanic State Bug
Input:
```foolish
{x=10; y=20; z=30; sum = x + y + z; avg = sum / 3;}
```

**Current behavior** (`complex_brane_with_operations_and_search.foo`):
"""
{
  sum = Operator(+, [WOCONSTANIC]
    Search(pattern='^sum$', dir=BACKWARD, FREE) [WOCONSTANIC]
      Operator(+) [WOCONSTANIC]
        Int(30) [CONSTANT]
        Search(pattern='^z$', dir=BACKWARD, FREE) [CONSTANT]
    Int(3) [INDEPENDENT]
  )
}
"""

**Expected behavior**: `sum` should be `CONSTANT`, not `WOCONSTANIC`. The `avg` search for `sum` should resolve to a `CONSTANT` value (the constanic clone should reference the constant, not remain WOConstanic).

**Root cause**: When UBC performs a "constanic clone" for a search result, it should clone/reference the CONSTANT value found by the search. Currently, the clone retains the WOConstanic state instead of propagating the CONSTANT state from the resolved target. Diagnose the problem, it is possible user misunderstand implementation. But in UBC (not UBCb), is it not the case that the search for `sum` in `avg = sum / 3` does not happen until `sum` is already coordinated in the brane and fully evaluated to constant, as matter of evaluation order? let me know if it is not the case.

A second problem is that the `+` operator should have be Woconstanic because it has no constanic children (ever, no matter how this is run. the searches for x,y,z is never constanic) These TWO problems should both be investigated and discuseed before repair.

**Location**: `foolish-core/src/ubc.rs` — the constanic clone logic (likely in `step_one` for `SearchFir`).

### Problem 4: Forward Reference Evaluation Bug

**Current behavior** (`complex_forward_refs_in_nested_branes.foo`):
"""
{nested = {inner = {val = x}}; x = 42;}
"""
Result shows `val = Int(42) [INDEPENDENT]` with the brane as `CONSTANT`.

**Expected behavior**: `nested` is defined BEFORE `x` (lower line number), which means `val = x` is evaluated before `x = 42` exists. The search for `x` should find it as a forward reference, making `val` ECONSTANIC (not CONSTANT). The current evaluation incorrectly resolves `x` to 42 during the forward reference phase.

**Root cause**: UBC evaluation order. The search for `x` in `val = x` should recognize that `x` is defined later (higher line number) and mark the search as ECONSTANIC. Currently, UBC resolves forward references too aggressively, treating them as if they're defined in the same scope regardless of line order.

**Location**: `foolish-core/src/ubc.rs` — search resolution logic (likely in `step_one` for `SearchFir`).

### Problem 5: Input File Digital Signature

**Current behavior**: Snapshots are approved without any cryptographic verification of who approved them. There's no way to distinguish whether a snapshot was auto-approved by an AI agent or manually reviewed by a human.

**Target behavior**: Add a digital signature line before `INPUT:...` in the snapshot output:
"""
SIGNATURE: <algorithm>:<public_key_fingerprint>:<signature_hex>
INPUT:
  ...
"""

**Cryptographic scheme**:

1. **Key derivation from passphrase**:
   - The passphrase (string) is used to derive an Ed25519 key pair via Argon2id key derivation
   - `passphrase → Argon2id(passphrase, salt) → Ed25519 seed → (private_key, public_key)`
   - The salt is derived from the input file path (or a fixed per-project salt)

2. **AI agent runs** (normal mode):
   - Passphrase is empty string `""`
   - Key pair is deterministically derived from the empty passphrase
   - The signature is produced by signing the input content with the derived private key
   - The public key fingerprint identifies this as an AI-generated signature

3. **Human reviews/approves**:
   - Human provides a signing passphrase via `--signing-passphrase` CLI parameter or `SIGNING_PASSPHRASE` environment variable
   - Key pair is derived from the human's passphrase (different from AI's empty passphrase)
   - The signature is produced by signing the input content with the human's private key
   - The public key fingerprint identifies this as a human-verified signature

4. **Verification**:
   - Anyone can verify a signature by:
     a. Deriving the public key from the passphrase (or the public key fingerprint)
     b. Verifying the signature against the input content
   - If the signature verifies with the AI passphrase (empty), it's AI-approved
   - If the signature verifies with a human passphrase, it's human-approved

**Signature format**:
"""
SIGNATURE: ed25519:<public_key_hex_64chars>:<signature_hex_128chars>
"""

- `ed25519` — algorithm identifier
- `public_key_hex` — 32-byte Ed25519 public key, hex-encoded (64 chars)
- `signature_hex` — 64-byte Ed25519 signature, hex-encoded (128 chars)

**Why this matters**:
- The public key fingerprint uniquely identifies who signed (AI agent vs human)
- The signature cryptographically binds the input content to the signer
- Tampering with the input content invalidates the signature
- A human inspector can verify whether a snapshot was AI-approved or human-reviewed

**Implementation**:
1. Add `--signing-passphrase` CLI parameter to `foolish-cli` (optional, defaults to empty string)
2. Add `SIGNING_PASSPHRASE` environment variable for test runs (defaults to empty string)
3. Add `ed25519-dalek` and `argon2` crates as dependencies (`ed25519-dalek` provides Ed25519 signing/verification; no need for `ring`)
4. Implement key derivation: `passphrase → Argon2id → Ed25519 keypair`
5. Implement signing: `private_key.sign(input_content) → signature`
6. Include signature line in snapshot output before INPUT

### Problem 5.1: Snapshot Signature Utility (`foolish-sig`)

A small Rust binary crate (`foolish-sig`) provides snapshot signature management: listing, verification, and re-signing.

**Location**: `foolish/foolish-sig/src/main.rs` (new crate in the workspace)

**Dependencies**: `ed25519-dalek`, `argon2`, `clap` (CLI parsing), `serde` (for snapshot parsing)

#### 5.1.1 Command: `list`

List snapshots and classify them by signer type:

"""bash
# List all snapshots, showing signer type
foolish-sig list --input-dir snapshot_tests/input --approved-dir snapshot_tests/approved
"""

**Output**:
"""
Computer-signed (passphrase=""):
  alarm_division_by_zero_deeply_nested.foo.snap
  forward_reference_basic.foo.snap
  ...

Human-signed (passphrase provided):
  complex_brane_with_operations_and_search.foo.snap  [public_key: ab12cd34...]

Unsigned (no SIGNATURE line):
  legacy_test.foo.snap
"""

**How it works**:
1. Scan all `.snap` files in the approved directory
2. Parse the `SIGNATURE:` line from each file
3. Derive the AI public key (from empty passphrase)
4. Compare each signature's public key fingerprint:
   - If matches AI key → "Computer-signed"
   - If doesn't match AI key → "Human-signed"
   - If no SIGNATURE line → "Unsigned"

#### 5.1.2 Command: `verify`

Given a passphrase, check which snapshots are signed by that passphrase:

"""bash
# Verify with a human passphrase (reads from stdin for security)
foolish-sig verify --passphrase-stdin --input-dir snapshot_tests/input --approved-dir snapshot_tests/approved

# Verify with a passphrase from command line (less secure)
foolish-sig verify --passphrase "my-secret" --input-dir snapshot_tests/input --approved-dir snapshot_tests/approved
"""

**Output**:
"""
Verifying passphrase-derived key (public_key: ab12cd34ef56...)

Matched (signed by this passphrase):
  ✓ complex_brane_with_operations_and_search.foo.snap
  ✓ complex_forward_refs_in_nested_branes.foo.snap

Not signed by this passphrase:
  ✗ alarm_division_by_zero_deeply_nested.foo.snap  [computer-signed]
  ✗ forward_reference_basic.foo.snap  [computer-signed]

Unsigned:
  ✗ legacy_test.foo.snap  [no signature]
"""

**How it works**:
1. Read passphrase (from stdin or command line)
2. Derive Ed25519 keypair from passphrase
3. For each `.snap` file:
   a. Parse the `SIGNATURE:` line
   b. Verify the signature against the input content using the derived public key
   c. If verification succeeds → "Matched"
   d. If verification fails → check if it's computer-signed or unsigned

#### 5.1.3 Command: `sign`

Read a passphrase and re-sign a computer-signed snapshot with a human signature. **Crucially, it first verifies that the current signature is the computer signature** before allowing re-signing:

"""bash
# Sign a specific file
foolish-sig sign --passphrase-stdin --input-dir snapshot_tests/input --approved-dir snapshot_tests/approved complex_brane_with_operations_and_search.foo.snap

# Sign all computer-signed files
foolish-sig sign --passphrase-stdin --all --input-dir snapshot_tests/input --approved-dir snapshot_tests/approved
"""

**Flow for single file**:
1. Read the `.snap` file
2. Parse the existing `SIGNATURE:` line
3. Derive the AI public key (from empty passphrase)
4. **Verify** the existing signature is valid AND matches the AI key
   - If NOT computer-signed → **REFUSE** with error: "Cannot re-sign: existing signature is not computer-signed"
   - If unsigned → **REFUSE** with error: "Cannot re-sign: no existing signature to verify"
5. Read the human passphrase (from stdin, hidden input)
6. Derive the human's Ed25519 keypair
7. Read the corresponding `.foo` input file
8. Sign the input content with the human's private key
9. Replace the `SIGNATURE:` line with the new human signature
10. Write the updated `.snap` file (backup original as `.snap.bak`)

**Output**:
"""
File: complex_brane_with_operations_and_search.foo.snap
  Current signature: computer-signed (public_key: 0000aabb...)
  Verifying computer signature... ✓ valid
  Re-signing with human passphrase...
  New signature: human-signed (public_key: ab12cd34ef56...)
  Written: complex_brane_with_operations_and_search.foo.snap
  Backup:  complex_brane_with_operations_and_search.foo.snap.bak
"""

**Flow for `--all`**:
1. List all computer-signed snapshots
2. For each, perform the single-file flow
3. Summary at the end

#### 5.1.4 Key Derivation Details

"""rust
/// Derive Ed25519 keypair from passphrase.
///
/// Uses Argon2id with a fixed per-project salt to ensure deterministic
/// key derivation. The same passphrase always produces the same keypair.
fn derive_keypair(passphrase: &str) -> (SigningKey, VerifyingKey) {
    let salt = b"foolish-rust:snapshot-sig:v1";  // Fixed project salt
    let params = argon2::Params::default();
    let hash = argon2::hash_raw(passphrase.as_bytes(), salt, params)
        .expect("Argon2id derivation failed");
    // Truncate/pad to 32 bytes for Ed25519 seed
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&hash[..32]);
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}
"""

#### 5.1.5 Snapshot File Format

A signed snapshot file has this structure:
"""
---
source: foolish-core/src/lib.rs
SIGNATURE: ed25519:<public_key_hex>:<signature_hex>
---
INPUT:
  ...
"""

The `SIGNATURE:` line appears after the YAML frontmatter but before the `---` separator (or immediately before `INPUT:` if no frontmatter).

The signature covers the **input content** (the `.foo` file content), not the entire snapshot file. This ensures:
- The input content is cryptographically bound to the signer
- The snapshot output (PARSED, RESULT sections) is verified by the test framework (insta)
- Re-signing only requires the input content, not the entire snapshot

## Implementation Plan

### Phase 0: Signing Infrastructure (Foundation)

**Files**: `foolish-core/Cargo.toml`, `foolish-core/src/`, `foolish-cli/src/main.rs`

**This phase must complete first — all other phases depend on it.**

1. Add `ed25519-dalek` and `argon2` crates to `foolish-core` dependencies

2. Implement key derivation (`derive_keypair: &str → (SigningKey, VerifyingKey)`):
   - Argon2id(passphrase, fixed_salt) → 32-byte seed → Ed25519 keypair
   - Fixed salt: `b"foolish-rust:snapshot-sig:v1"`

3. Implement signing (`sign_content: (SigningKey, &str) → (VerifyingKey, Vec<u8>)`):
   - Sign the input content bytes with the private key
   - Return public key + signature for inclusion in snapshot

4. Implement verification (`verify_signature: (VerifyingKey, &str, &[u8]) → bool`):
   - Verify that content was signed by the given public key

5. Add `--signing-passphrase` CLI parameter to `foolish-cli` (defaults to empty string)
6. Add `SIGNING_PASSPHRASE` environment variable for test runs

### Phase 1: Multi-line Input Fix + Signature Integration

**Files**: `foolish-core/src/sequencer.rs`

**(Depends on Phase 0 — uses signing infrastructure from Phase 0)**

1. Modify `Sequencer::format_with_header` to handle multi-line input:
   - If source contains newlines, wrap in ```foolish code fence:
     """
     INPUT:
     ```foolish
     <line 1>
     <line 2>
     ...
     ```
     """
   - The foolish code fence preserves the ORIGINAL CODE exactly as written
   - If source is single-line, use current format: `INPUT: <source>`

2. Modify `Sequencer::format_with_header` to include the SIGNATURE line before INPUT:
   - Use signing infrastructure from Phase 0 to compute Ed25519 signature of source content
   - Insert `SIGNATURE: ed25519:<pubkey_hex>:<sig_hex>` before `INPUT:`

3. Update `UbcEvaluator::evaluate` to pass passphrase to formatter.

4. Regenerate all UBC snapshots (`INSTA_UPDATE=always`).

### Phase 2: Humanizing Sequencer Overhaul

**Files**: `foolish-core/src/sequencer.rs`

1. Define constants:
   - `GROUPING_INDENT: &str = "  "` (2 spaces — added for each structural nesting level)
   - `ALIGN_INDENT` is dynamic: counts the number of characters before the last grouping symbol (`{`, `(`, `[`) on the current line, and adds that many spaces to all lines within the grouping

2. Rewrite `format_fir_q` (the main formatting function) with the new rules:
   - **Atomic values** (Int, NK): inline, NO state suffix for CONSTANT or INDEPENDENT
     - `Int(10)` not `Int(10) [INDEPENDENT]`
     - `??? (division by zero)` (NK always shows, no state suffix)
   - **Non-atomic FIRs**: function-call style with parentheses, state ONLY if pre-constant
     - `Operator(/, [EMBRYONIC]` — state shown because EMBRYONIC is pre-constant
     - `Operator(+)` — no state shown because it's CONSTANT/INDEPENDENT
     - Children indented by GROUPING_INDENT + ALIGN_INDENT
     - Closing `)` aligns with opening operator name
   - **Branes**: brace-delimited
     - `Brane{` with state on first line only if pre-constant
     - Closing `}` aligns with start of `Brane{`
     - Statement body alignment: count characters before the `{` of the body brane, add that many spaces

3. Rewrite `format_fir_simple_indent` (compact formatting for UBCb) to match the new style.

4. Update `HumanizingSequencerRef` to use the new formatting.

5. Regenerate all snapshots.

### Phase 3: WOConstanic Bug Investigation and Fix

**Files**: `foolish-core/src/ubc.rs`

**PRE-IMPLEMENTATION: TWO problems must be investigated and discussed with human before repair.**

**Problem 3A: Search constanic clone retains WOConstanic instead of CONSTANT**
- When `avg = sum / 3` searches for `sum`, the search finds a CONSTANT value (Int(60))
- The constanic clone of `sum` should inherit CONSTANT, not WOConstanic
- **Investigate**: Does UBC evaluation order guarantee that `sum` is fully evaluated to CONSTANT before `avg`'s search runs? If yes, the bug is in the clone logic. If no, the bug is in evaluation ordering.

**Problem 3B: `+` operator shows WOConstanic when it should be something else**
- The `+` operator for `sum = x + y + z` should NOT be WOConstanic
- The operator's children (Search for x, y, z) never become constanic — they resolve inline
- **Investigate**: Why does the `+` operator retain WOConstanic? Is this a state propagation bug or a fundamental misunderstanding of when operators become constanic?

**Steps**:
1. Diagnose Problem 3A: trace `SearchFir::step_one` constanic clone logic
2. Diagnose Problem 3B: trace state propagation for `+` operator
3. Discuss findings with human before implementing fixes
4. Implement fixes for both problems
5. Add regression unit tests
6. Update affected snapshots

### Phase 4: Forward Reference Bug Fix

**Files**: `foolish-core/src/ubc.rs`

1. Locate the search resolution logic in `SearchFir::step_one`
2. When a search finds a name defined later (higher line number), mark the search as ECONSTANIC, not CONSTANT
3. Add unit tests for forward reference behavior
4. Update affected snapshots

### Phase 5: foolish-sig Utility

**Files**: New crate `foolish/foolish-sig/`

**(Depends on Phase 0 — reuses signing/verification infrastructure from `foolish-core`)**

1. Create `foolish-sig/Cargo.toml` with dependencies: `ed25519-dalek`, `argon2`, `clap`, `foolish-core` (for shared signing infrastructure)
2. Implement `list` command — classify snapshots as computer-signed/human-signed/unsigned
3. Implement `verify` command — verify snapshots against a passphrase
4. Implement `sign` command — re-sign computer-signed snapshots with human signature (with verification of existing computer signature)
5. Add `foolish-sig` to workspace Cargo.toml

## Verification Criteria

1. `cargo check --workspace` passes
2. `cargo test -p foolish-core --lib` passes (265+ tests)
3. `cargo test -p foolish-ubcb-cli --lib` passes (pre-existing failure unchanged)
4. Multi-line input files are preserved in snapshots with ```foolish code fence
5. New formatting matches the examples in this spec (grouping/alignment indent, parentheses for non-atomic FIRs, braces for branes)
6. WOConstanic bug is fixed (sum is CONSTANT in complex_brane_with_operations_and_search) — AFTER human discussion
7. Forward reference bug is fixed (val is ECONSTANIC in complex_forward_refs_in_nested_branes)
8. Ed25519 digital signatures are present in snapshot output
9. `foolish-sig list` correctly classifies computer-signed vs human-signed snapshots
10. `foolish-sig sign` refuses to re-sign non-computer-signed snapshots

## Dependencies

New crates:
- `ed25519-dalek` — Ed25519 signing and verification (for `foolish-core` and `foolish-sig`)
- `argon2` — Key derivation from passphrase (for `foolish-core` and `foolish-sig`)
- `clap` — CLI argument parsing (for `foolish-sig`)

No `ring` needed — `ed25519-dalek` provides all required cryptographic functionality.

## Risks

- **High**: Humanizing sequencer overhaul affects all 136+ snapshots — complete regeneration required
- **High**: WOConstanic and forward reference fixes may change evaluation semantics — requires careful regression testing
- **Medium**: ALIGN_INDENT calculation (counting characters before grouping symbol) is a new concept — edge cases need testing
- **Low**: Digital signature feature is additive and doesn't break existing functionality

## Notes

- GROUPING_INDENT is 2 spaces. ALIGN_INDENT is dynamic (counts characters before the last grouping symbol).
- The new formatting style is inspired by Rust function call syntax, making FIR output more readable and closer to the source code.
- Phase 3 (WOConstanic) requires human discussion before implementation — two problems are identified but not yet diagnosed.
- Digital signatures use Ed25519 with Argon2id key derivation — the same passphrase always produces the same keypair.
- `foolish-sig` is a separate binary crate to keep the signing tooling independent from the main CLI.
