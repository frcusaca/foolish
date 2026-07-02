/// Ed25519 snapshot signing infrastructure.
///
/// Derives deterministic Ed25519 keypairs from passphrases via Argon2id,
/// signs content, and verifies signatures.
///
/// ## Design
///
/// - **derive_keypair**: `passphrase` → Argon2id (fixed salt) → 32-byte seed → Ed25519 keypair.
///   Deterministic: same passphrase always yields the same keypair.
///   Empty passphrase (`""`) is valid and produces the default "computer/AI agent" keypair.
///
/// - **sign_content**: Signs UTF-8 content bytes with the given `SigningKey`.
///   Returns `(verifying_key, signature_bytes)`.
///
/// - **verify_signature**: Returns `true` if `signature` is a valid Ed25519 signature
///   of `content` bytes under `verifying_key`.
///
/// ## High-level snapshot API
///
/// - **canonicalize_input** / **canonicalize_output_block** / **canonicalize_comment_block**:
///   strip whitespace, append `\n`.
/// - **canonicalize_all_outputs**: canonicalize each block and join with `\n`.
/// - **sign_snapshot**: progressive triple-signing; each sig covers all preceding blocks.
/// - **verify_snapshot**: verify all three progressive signatures; return [`SnapshotVerification`].
/// - **parse_snapshot_footer**: extract [`SnapshotSignature`] from the SIGNATURES section.
///
/// ## Progressive signing
///
/// Each signature covers the cumulative canonical content up to and including its own block:
/// - `foolish_sig`  = sign(`canon_input`)
/// - `hs_sig`       = sign(`canon_input + canon_hs`)
/// - `comments_sig` = sign(`canon_input + canon_hs + canon_comments`)
///
/// Canonical blocks already end with `\n`, so concatenation is unambiguous.
/// This means tampering with any earlier block invalidates all later signatures.

use ed25519_dalek::{Signer, SigningKey, VerifyingKey, Signature};
use argon2::Argon2;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;

/// Fixed salt for deterministic key derivation.
const SALT: &[u8] = b"foolish-rust:snapshot-sig:v1";

/// Derive an Ed25519 keypair from a passphrase.
///
/// Uses Argon2id with a fixed salt to produce a 32-byte seed, which is then
/// used to construct a deterministic Ed25519 keypair.
///
/// # Determinism
///
/// The same passphrase always produces the same keypair. This is intentional:
/// it allows human and AI agents to reproduce signing keys from a known passphrase.
///
/// # Arguments
///
/// * `passphrase` - The passphrase string. May be empty; empty produces
///   the default keypair used by computers and AI agents.
///
/// # Returns
///
/// A `(SigningKey, VerifyingKey)` tuple ready for signing and verification.
pub fn derive_keypair(passphrase: &str) -> (SigningKey, VerifyingKey) {
    let argon2 = Argon2::default();

    // Produce a 32-byte hash suitable as an Ed25519 seed.
    let mut hash_output = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), SALT, &mut hash_output)
        .expect("Argon2id hash should not fail with valid inputs");

    let signing_key = SigningKey::from(&hash_output.into());
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Sign content bytes with the given signing key.
///
/// # Arguments
///
/// * `signing_key` - The Ed25519 signing key.
/// * `content` - The UTF-8 string to sign (signed as its byte representation).
///
/// # Returns
///
/// A tuple of `(VerifyingKey, Vec<u8>)` where the second element is the
/// 64-byte Ed25519 signature.
pub fn sign_content(
    signing_key: &SigningKey,
    content: &str,
) -> (VerifyingKey, Vec<u8>) {
    let signature: Signature = signing_key.sign(content.as_bytes());
    (signing_key.verifying_key(), signature.to_bytes().to_vec())
}

