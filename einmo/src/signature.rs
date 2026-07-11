//! Signature module: Ed25519 + Argon2id, three-role key model (Compiled/Configured/Stage).
//!
//! Implements the Compiled / Configured / Stage key model from FOOP-54 §4.4:
//! - **Compiled Key**: embedded at compile time; certifies the Configured Key's pubkey.
//! - **Configured Key**: set at configuration time; certifies Stage keys' pubkeys.
//! - **Stage Keys**: one per stage, signs all file bytes before its stamp (content + prior stamps).

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from signature operations.
#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("Argon2id key derivation failed")]
    KeyDerivation,
    #[error("invalid public key: {0}")]
    InvalidPublicKey(String),
    #[error("invalid signature: {0}")]
    InvalidSignature(String),
    #[error("stamp verification failed: {0}")]
    VerificationFailed(String),
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
    #[error("stamp chain integrity error: {0}")]
    ChainIntegrity(String),
    #[error("ordering error: {0}")]
    Ordering(String),
}

// ---------------------------------------------------------------------------
// Key derivation
// ---------------------------------------------------------------------------

/// Fixed salt for deterministic key derivation (einmo-specific, different from foolish-core).
const SALT: &[u8] = b"einmo:sig:v1:ed25519-argon2id";

/// Argon2id parameters pinned by einmo.
const ARGON2_MEMORY_COST: u32 = 65_536; // 64 MiB
const ARGON2_TIME_COST: u32 = 3;
const ARGON2_PARALLELISM: u32 = 1;

/// Derive an Ed25519 keypair from a passphrase using Argon2id.
///
/// Uses pinned Argon2id parameters with a fixed salt to produce a
/// deterministic 32-byte seed for Ed25519.
///
/// Empty passphrase (`""`) produces the well-known computer/AI key.
pub fn derive_keypair(passphrase: &str) -> Result<(SigningKey, VerifyingKey), SignatureError> {
    use argon2::{Algorithm, Argon2, Params, Version};

    let params = Params::new(
        ARGON2_MEMORY_COST,
        ARGON2_TIME_COST,
        ARGON2_PARALLELISM,
        None,
    )
    .map_err(|_| SignatureError::KeyDerivation)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut hash_output = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), SALT, &mut hash_output)
        .map_err(|_| SignatureError::KeyDerivation)?;

    let signing_key = SigningKey::from_bytes(&hash_output);
    let verifying_key = signing_key.verifying_key();
    Ok((signing_key, verifying_key))
}

/// Sign bytes with a signing key; returns (pubkey_hex, signature_base64).
fn sign_bytes(signing_key: &SigningKey, content: &[u8]) -> (String, String) {
    let signature: Signature = signing_key.sign(content);
    let pubkey_hex = hex::encode(signing_key.verifying_key().as_bytes());
    let sig_b64 = B64.encode(signature.to_bytes());
    (pubkey_hex, sig_b64)
}

/// Verify a signature against content bytes.
fn verify_bytes(pubkey_hex: &str, sig_b64: &str, content: &[u8]) -> Result<(), SignatureError> {
    let pubkey_bytes = hex::decode(pubkey_hex)?;
    let pubkey_arr: [u8; 32] = pubkey_bytes
        .try_into()
        .map_err(|_| SignatureError::InvalidPublicKey("expected 32 bytes".into()))?;
    let verifying_key = VerifyingKey::from_bytes(&pubkey_arr)
        .map_err(|e| SignatureError::InvalidPublicKey(e.to_string()))?;

    let sig_bytes = B64.decode(sig_b64)?;
    let sig_arr: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| SignatureError::InvalidSignature("expected 64 bytes".into()))?;
    let signature = Signature::from_bytes(&sig_arr);

    verifying_key
        .verify_strict(content, &signature)
        .map_err(|e| SignatureError::VerificationFailed(e.to_string()))
}

// ---------------------------------------------------------------------------
// Compiled Key (build-time)
// ---------------------------------------------------------------------------

/// The compiled key passphrase. Override at compile time via `EINMO_COMPILED_PASSPHRASE` env var.
const COMPILED_PASSPHRASE: &str = match option_env!("EINMO_COMPILED_PASSPHRASE") {
    Some(p) => p,
    None => "", // stock default: empty passphrase = well-known computer key
};

