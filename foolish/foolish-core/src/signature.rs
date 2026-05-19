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

use ed25519_dalek::{Signer, SigningKey, VerifyingKey, Signature};
use argon2::Argon2;

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
        // Verify the default "computer/AI agent" keypair works end-to-end
        let (sk, vk) = derive_keypair("");
        let content = "{}";

        let (_, sig) = sign_content(&sk, content);
        assert!(
            verify_signature(&vk, content, &sig),
            "Empty passphrase keypair must support full sign/verify cycle"
        );
    }
}