/// Verify an Ed25519 signature.
///
/// # Arguments
///
/// * `verifying_key` - The public key to verify against.
/// * `content` - The original UTF-8 content (verified as its byte representation).
/// * `signature` - The raw signature bytes (expected 64 bytes for Ed25519).
///
/// # Returns
///
/// `true` if the signature is valid; `false` otherwise.
/// Returns `false` for malformed signatures rather than panicking.
pub fn verify_signature(
    verifying_key: &VerifyingKey,
    content: &str,
    signature: &[u8],
) -> bool {
    let sig = match Signature::from_slice(signature) {
        Ok(s) => s,
        Err(_) => return false,
    };
    verifying_key.verify_strict(content.as_bytes(), &sig).is_ok()
}

// ============================================================================
// Canonicalization
// ============================================================================

/// Canonicalize a Foolish input source for signing.
///
/// Strips leading and trailing whitespace (including final newline), then
/// appends exactly one `\n`. This ensures the signed content is deterministic
/// regardless of trailing whitespace in the `.foo` file.
pub fn canonicalize_input(source: &str) -> String {
    let mut s = source.trim().to_string();
    s.push('\n');
    s
}

/// Canonicalize a single HS output block for signing.
///
/// Same rule as `canonicalize_input`: strip whitespace, append one `\n`.
pub fn canonicalize_output_block(output: &str) -> String {
    let mut s = output.trim().to_string();
    s.push('\n');
    s
}

/// Canonicalize a comments block for signing.
///
/// Same rule as the other blocks: strip whitespace, append one `\n`.
pub fn canonicalize_comment_block(text: &str) -> String {
    let mut s = text.trim().to_string();
    s.push('\n');
    s
}

/// Canonicalize all HS output blocks and join them for signing.
///
/// Each block is passed through `canonicalize_output_block`, then joined
/// with `\n` as a separator between blocks. An empty slice returns `"\n"`.
pub fn canonicalize_all_outputs(outputs: &[String]) -> String {
    if outputs.is_empty() {
        return "\n".to_string();
    }
    outputs.iter()
        .map(|o| canonicalize_output_block(o))
        .collect::<Vec<_>>()
        .join("\n")
}

// ============================================================================
// High-level snapshot signing
// ============================================================================

/// The SIGNATURES section appended to a signed snapshot file.
///
/// All three signatures are progressive: each covers the cumulative canonical
/// content up to and including its own block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotSignature {
    /// Hex-encoded 32-byte Ed25519 verifying key.
    pub public_key_hex: String,
    /// Base64-encoded 64-byte Ed25519 signature over `canon_input`.
    pub foolish_sig_b64: String,
    /// Base64-encoded 64-byte Ed25519 signature over `canon_input + canon_hs`.
    pub hs_sig_b64: String,
    /// Base64-encoded 64-byte Ed25519 signature over `canon_input + canon_hs + canon_comments`.
    pub comments_sig_b64: String,
}

impl SnapshotSignature {
    /// Render the SIGNATURES section: label line followed by the four key/sig lines.
    pub fn format_footer(&self) -> String {
        format!(
            "SIGNATURES:\nPublic key: {}\nFoolish signature: {}\nHFS signature: {}\nComments signature: {}",
            self.public_key_hex,
            self.foolish_sig_b64,
            self.hs_sig_b64,
            self.comments_sig_b64,
        )
    }
}

/// Result of verifying a snapshot's three progressive signatures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotVerification {
    /// The passphrase-derived public key matches the key embedded in the footer.
    pub key_match: bool,
    /// The Foolish signature verifies over `canon_input`.
    pub foolish_ok: bool,
    /// The HFS signature verifies over `canon_input + canon_hs`.
    pub hs_ok: bool,
    /// The comments signature verifies over `canon_input + canon_hs + canon_comments`.
    pub comments_ok: bool,
}

impl SnapshotVerification {
    /// Returns `true` if all four checks pass.
    pub fn all_ok(&self) -> bool {
        self.key_match && self.foolish_ok && self.hs_ok && self.comments_ok
    }

    /// Short status string for CLI output.
    pub fn status_line(&self) -> String {
        let key      = if self.key_match    { "match" } else { "no_match" };
        let foolish  = if self.foolish_ok   { "ok" } else { "fail" };
        let hs       = if self.hs_ok        { "ok" } else { "fail" };
        let comments = if self.comments_ok  { "ok" } else { "fail" };
        format!("key={key} foolish={foolish} hs={hs} comments={comments}")
    }
}