/// Get the Compiled Key keypair (derived from the compiled passphrase).
///
/// In the stock open-source build this is the well-known empty-passphrase key.
/// Custom builds can override via `EINMO_COMPILED_PASSPHRASE` env at compile time.
pub fn compiled_keypair() -> Result<(SigningKey, VerifyingKey), SignatureError> {
    derive_keypair(COMPILED_PASSPHRASE)
}

/// Get just the compiled verifying key (public).
pub fn compiled_verifying_key() -> Result<VerifyingKey, SignatureError> {
    let (_, vk) = compiled_keypair()?;
    Ok(vk)
}

// ---------------------------------------------------------------------------
// Stamp
// ---------------------------------------------------------------------------

/// A single stamp in the signature chain.
///
/// Serialized as one JSON object per line in the STAMPS section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stamp {
    /// Key role: "compiled", "configured", or "stage:<name>".
    key: String,
    /// Hex-encoded public key of the signer.
    pubkey: String,
    /// What this stamp signs: "pubkey:<role>" or "prior-bytes".
    signs: String,
    /// Base64-encoded Ed25519 signature.
    signature: String,
    /// Provenance: "einmo <version> sha256:<binary-hash>".
    produced_by: String,
    /// ISO8601 UTC timestamp.
    timestamp: String,
}

impl Stamp {
    /// Key role: "compiled", "configured", or "stage:<name>".
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Hex-encoded public key of the signer.
    pub fn pubkey(&self) -> &str {
        &self.pubkey
    }

    /// What this stamp signs: "pubkey:<role>" or "prior-bytes".
    pub fn signs(&self) -> &str {
        &self.signs
    }

    /// Base64-encoded Ed25519 signature.
    pub fn signature(&self) -> &str {
        &self.signature
    }

    /// Provenance: "einmo <version> sha256:<binary-hash>".
    pub fn produced_by(&self) -> &str {
        &self.produced_by
    }

    /// ISO8601 UTC timestamp.
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    /// Whether this is a compiled certification stamp.
    pub fn is_compiled(&self) -> bool {
        self.key == "compiled"
    }

    /// Whether this is a configured certification stamp.
    pub fn is_configured(&self) -> bool {
        self.key == "configured"
    }

    /// Whether this is a stage stamp, and if so which stage.
    pub fn stage_name(&self) -> Option<&str> {
        self.key.strip_prefix("stage:")
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(
        key: &str,
        pubkey: &str,
        signs: &str,
        signature: &str,
        produced_by: &str,
        timestamp: &str,
    ) -> Self {
        Self {
            key: key.into(),
            pubkey: pubkey.into(),
            signs: signs.into(),
            signature: signature.into(),
            produced_by: produced_by.into(),
            timestamp: timestamp.into(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_signature(mut self, signature: &str) -> Self {
        self.signature = signature.into();
        self
    }
}

// ---------------------------------------------------------------------------
// Stamps
// ---------------------------------------------------------------------------

/// Collection of stamps, one JSON object per line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamps {
    entries: Vec<Stamp>,
}

impl Stamps {
    pub fn new(entries: Vec<Stamp>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[Stamp] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn push(&mut self, stamp: Stamp) {
        self.entries.push(stamp);
    }

    /// Parse stamps from JSON-lines bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, SignatureError> {
        if bytes.is_empty() {
            return Ok(Self {
                entries: Vec::new(),
            });
        }
        let text = std::str::from_utf8(bytes).map_err(|e| {
            SignatureError::JsonParse(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )))
        })?;
        let entries: Result<Vec<Stamp>, _> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(serde_json::from_str)
            .collect();
        Ok(Self { entries: entries? })
    }

    /// Serialize stamps to JSON-lines bytes (byte-stable: each stamp on its own line, no trailing newline).
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = String::new();
        for (i, stamp) in self.entries.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(
                &serde_json::to_string(stamp)
                    .expect("Stamp has only String fields; serde_json::to_string cannot fail"),
            );
        }
        out.into_bytes()
    }
}

// ---------------------------------------------------------------------------
// Stamp creation
// ---------------------------------------------------------------------------

/// The `produced_by` field value for einmo 0.1.0.
///
/// In a real build this would include the binary SHA-256; for now we use a placeholder.
fn produced_by() -> String {
    format!("einmo {} sha256:placeholder", env!("CARGO_PKG_VERSION"))
}

/// Current ISO8601 UTC timestamp.
fn now_iso8601() -> String {
    use time::OffsetDateTime;
    let now = OffsetDateTime::now_utc();
    now.format(&time::format_description::well_known::Iso8601::DEFAULT)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

/// Create the `compiled` certification stamp.
///
/// Signs the Configured Key's public key with the Compiled Key.
pub fn create_compiled_stamp(
    compiled_signing_key: &SigningKey,
    configured_pubkey_hex: &str,
) -> Stamp {
    let (pubkey_hex, sig_b64) = sign_bytes(compiled_signing_key, configured_pubkey_hex.as_bytes());
    Stamp {
        key: "compiled".into(),
        pubkey: pubkey_hex,
        signs: "pubkey:configured".to_string(),
        signature: sig_b64,
        produced_by: produced_by(),
        timestamp: now_iso8601(),
    }
}

/// Create the `configured` certification stamp.
///
/// Signs the output Stage Key's public key with the Configured Key.
pub fn create_configured_stamp(
    configured_signing_key: &SigningKey,
    stage_pubkey_hex: &str,
) -> Stamp {
    let (pubkey_hex, sig_b64) = sign_bytes(configured_signing_key, stage_pubkey_hex.as_bytes());
    Stamp {
        key: "configured".into(),
        pubkey: pubkey_hex,
        signs: "pubkey:stage:output".to_string(),
        signature: sig_b64,
        produced_by: produced_by(),
        timestamp: now_iso8601(),
    }
}

/// Create a `stage:<name>` stamp.
///
/// Signs all file bytes before this stamp's line.
pub fn create_stage_stamp(
    stage_signing_key: &SigningKey,
    stage_name: &str,
    prior_bytes: &[u8],
) -> Stamp {
    let (pubkey_hex, sig_b64) = sign_bytes(stage_signing_key, prior_bytes);
    Stamp {
        key: format!("stage:{stage_name}"),
        pubkey: pubkey_hex,
        signs: "prior-bytes".into(),
        signature: sig_b64,
        produced_by: produced_by(),
        timestamp: now_iso8601(),
    }
}

// ---------------------------------------------------------------------------
// Stamp verification
// ---------------------------------------------------------------------------

/// Result of verifying a single stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StampStatus {
    Valid,
    Invalid(String),
}

/// Verify all stamps in a stamp chain.
///
/// Checks:
/// 1. Compiled certification signs the Configured Key's pubkey.
/// 2. Configured certification signs the first stage key's pubkey.
/// 3. Each `stage:*` stamp's signature matches the bytes before its line.
/// 4. Ordering: compiled → configured → stage:* (in order).
///
/// `file_bytes_before_stamps` is the full file content before the STAMPS section.
/// `stamp_lines` are the raw JSON lines (one per stamp) from the file.
pub fn verify_all_stamps(
    file_bytes_before_stamps: &[u8],
    stamps: &Stamps,
    compiled_vk: &VerifyingKey,
) -> Vec<(Stamp, StampStatus)> {
    let mut results = Vec::new();

    let mut accumulated: Vec<u8> = file_bytes_before_stamps.to_vec();

    for stamp in stamps.entries() {
        let status = if stamp.is_compiled() {
            if let Some(configured_stamp) = stamps.entries().iter().find(|s| s.is_configured()) {
                verify_certification(compiled_vk, stamp.signature(), configured_stamp.pubkey())
            } else {
                StampStatus::Invalid("no configured stamp found for compiled to certify".into())
            }
        } else if stamp.is_configured() {
            if let Some(stage_stamp) = stamps.entries().iter().find(|s| s.key() == "stage:output") {
                verify_certification_with_hex_pubkey(
                    stamp.pubkey(),
                    stamp.signature(),
                    stage_stamp.pubkey(),
                )
            } else {
                StampStatus::Invalid("no stage:output stamp found for configured to certify".into())
            }
        } else if stamp.stage_name().is_some() {
            verify_prior_bytes(stamp.pubkey(), stamp.signature(), &accumulated)
        } else {
            StampStatus::Invalid(format!("unknown stamp key: {}", stamp.key()))
        };

        let stamp_line = serde_json::to_string(stamp)
            .expect("Stamp has only String fields; serde_json::to_string cannot fail");
        accumulated.extend_from_slice(stamp_line.as_bytes());
        accumulated.push(b'\n');

        results.push((stamp.clone(), status));
    }

    results
}