/// Sign a snapshot using progressive Ed25519 signatures.
///
/// Each signature covers the cumulative canonical content up to its block:
/// - `foolish_sig`  = sign(`canon_input`)
/// - `hs_sig`       = sign(`canon_input + canon_hs`)
/// - `comments_sig` = sign(`canon_input + canon_hs + canon_comments`)
///
/// Pass `passphrase = ""` for the default computer/AI-agent key.
pub fn sign_snapshot(
    passphrase: &str,
    input: &str,
    hs_outputs: &[String],
    comments: &str,
) -> SnapshotSignature {
    let canon_input    = canonicalize_input(input);
    let canon_hs       = canonicalize_all_outputs(hs_outputs);
    let canon_comments = canonicalize_comment_block(comments);
    let (sk, vk) = derive_keypair(passphrase);
    let hs_content       = format!("{canon_input}{canon_hs}");
    let comments_content = format!("{canon_input}{canon_hs}{canon_comments}");
    let (_, foolish_sig_bytes)  = sign_content(&sk, &canon_input);
    let (_, hs_sig_bytes)       = sign_content(&sk, &hs_content);
    let (_, comments_sig_bytes) = sign_content(&sk, &comments_content);
    SnapshotSignature {
        public_key_hex:   hex::encode(vk.to_bytes()),
        foolish_sig_b64:  B64.encode(&foolish_sig_bytes),
        hs_sig_b64:       B64.encode(&hs_sig_bytes),
        comments_sig_b64: B64.encode(&comments_sig_bytes),
    }
}

/// Verify a snapshot's three progressive signatures.
///
/// Uses the key embedded in `sig`. `key_match` reports whether `passphrase`
/// derives the same public key as `sig.public_key_hex`.
pub fn verify_snapshot(
    passphrase: &str,
    input: &str,
    hs_outputs: &[String],
    comments: &str,
    sig: &SnapshotSignature,
) -> SnapshotVerification {
    let fail = |key_match| SnapshotVerification {
        key_match, foolish_ok: false, hs_ok: false, comments_ok: false
    };
    let vk_bytes: [u8; 32] = match hex::decode(&sig.public_key_hex)
        .ok()
        .and_then(|v| v.try_into().ok())
    {
        Some(b) => b,
        None => return fail(false),
    };
    let vk = match VerifyingKey::from_bytes(&vk_bytes) {
        Ok(k) => k,
        Err(_) => return fail(false),
    };
    let (_, derived_vk) = derive_keypair(passphrase);
    let key_match = derived_vk.to_bytes() == vk.to_bytes();

    let canon_input    = canonicalize_input(input);
    let canon_hs       = canonicalize_all_outputs(hs_outputs);
    let canon_comments = canonicalize_comment_block(comments);
    let hs_content       = format!("{canon_input}{canon_hs}");
    let comments_content = format!("{canon_input}{canon_hs}{canon_comments}");

    let decode = |b64: &str| B64.decode(b64).ok();
    let foolish_sig_bytes  = match decode(&sig.foolish_sig_b64)  { Some(b) => b, None => return fail(key_match) };
    let hs_sig_bytes       = match decode(&sig.hs_sig_b64)       { Some(b) => b, None => return fail(key_match) };
    let comments_sig_bytes = match decode(&sig.comments_sig_b64) { Some(b) => b, None => return fail(key_match) };

    let foolish_ok  = verify_signature(&vk, &canon_input,       &foolish_sig_bytes);
    let hs_ok       = verify_signature(&vk, &hs_content,        &hs_sig_bytes);
    let comments_ok = verify_signature(&vk, &comments_content,  &comments_sig_bytes);
    SnapshotVerification { key_match, foolish_ok, hs_ok, comments_ok }
}