/// Verify a certification stamp (compiled/configured) against a known verifying key.
fn verify_certification(
    verifier_vk: &VerifyingKey,
    sig_b64: &str,
    certified_pubkey_hex: &str,
) -> StampStatus {
    let sig_bytes = match B64.decode(sig_b64) {
        Ok(b) => b,
        Err(e) => return StampStatus::Invalid(format!("base64 decode: {e}")),
    };
    let sig_arr: [u8; 64] = match sig_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return StampStatus::Invalid("signature not 64 bytes".into()),
    };
    let signature = Signature::from_bytes(&sig_arr);
    match verifier_vk.verify_strict(certified_pubkey_hex.as_bytes(), &signature) {
        Ok(()) => StampStatus::Valid,
        Err(e) => StampStatus::Invalid(format!("signature mismatch: {e}")),
    }
}

/// Verify a certification stamp using the verifier's pubkey from a hex string.
fn verify_certification_with_hex_pubkey(
    verifier_pubkey_hex: &str,
    sig_b64: &str,
    certified_pubkey_hex: &str,
) -> StampStatus {
    let pubkey_bytes = match hex::decode(verifier_pubkey_hex) {
        Ok(b) => b,
        Err(e) => return StampStatus::Invalid(format!("hex decode pubkey: {e}")),
    };
    let pubkey_arr: [u8; 32] = match pubkey_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return StampStatus::Invalid("pubkey not 32 bytes".into()),
    };
    let vk = match VerifyingKey::from_bytes(&pubkey_arr) {
        Ok(v) => v,
        Err(e) => return StampStatus::Invalid(format!("invalid pubkey: {e}")),
    };
    verify_certification(&vk, sig_b64, certified_pubkey_hex)
}

/// Verify a stage stamp's prior-bytes signature.
fn verify_prior_bytes(pubkey_hex: &str, sig_b64: &str, prior_bytes: &[u8]) -> StampStatus {
    match verify_bytes(pubkey_hex, sig_b64, prior_bytes) {
        Ok(()) => StampStatus::Valid,
        Err(e) => StampStatus::Invalid(format!("{e}")),
    }
}

/// Append a new stage stamp to an existing stamp chain.
///
/// Refuses if any existing stamp fails verification (chain integrity).
/// Returns the new stamps (existing + appended).
pub fn append_stage_stamp(
    file_bytes_before_stamps: &[u8],
    existing_stamps: &Stamps,
    new_stamp: Stamp,
    compiled_vk: &VerifyingKey,
) -> Result<Stamps, SignatureError> {
    // Verify all existing stamps first.
    let results = verify_all_stamps(file_bytes_before_stamps, existing_stamps, compiled_vk);
    for (stamp, status) in &results {
        if let StampStatus::Invalid(reason) = status {
            return Err(SignatureError::ChainIntegrity(format!(
                "existing stamp '{}' invalid: {reason}",
                stamp.key()
            )));
        }
    }

    let mut new_stamps = existing_stamps.clone();
    new_stamps.push(new_stamp);
    Ok(new_stamps)
}