/// Extract a [`SnapshotSignature`] from the SIGNATURES section of a snapshot file.
///
/// Finds the last `SIGNATURES:` label line and parses the three key/sig lines that follow.
/// Returns `None` if the label or any required line is missing or malformed.
pub fn parse_snapshot_footer(snapshot_text: &str) -> Option<SnapshotSignature> {
    let lines: Vec<&str> = snapshot_text.lines().collect();
    let sig_pos = lines.iter().rposition(|l| l.trim() == "SIGNATURES:")?;
    let after: Vec<&str> = lines[sig_pos + 1..]
        .iter()
        .filter(|l| !l.trim().is_empty())
        .copied()
        .collect();
    if after.len() < 4 {
        return None;
    }
    let public_key_hex   = after[0].strip_prefix("Public key: ")?.trim().to_string();
    let foolish_sig_b64  = after[1].strip_prefix("Foolish signature: ")?.trim().to_string();
    let hs_sig_b64       = after[2].strip_prefix("HFS signature: ")?.trim().to_string();
    let comments_sig_b64 = after[3].strip_prefix("Comments signature: ")?.trim().to_string();
    if public_key_hex.is_empty() || foolish_sig_b64.is_empty()
        || hs_sig_b64.is_empty() || comments_sig_b64.is_empty()
    {
        return None;
    }
    Some(SnapshotSignature { public_key_hex, foolish_sig_b64, hs_sig_b64, comments_sig_b64 })
}

// ============================================================================
// Legacy single-sig API (deprecated)
// ============================================================================

/// Sign `input` with the default keypair and return a `SIG:` line.
#[deprecated(since = "0.0.0", note = "Use `sign_snapshot` instead")]
pub fn sign_input_line(input: &str) -> String {
    let (sk, vk) = derive_keypair("");
    let (_, sig) = sign_content(&sk, input);
    let b64 = B64.encode(&sig);
    format!("SIG: {} {}", hex::encode(&vk.to_bytes()), b64)
}