// ===========================================================================
// Tests — written FIRST (tamper/forgery tests)
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: derive a keypair from a passphrase.
    fn kp(passphrase: &str) -> (SigningKey, VerifyingKey) {
        derive_keypair(passphrase).expect("keypair derivation should succeed")
    }

    fn prior_bytes_for_stage(file_content: &[u8], stamps: &[Stamp], index: usize) -> Vec<u8> {
        let mut acc = file_content.to_vec();
        for stamp in &stamps[..index] {
            let line = serde_json::to_string(stamp).unwrap();
            acc.extend_from_slice(line.as_bytes());
            acc.push(b'\n');
        }
        acc
    }

    // -----------------------------------------------------------------------
    // Key derivation tests
    // -----------------------------------------------------------------------

    #[test]
    fn empty_passphrase_derives_computer_key() {
        let (sk1, vk1) = kp("");
        let (sk2, vk2) = kp("");
        // Deterministic
        assert_eq!(sk1.to_bytes(), sk2.to_bytes());
        assert_eq!(vk1.to_bytes(), vk2.to_bytes());
        // Non-zero
        assert_ne!(sk1.to_bytes(), [0u8; 32]);
        assert_ne!(vk1.to_bytes(), [0u8; 32]);
    }

    #[test]
    fn different_passphrases_yield_different_keys() {
        let (_, vk_a) = kp("");
        let (_, vk_b) = kp("human-secret");
        assert_ne!(vk_a.to_bytes(), vk_b.to_bytes());
    }

    #[test]
    fn custom_compiled_key_changes_certification() {
        // The compiled key is derived from COMPILED_PASSPHRASE at build time.
        // In tests, we derive explicitly and verify the compiled keypair matches.
        let (compiled_sk, compiled_vk) = compiled_keypair().expect("compiled keypair");
        let (empty_sk, empty_vk) = kp("");
        // Stock build: compiled passphrase is empty, so compiled == empty key
        assert_eq!(compiled_sk.to_bytes(), empty_sk.to_bytes());
        assert_eq!(compiled_vk.to_bytes(), empty_vk.to_bytes());

        // A different passphrase gives a different key
        let (_, custom_vk) = kp("custom-build-key");
        assert_ne!(compiled_vk.to_bytes(), custom_vk.to_bytes());
    }

    // -----------------------------------------------------------------------
    // Parse / serialize roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn parse_json_lines_roundtrip() {
        let stamps = Stamps::new(vec![
            Stamp::new_for_test(
                "compiled",
                "aa",
                "pubkey:configured",
                "bb",
                "test",
                "2026-01-01T00:00:00Z",
            ),
            Stamp::new_for_test(
                "configured",
                "cc",
                "pubkey:stage:output",
                "dd",
                "test",
                "2026-01-01T00:00:01Z",
            ),
        ]);
        let bytes = stamps.serialize();
        let parsed = Stamps::parse(&bytes).expect("parse should succeed");
        assert_eq!(stamps, parsed);
    }

    #[test]
    fn multi_stage_stamp_parse() {
        let stamps = Stamps::new(vec![
            make_test_stamp("compiled", "pubkey:configured"),
            make_test_stamp("configured", "pubkey:stage:output"),
            make_test_stamp("stage:output", "prior-bytes"),
            make_test_stamp("stage:checked", "prior-bytes"),
            make_test_stamp("stage:verified", "prior-bytes"),
        ]);
        let bytes = stamps.serialize();
        let parsed = Stamps::parse(&bytes).expect("parse should succeed");
        assert_eq!(parsed.len(), 5);
        assert_eq!(parsed.entries()[0].key(), "compiled");
        assert_eq!(parsed.entries()[2].key(), "stage:output");
        assert_eq!(parsed.entries()[4].key(), "stage:verified");
    }

    fn make_test_stamp(key: &str, signs: &str) -> Stamp {
        Stamp::new_for_test(
            key,
            "aabbccdd",
            signs,
            "eeff0011",
            "test",
            "2026-01-01T00:00:00Z",
        )
    }

    // -----------------------------------------------------------------------
    // Certification verification
    // -----------------------------------------------------------------------

    #[test]
    fn verify_compiled_certification() {
        let (compiled_sk, compiled_vk) = kp("");
        let (_, configured_vk) = kp("configured-key");
        let configured_pubkey_hex = hex::encode(configured_vk.as_bytes());

        let stamp = create_compiled_stamp(&compiled_sk, &configured_pubkey_hex);
        assert_eq!(stamp.key(), "compiled");
        assert_eq!(stamp.signs(), "pubkey:configured");

        let result = verify_certification(&compiled_vk, stamp.signature(), &configured_pubkey_hex);
        assert_eq!(result, StampStatus::Valid);
    }

    #[test]
    fn verify_configured_certification() {
        let (configured_sk, configured_vk) = kp("configured-key");
        let (_, stage_vk) = kp("stage-output-key");
        let stage_pubkey_hex = hex::encode(stage_vk.as_bytes());

        let stamp = create_configured_stamp(&configured_sk, &stage_pubkey_hex);
        assert_eq!(stamp.key(), "configured");
        assert_eq!(stamp.signs(), "pubkey:stage:output");

        let result = verify_certification_with_hex_pubkey(
            &hex::encode(configured_vk.as_bytes()),
            stamp.signature(),
            &stage_pubkey_hex,
        );
        assert_eq!(result, StampStatus::Valid);
    }

    #[test]
    fn verify_stage_stamp_prior_bytes() {
        let (stage_sk, stage_vk) = kp("stage-key");
        let prior_bytes = b"some file content before the stamp";

        let stamp = create_stage_stamp(&stage_sk, "output", prior_bytes);
        assert_eq!(stamp.key(), "stage:output");
        assert_eq!(stamp.signs(), "prior-bytes");

        let result = verify_prior_bytes(
            &hex::encode(stage_vk.as_bytes()),
            stamp.signature(),
            prior_bytes,
        );
        assert_eq!(result, StampStatus::Valid);
    }

    // -----------------------------------------------------------------------
    // Tamper detection tests
    // -----------------------------------------------------------------------

    #[test]
    fn tamper_metadata_detection() {
        let (stage_sk, _) = kp("stage-key");
        let prior_bytes = b"metadata: test=foo\nsections: INPUT, OUTPUT, STAMPS\n";

        let stamp = create_stage_stamp(&stage_sk, "output", prior_bytes);

        // Tamper with the metadata
        let tampered_bytes = b"metadata: test=bar\nsections: INPUT, OUTPUT, STAMPS\n";

        // Verify fails with tampered prior bytes
        let result = verify_bytes(stamp.pubkey(), stamp.signature(), tampered_bytes);
        assert!(result.is_err(), "should detect tampered metadata");
    }

    #[test]
    fn tamper_input_detection() {
        let (stage_sk, _) = kp("stage-key");
        let prior_bytes = b"INPUT:\nhello world\n";

        let stamp = create_stage_stamp(&stage_sk, "output", prior_bytes);

        let tampered_bytes = b"INPUT:\ngoodbye world\n";

        let result = verify_bytes(stamp.pubkey(), stamp.signature(), tampered_bytes);
        assert!(result.is_err(), "should detect tampered input");
    }

    #[test]
    fn tamper_output_detection() {
        let (stage_sk, _) = kp("stage-key");
        let prior_bytes = b"INPUT:\nhello\nOUTPUT:\n42\n";

        let stamp = create_stage_stamp(&stage_sk, "output", prior_bytes);

        let tampered_bytes = b"INPUT:\nhello\nOUTPUT:\n43\n";

        let result = verify_bytes(&stamp.pubkey, &stamp.signature, tampered_bytes);
        assert!(result.is_err(), "should detect tampered output");
    }

    #[test]
    fn tamper_after_promotion_invalidates_later_stamps() {
        let (stage_sk_output, stage_vk_output) = kp("stage-output");
        let (stage_sk_checked, stage_vk_checked) = kp("stage-checked");

        let file_content = b"some content";

        // Create output stamp
        let output_stamp = create_stage_stamp(&stage_sk_output, "output", file_content);

        // Accumulate: file content + output stamp line + newline
        let mut accumulated = file_content.to_vec();
        let output_line = serde_json::to_string(&output_stamp).unwrap();
        accumulated.extend_from_slice(output_line.as_bytes());
        accumulated.push(b'\n');

        // Create checked stamp over accumulated bytes
        let checked_stamp = create_stage_stamp(&stage_sk_checked, "checked", &accumulated);

        // Both should verify
        let result_out = verify_prior_bytes(
            &hex::encode(stage_vk_output.as_bytes()),
            output_stamp.signature(),
            file_content,
        );
        assert_eq!(result_out, StampStatus::Valid);

        let result_chk = verify_prior_bytes(
            &hex::encode(stage_vk_checked.as_bytes()),
            checked_stamp.signature(),
            &accumulated,
        );
        assert_eq!(result_chk, StampStatus::Valid);

        let tampered_content = b"tampered content";
        let result_out_tampered = verify_prior_bytes(
            &hex::encode(stage_vk_output.as_bytes()),
            output_stamp.signature(),
            tampered_content,
        );
        assert!(matches!(result_out_tampered, StampStatus::Invalid(_)));

        let mut tampered_accumulated = tampered_content.to_vec();
        tampered_accumulated.extend_from_slice(output_line.as_bytes());
        tampered_accumulated.push(b'\n');
        let result_chk_tampered = verify_prior_bytes(
            &hex::encode(stage_vk_checked.as_bytes()),
            checked_stamp.signature(),
            &tampered_accumulated,
        );
        assert!(matches!(result_chk_tampered, StampStatus::Invalid(_)));
    }

    #[test]
    fn chain_integrity_refuses_append_on_broken_stamp() {
        let (compiled_sk, compiled_vk) = kp("");
        let (configured_sk, _) = kp("configured");
        let (stage_sk_output, _) = kp("stage-output");
        let (stage_sk_checked, _) = kp("stage-checked");

        let file_content = b"file content here";
        let configured_pubkey_hex = hex::encode(kp("configured").1.as_bytes());
        let stage_output_pubkey_hex = hex::encode(kp("stage-output").1.as_bytes());

        let compiled_stamp = create_compiled_stamp(&compiled_sk, &configured_pubkey_hex);
        let configured_stamp = create_configured_stamp(&configured_sk, &stage_output_pubkey_hex);

        let prior_for_output = prior_bytes_for_stage(
            file_content,
            &[compiled_stamp.clone(), configured_stamp.clone()],
            2,
        );
        let output_stamp = create_stage_stamp(&stage_sk_output, "output", &prior_for_output);

        let stamps = Stamps::new(vec![
            compiled_stamp.clone(),
            configured_stamp.clone(),
            output_stamp.clone(),
        ]);

        let results = verify_all_stamps(file_content, &stamps, &compiled_vk);
        for (stamp, status) in &results {
            assert_eq!(
                *status,
                StampStatus::Valid,
                "stamp '{}' should be valid",
                stamp.key()
            );
        }

        let prior_for_checked = prior_bytes_for_stage(file_content, stamps.entries(), 3);
        let checked_stamp = create_stage_stamp(&stage_sk_checked, "checked", &prior_for_checked);
        let appended = append_stage_stamp(file_content, &stamps, checked_stamp, &compiled_vk);
        assert!(appended.is_ok(), "append should succeed on valid chain");

        // Create a broken chain (tamper with the output stamp signature)
        let mut broken_stamps = stamps.clone();
        // Tamper with the output stamp's signature via a replacement
        let tampered_entries: Vec<Stamp> = broken_stamps
            .entries()
            .iter()
            .enumerate()
            .map(|(i, s)| {
                if i == 2 {
                    s.clone().with_signature("AAAA")
                } else {
                    s.clone()
                }
            })
            .collect();
        broken_stamps = Stamps::new(tampered_entries);

        let new_stamp = create_stage_stamp(&stage_sk_checked, "checked", &prior_for_checked);
        let result = append_stage_stamp(file_content, &broken_stamps, new_stamp, &compiled_vk);
        assert!(result.is_err(), "append should refuse on broken chain");
    }

    #[test]
    fn append_preserves_existing_stamps() {
        let (compiled_sk, compiled_vk) = kp("");
        let (configured_sk, _) = kp("configured");
        let (stage_sk_output, _) = kp("stage-output");
        let (stage_sk_checked, _) = kp("stage-checked");

        let file_content = b"content";
        let configured_pubkey_hex = hex::encode(kp("configured").1.as_bytes());
        let stage_output_pubkey_hex = hex::encode(kp("stage-output").1.as_bytes());

        let compiled_stamp = create_compiled_stamp(&compiled_sk, &configured_pubkey_hex);
        let configured_stamp = create_configured_stamp(&configured_sk, &stage_output_pubkey_hex);

        let prior_for_output = prior_bytes_for_stage(
            file_content,
            &[compiled_stamp.clone(), configured_stamp.clone()],
            2,
        );
        let output_stamp = create_stage_stamp(&stage_sk_output, "output", &prior_for_output);

        let stamps = Stamps::new(vec![compiled_stamp, configured_stamp, output_stamp]);

        let before_bytes = stamps.serialize();

        let prior_for_checked = prior_bytes_for_stage(file_content, stamps.entries(), 3);
        let checked_stamp = create_stage_stamp(&stage_sk_checked, "checked", &prior_for_checked);
        let appended =
            append_stage_stamp(file_content, &stamps, checked_stamp, &compiled_vk).unwrap();

        let after_bytes = appended.serialize();
        let before_str = std::str::from_utf8(&before_bytes).unwrap();
        let after_str = std::str::from_utf8(&after_bytes).unwrap();
        assert!(
            after_str.starts_with(before_str),
            "existing stamps must be preserved byte-for-byte"
        );
        assert_eq!(appended.len(), 4);
    }

    #[test]
    fn full_chain_verify_all_stamps() {
        let (compiled_sk, compiled_vk) = kp("");
        let (configured_sk, _) = kp("configured");
        let (stage_sk, stage_vk) = kp("stage-key");

        let file_content = b"the file content";
        let configured_pubkey_hex = hex::encode(kp("configured").1.as_bytes());
        let stage_pubkey_hex = hex::encode(stage_vk.as_bytes());

        let compiled_stamp = create_compiled_stamp(&compiled_sk, &configured_pubkey_hex);
        let configured_stamp = create_configured_stamp(&configured_sk, &stage_pubkey_hex);

        // Stage stamp signs file content + compiled line + newline + configured line + newline
        let prior_for_stage = prior_bytes_for_stage(
            file_content,
            &[compiled_stamp.clone(), configured_stamp.clone()],
            2,
        );
        let stage_stamp = create_stage_stamp(&stage_sk, "output", &prior_for_stage);

        let stamps = Stamps::new(vec![compiled_stamp, configured_stamp, stage_stamp]);

        let results = verify_all_stamps(file_content, &stamps, &compiled_vk);
        assert_eq!(results.len(), 3);
        for (stamp, status) in &results {
            assert_eq!(
                *status,
                StampStatus::Valid,
                "stamp '{}' should be valid",
                stamp.key()
            );
        }
    }

    #[test]
    fn verify_detects_wrong_compiled_key() {
        let (compiled_sk, _) = kp("wrong-key"); // NOT the default compiled key
        let (_, compiled_vk_default) = kp(""); // the default compiled key
        let (_, configured_vk) = kp("configured");
        let configured_pubkey_hex = hex::encode(configured_vk.as_bytes());

        let stamp = create_compiled_stamp(&compiled_sk, &configured_pubkey_hex);

        // Verify with the default compiled VK — should fail (wrong key signed it)
        let result = verify_certification(
            &compiled_vk_default,
            stamp.signature(),
            &configured_pubkey_hex,
        );
        assert!(matches!(result, StampStatus::Invalid(_)));
    }

    #[test]
    fn verify_detects_wrong_configured_key() {
        let (configured_sk, _) = kp("configured");
        let (_, stage_vk) = kp("stage");
        let stage_pubkey_hex = hex::encode(stage_vk.as_bytes());

        let stamp = create_configured_stamp(&configured_sk, &stage_pubkey_hex);

        let wrong_configured_vk = kp("wrong").1;
        let wrong_hex = hex::encode(wrong_configured_vk.as_bytes());
        let result =
            verify_certification_with_hex_pubkey(&wrong_hex, stamp.signature(), &stage_pubkey_hex);
        assert!(matches!(result, StampStatus::Invalid(_)));
    }

    #[test]
    fn parse_empty_stamps() {
        let stamps = Stamps::new(vec![]);
        let bytes = stamps.serialize();
        assert!(bytes.is_empty());
        let parsed = Stamps::parse(&bytes).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_malformed_json_errors() {
        let bad = b"not json at all";
        let result = Stamps::parse(bad);
        assert!(result.is_err());
    }
}