/// Verify a `SIG:` line against the original input.
#[deprecated(since = "0.0.0", note = "Use `verify_snapshot` instead")]
pub fn verify_input_line(line: &str, input: &str) -> bool {
    let Some(rest) = line.strip_prefix("SIG: ") else {
        return false;
    };
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() != 2 {
        return false;
    }
    let vk_bytes: [u8; 32] = match hex::decode(parts[0])
        .ok()
        .and_then(|v| v.try_into().ok())
    {
        Some(b) => b,
        None => return false,
    };
    let vk = match VerifyingKey::from_bytes(&vk_bytes) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let sig_bytes: Vec<u8> = match B64.decode(parts[1]) {
        Ok(b) => b,
        Err(_) => return false,
    };
    verify_signature(&vk, input, &sig_bytes)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_keypair_empty_is_deterministic() {
        let (sk1, vk1) = derive_keypair("");
        let (sk2, vk2) = derive_keypair("");

        assert_eq!(
            vk1.to_bytes(),
            vk2.to_bytes(),
            "derive_keypair(\"\") must produce the same keypair on repeated calls"
        );
        assert_eq!(
            sk1.to_bytes(),
            sk2.to_bytes(),
            "Signing keys must also match"
        );
    }

    #[test]
    fn derive_keypair_different_passphrases_produce_different_keys() {
        let (_, vk_empty) = derive_keypair("");
        let (_, vk_human) = derive_keypair("human");

        assert_ne!(
            vk_empty.to_bytes(),
            vk_human.to_bytes(),
            "Different passphrases must produce different verifying keys"
        );
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let (sk, vk) = derive_keypair("test-passphrase");
        let content = "snapshot content to sign";

        let (returned_vk, sig_bytes) = sign_content(&sk, content);
        assert_eq!(vk.to_bytes(), returned_vk.to_bytes(), "Returned verifying key must match");
        assert_eq!(sig_bytes.len(), 64, "Ed25519 signature must be 64 bytes");

        assert!(
            verify_signature(&vk, content, &sig_bytes),
            "Valid signature must verify successfully"
        );
    }

    #[test]
    fn verify_fails_with_wrong_key() {
        let (sk_a, _) = derive_keypair("key-a");
        let (_, vk_b) = derive_keypair("key-b");
        let content = "signed with key-a";

        let (_, sig_bytes) = sign_content(&sk_a, content);

        assert!(
            !verify_signature(&vk_b, content, &sig_bytes),
            "Signature made with key-a must fail verification with key-b"
        );
    }

    #[test]
    fn verify_fails_with_tampered_content() {
        let (_, vk) = derive_keypair("test");
        let original = "original content";
        let tampered = "tampered content";

        let (_, sig_bytes) = sign_content(&derive_keypair("test").0, original);

        assert!(
            !verify_signature(&vk, tampered, &sig_bytes),
            "Signature must fail when content is tampered"
        );
    }

    #[test]
    fn verify_fails_with_truncated_signature() {
        let (_, vk) = derive_keypair("test");
        let content = "some content";
        let (_, full_sig) = sign_content(&derive_keypair("test").0, content);

        // Truncate signature to 32 bytes (half of valid 64)
        let truncated: Vec<u8> = full_sig[..32].to_vec();

        assert!(
            !verify_signature(&vk, content, &truncated),
            "Truncated signature must fail verification"
        );
    }

    #[test]
    fn verify_fails_with_empty_signature() {
        let (_, vk) = derive_keypair("test");
        let content = "content";

        assert!(
            !verify_signature(&vk, content, &[]),
            "Empty signature must fail verification"
        );
    }

    #[test]
    fn empty_passphrase_signing_roundtrip() {
        let (sk, vk) = derive_keypair("");
        let content = "{}";
        let (_, sig) = sign_content(&sk, content);
        assert!(verify_signature(&vk, content, &sig));
    }

    // ---- canonicalization ----

    #[test]
    fn canonicalize_input_empty() {
        assert_eq!(canonicalize_input(""), "\n");
    }

    #[test]
    fn canonicalize_input_strips_whitespace_and_appends_newline() {
        assert_eq!(canonicalize_input("  hello  \n\n"), "hello\n");
        assert_eq!(canonicalize_input("hello"), "hello\n");
    }

    #[test]
    fn canonicalize_input_idempotent_on_already_canonical() {
        let canon = canonicalize_input("hello");
        assert_eq!(canonicalize_input(&canon), canon);
    }

    #[test]
    fn canonicalize_output_block_mirrors_input_rules() {
        assert_eq!(canonicalize_output_block(""), "\n");
        assert_eq!(canonicalize_output_block("  result  \n"), "result\n");
    }

    #[test]
    fn canonicalize_all_outputs_empty_slice() {
        assert_eq!(canonicalize_all_outputs(&[]), "\n");
    }

    #[test]
    fn canonicalize_all_outputs_single_block() {
        let blocks = vec!["  Brane [NK]\n".to_string()];
        assert_eq!(canonicalize_all_outputs(&blocks), "Brane [NK]\n");
    }

    #[test]
    fn canonicalize_all_outputs_multiple_blocks_joined_with_newline() {
        let blocks = vec!["block a\n".to_string(), "block b\n".to_string()];
        let result = canonicalize_all_outputs(&blocks);
        assert_eq!(result, "block a\n\nblock b\n");
    }

    // ---- canonicalize_comment_block ----

    #[test]
    fn canonicalize_comment_block_empty() {
        assert_eq!(canonicalize_comment_block(""), "\n");
    }

    #[test]
    fn canonicalize_comment_block_strips_outer_whitespace_only() {
        assert_eq!(canonicalize_comment_block("  line1\n  line2  \n"), "line1\n  line2\n");
    }

    #[test]
    fn canonicalize_comment_block_idempotent() {
        let canon = canonicalize_comment_block("test_name\nI approve");
        assert_eq!(canonicalize_comment_block(&canon), canon);
    }

    // ---- sign_snapshot / verify_snapshot ----

    #[test]
    fn sign_snapshot_roundtrip_empty_passphrase() {
        let input = "{ x = 1 }";
        let hs_outputs = vec!["x = Int(1)".to_string()];
        let comments = "test_name";
        let sig = sign_snapshot("", input, &hs_outputs, comments);
        assert!(!sig.public_key_hex.is_empty());
        let v = verify_snapshot("", input, &hs_outputs, comments, &sig);
        assert!(v.all_ok(), "Round-trip must pass: {:?}", v);
    }

    #[test]
    fn verify_snapshot_fails_tampered_input() {
        let input = "{ x = 1 }";
        let hs_outputs = vec!["x = Int(1)".to_string()];
        let comments = "test_name";
        let sig = sign_snapshot("", input, &hs_outputs, comments);
        // Tampered input fails foolish_sig; hs_sig and comments_sig also fail
        // because they are progressive (include input in their content)
        let v = verify_snapshot("", "{ x = 2 }", &hs_outputs, comments, &sig);
        assert!(!v.foolish_ok,  "Tampered input must fail foolish_ok");
        assert!(!v.hs_ok,       "Tampered input must also fail hs_ok (progressive)");
        assert!(!v.comments_ok, "Tampered input must also fail comments_ok (progressive)");
    }

    #[test]
    fn verify_snapshot_fails_tampered_hs_output() {
        let input = "{ x = 1 }";
        let hs_outputs = vec!["x = Int(1)".to_string()];
        let comments = "test_name";
        let sig = sign_snapshot("", input, &hs_outputs, comments);
        let tampered = vec!["x = Int(99)".to_string()];
        // Tampered HS fails hs_sig and comments_sig but not foolish_sig
        let v = verify_snapshot("", input, &tampered, comments, &sig);
        assert!( v.foolish_ok,  "Untouched input must still pass foolish_ok");
        assert!(!v.hs_ok,       "Tampered HS must fail hs_ok");
        assert!(!v.comments_ok, "Tampered HS must also fail comments_ok (progressive)");
    }

    #[test]
    fn verify_snapshot_fails_tampered_comments() {
        let input = "{ x = 1 }";
        let hs_outputs = vec!["x = Int(1)".to_string()];
        let sig = sign_snapshot("", input, &hs_outputs, "test_name");
        // Tampered comments fails only comments_sig
        let v = verify_snapshot("", input, &hs_outputs, "test_name\ntampered", &sig);
        assert!( v.foolish_ok,  "Untouched input must still pass foolish_ok");
        assert!( v.hs_ok,       "Untouched HS must still pass hs_ok");
        assert!(!v.comments_ok, "Tampered comments must fail comments_ok");
    }

    #[test]
    fn verify_snapshot_key_mismatch_different_passphrase() {
        let input = "{ x = 1 }";
        let hs = vec!["x = Int(1)".to_string()];
        let comments = "test_name";
        let sig = sign_snapshot("human-secret", input, &hs, comments);
        let v = verify_snapshot("", input, &hs, comments, &sig);
        assert!(!v.key_match, "Wrong passphrase must not match key");
        // All three sigs still verify because they use the embedded public key
        assert!(v.foolish_ok);
        assert!(v.hs_ok);
        assert!(v.comments_ok);
    }

    // ---- parse_snapshot_footer ----

    #[test]
    fn parse_snapshot_footer_well_formed() {
        let sig = sign_snapshot("", "{ x = 1 }", &["x = Int(1)".to_string()], "test_name");
        let footer = sig.format_footer();
        let snapshot = format!("---\nsome content\n---\nINPUT:\n...\n{footer}\n");
        let parsed = parse_snapshot_footer(&snapshot).expect("should parse footer");
        assert_eq!(parsed, sig);
    }

    #[test]
    fn parse_snapshot_footer_missing_returns_none() {
        assert!(parse_snapshot_footer("").is_none());
        assert!(parse_snapshot_footer("no SIGNATURES label here").is_none());
    }

    #[test]
    fn parse_snapshot_footer_no_signatures_label_returns_none() {
        // Lines with the right prefixes but no SIGNATURES: marker → None
        let old = "Public key: abc\nFoolish signature: def\nHFS signature: ghi\nComments signature: jkl";
        assert!(parse_snapshot_footer(old).is_none());
    }

    #[test]
    fn parse_snapshot_footer_wrong_prefixes_returns_none() {
        let bad = "SIGNATURES:\nPublic key: abc\nBad prefix: xyz\nHFS signature: def\nComments signature: jkl";
        assert!(parse_snapshot_footer(bad).is_none());
    }
}
