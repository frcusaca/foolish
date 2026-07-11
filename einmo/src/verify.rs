//! Verify-on-inspect: pure verification over parsed data.
//!
//! This module contains **no filesystem I/O, no tty access, and no argon2
//! dependency** — it operates purely on in-memory parsed types.  This keeps
//! it compilable to WASM (Proposal C) and cleanly separated from side-effecting
//! code.

use std::path::PathBuf;

use ed25519_dalek::VerifyingKey;

use crate::config::{Stage, TestConfig};
use crate::signature::{StampStatus, Stamps};

// ---------------------------------------------------------------------------
// StampVerification
// ---------------------------------------------------------------------------

/// Result of verifying a single stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StampVerification {
    /// Index of the stamp in the chain (0-based).
    pub stamp_index: usize,
    /// Whether the stamp is valid.
    pub valid: bool,
    /// Error message if invalid; `None` if valid.
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// verify_all_stamps (pure)
// ---------------------------------------------------------------------------

/// Verify every stamp in a stamp chain against the given file bytes.
///
/// `file_bytes` is the content of the file **before** the STAMPS section
/// (i.e. header + metadata + sections).  `stamps` are the parsed stamps.
/// `compiled_vk` is the compiled verifying key (the caller provides it;
/// this function performs no key derivation).
///
/// Checks:
/// 1. The `compiled` stamp certifies the configured pubkey.
/// 2. The `configured` stamp certifies the first stage key's pubkey.
/// 3. Each `stage:*` stamp's signature matches all accumulated bytes before it.
pub fn verify_all_stamps(
    file_bytes: &[u8],
    stamps: &Stamps,
    compiled_vk: &VerifyingKey,
) -> Vec<StampVerification> {
    let results = crate::signature::verify_all_stamps(file_bytes, stamps, compiled_vk);
    results
        .into_iter()
        .enumerate()
        .map(|(i, (_stamp, status))| StampVerification {
            stamp_index: i,
            valid: matches!(status, StampStatus::Valid),
            error: match status {
                StampStatus::Valid => None,
                StampStatus::Invalid(msg) => Some(msg),
            },
        })
        .collect()
}

/// Returns `true` if every stamp in the result is valid.
pub fn all_valid(results: &[StampVerification]) -> bool {
    results.iter().all(|r| r.valid)
}

// ---------------------------------------------------------------------------
// VerifyReport
// ---------------------------------------------------------------------------

/// Per-file verification result.
#[derive(Debug)]
pub struct FileVerification {
    /// Path to the file (relative to the stage directory).
    pub path: PathBuf,
    /// Per-stamp verification results.
    pub stamps: Vec<StampVerification>,
    /// Whether all stamps in this file are valid.
    pub valid: bool,
}

/// Aggregate verification report across multiple files.
#[derive(Debug)]
pub struct VerifyReport {
    /// Per-file results.
    pub files: Vec<FileVerification>,
}

impl VerifyReport {
    /// Returns `true` if all files verified successfully.
    pub fn all_valid(&self) -> bool {
        self.files.iter().all(|f| f.valid)
    }

    /// Number of files that failed verification.
    pub fn invalid_count(&self) -> usize {
        self.files.iter().filter(|f| !f.valid).count()
    }

    /// Number of files that passed verification.
    pub fn valid_count(&self) -> usize {
        self.files.iter().filter(|f| f.valid).count()
    }

    /// Total number of files.
    pub fn total_count(&self) -> usize {
        self.files.len()
    }
}

// ---------------------------------------------------------------------------
// verify (stub — depends on Phase 3: EinmoFile::from_file)
// ---------------------------------------------------------------------------

/// Walk a stage (or all stages) and verify every `.einmo` file.
///
/// **Stub**: `EinmoFile::from_file` (Phase 3) is required for the full
/// implementation.  Currently returns an empty report.
pub fn verify(_config: &TestConfig, _stage: Option<Stage>) -> VerifyReport {
    VerifyReport { files: Vec::new() }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::{
        Stamp, Stamps, create_compiled_stamp, create_configured_stamp, create_stage_stamp,
        derive_keypair,
    };

    /// Helper: derive a keypair from a passphrase.
    fn kp(passphrase: &str) -> (ed25519_dalek::SigningKey, VerifyingKey) {
        derive_keypair(passphrase).expect("keypair derivation")
    }

    /// Helper: build a full stamp chain (compiled + configured + stage stamps)
    /// and return (file_bytes, stamps, compiled_vk).
    fn build_chain(
        file_content: &[u8],
        stage_names: &[&str],
        key_passphrases: &[&str],
    ) -> (Vec<u8>, Stamps, VerifyingKey) {
        let (compiled_sk, compiled_vk) = kp("");
        let (configured_sk, _) = kp("configured");
        let configured_pubkey_hex = hex::encode(kp("configured").1.as_bytes());

        // For simplicity, use the same stage key for all stages in tests.
        let stage_pass = key_passphrases.first().copied().unwrap_or("stage");
        let (_, stage_vk) = kp(stage_pass);
        let stage_pubkey_hex = hex::encode(stage_vk.as_bytes());

        let compiled_stamp = create_compiled_stamp(&compiled_sk, &configured_pubkey_hex);
        let configured_stamp = create_configured_stamp(&configured_sk, &stage_pubkey_hex);

        let mut stamps_vec = vec![compiled_stamp, configured_stamp];
        let mut accumulated = file_content.to_vec();

        // Accumulate compiled + configured lines
        for stamp in &stamps_vec {
            let line = serde_json::to_string(stamp).unwrap();
            accumulated.extend_from_slice(line.as_bytes());
            accumulated.push(b'\n');
        }

        for name in stage_names {
            let (stage_sk, _) = kp(stage_pass);
            let stamp = create_stage_stamp(&stage_sk, name, &accumulated);
            let line = serde_json::to_string(&stamp).unwrap();
            stamps_vec.push(stamp);
            accumulated.extend_from_slice(line.as_bytes());
            accumulated.push(b'\n');
        }

        let stamps = Stamps::new(stamps_vec);
        (file_content.to_vec(), stamps, compiled_vk)
    }

    // -- Valid file verifies ------------------------------------------------

    #[test]
    fn valid_file_verifies() {
        let content = b"INPUT:\nhello\nOUTPUT:\n42\n";
        let (file_bytes, stamps, compiled_vk) = build_chain(content, &["output"], &["stage"]);
        let results = verify_all_stamps(&file_bytes, &stamps, &compiled_vk);
        assert!(all_valid(&results), "all stamps should be valid");
        assert_eq!(results.len(), 3); // compiled + configured + stage:output
        for r in &results {
            assert!(r.valid, "stamp {} should be valid", r.stamp_index);
            assert!(r.error.is_none());
        }
    }

    // -- Tampered input fails -----------------------------------------------

    #[test]
    fn tampered_input_fails() {
        let content = b"INPUT:\nhello\nOUTPUT:\n42\n";
        let (_file_bytes, stamps, compiled_vk) = build_chain(content, &["output"], &["stage"]);

        // Tamper with the file bytes (change INPUT)
        let tampered = b"INPUT:\ngoodbye\nOUTPUT:\n42\n";
        let results = verify_all_stamps(tampered, &stamps, &compiled_vk);

        // The stage:output stamp should fail (its prior bytes changed).
        let stage_result = results.iter().find(|r| r.stamp_index == 2).unwrap();
        assert!(
            !stage_result.valid,
            "tampered input should fail stage stamp"
        );
        assert!(stage_result.error.is_some());
    }

    // -- Tampered output fails ----------------------------------------------

    #[test]
    fn tampered_output_fails() {
        let content = b"INPUT:\nhello\nOUTPUT:\n42\n";
        let (_file_bytes, stamps, compiled_vk) = build_chain(content, &["output"], &["stage"]);

        // Tamper with output
        let tampered = b"INPUT:\nhello\nOUTPUT:\n99\n";
        let results = verify_all_stamps(tampered, &stamps, &compiled_vk);

        let stage_result = results.iter().find(|r| r.stamp_index == 2).unwrap();
        assert!(
            !stage_result.valid,
            "tampered output should fail stage stamp"
        );
    }

    // -- Tampered metadata fails --------------------------------------------

    #[test]
    fn tampered_metadata_fails() {
        let content = b"metadata: test=foo\nINPUT:\nhello\n";
        let (_file_bytes, stamps, compiled_vk) = build_chain(content, &["output"], &["stage"]);

        let tampered = b"metadata: test=bar\nINPUT:\nhello\n";
        let results = verify_all_stamps(tampered, &stamps, &compiled_vk);

        let stage_result = results.iter().find(|r| r.stamp_index == 2).unwrap();
        assert!(
            !stage_result.valid,
            "tampered metadata should fail stage stamp"
        );
    }

    // -- Broken stage-stamp chain fails -------------------------------------

    #[test]
    fn broken_stage_stamp_chain_fails() {
        let content = b"content";
        let (file_bytes, stamps, compiled_vk) = build_chain(content, &["output"], &["stage"]);

        // Tamper with the stamp itself (change its signature)
        let tampered_entries: Vec<Stamp> = stamps
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
        let tampered_stamps = Stamps::new(tampered_entries);

        let results = verify_all_stamps(&file_bytes, &tampered_stamps, &compiled_vk);
        let stage_result = results.iter().find(|r| r.stamp_index == 2).unwrap();
        assert!(
            !stage_result.valid,
            "broken stamp signature should fail verification"
        );
    }

    // -- Multi-stage-stamp chain validates ----------------------------------

    #[test]
    fn multi_stage_stamp_chain_validates() {
        let content = b"the file content\n";
        let (file_bytes, stamps, compiled_vk) =
            build_chain(content, &["output", "checked", "verified"], &["stage"]);
        let results = verify_all_stamps(&file_bytes, &stamps, &compiled_vk);

        assert_eq!(results.len(), 5); // compiled + configured + 3 stage stamps
        assert!(all_valid(&results), "multi-stage chain should all be valid");
        for r in &results {
            assert!(r.valid, "stamp {} should be valid", r.stamp_index);
        }
    }

    // -- Tampered after promotion invalidates later stamps ------------------

    #[test]
    fn tampered_after_promotion_invalidates_later_stamps() {
        let content = b"file content\n";
        let (file_bytes, stamps, compiled_vk) =
            build_chain(content, &["output", "checked"], &["stage"]);

        // All valid initially
        let results = verify_all_stamps(&file_bytes, &stamps, &compiled_vk);
        assert!(all_valid(&results));

        // Tamper with file content — stage:output fails, stage:checked also fails
        let tampered = b"tampered content\n";
        let results = verify_all_stamps(tampered, &stamps, &compiled_vk);

        // stage:output (index 2) should fail
        let out = results.iter().find(|r| r.stamp_index == 2).unwrap();
        assert!(!out.valid);

        // stage:checked (index 3) should also fail (accumulated bytes changed)
        let chk = results.iter().find(|r| r.stamp_index == 3).unwrap();
        assert!(!chk.valid);
    }

    // -- Certification verification -----------------------------------------

    #[test]
    fn compiled_certification_valid() {
        let (compiled_sk, compiled_vk) = kp("");
        let configured_pubkey_hex = hex::encode(kp("configured").1.as_bytes());
        let stage_pubkey_hex = hex::encode(kp("stage").1.as_bytes());

        let compiled_stamp = create_compiled_stamp(&compiled_sk, &configured_pubkey_hex);
        let configured_stamp = create_configured_stamp(&kp("configured").0, &stage_pubkey_hex);

        let file_bytes = b"content";
        let mut acc = file_bytes.to_vec();
        for s in [&compiled_stamp, &configured_stamp] {
            let line = serde_json::to_string(s).unwrap();
            acc.extend_from_slice(line.as_bytes());
            acc.push(b'\n');
        }

        let stage_stamp = create_stage_stamp(&kp("stage").0, "output", &acc);

        let stamps = Stamps::new(vec![compiled_stamp, configured_stamp, stage_stamp]);

        let results = verify_all_stamps(file_bytes, &stamps, &compiled_vk);
        assert!(all_valid(&results));
    }

    #[test]
    fn wrong_compiled_key_fails_certification() {
        // Use a different compiled key than the default
        let (wrong_sk, _) = kp("wrong-compiled-key");
        let (_, default_vk) = kp(""); // the default compiled key
        let configured_pubkey_hex = hex::encode(kp("configured").1.as_bytes());

        let stamp = create_compiled_stamp(&wrong_sk, &configured_pubkey_hex);
        let configured_stamp =
            create_configured_stamp(&kp("configured").0, &hex::encode(kp("stage").1.as_bytes()));
        let stage_stamp = create_stage_stamp(&kp("stage").0, "output", b"content");

        let stamps = Stamps::new(vec![stamp, configured_stamp, stage_stamp]);

        // Verify with the default compiled VK — compiled stamp should fail
        let results = verify_all_stamps(b"content", &stamps, &default_vk);
        let compiled_result = &results[0];
        assert!(
            !compiled_result.valid,
            "wrong compiled key should fail certification"
        );
    }

    // -- all_valid helper ---------------------------------------------------

    #[test]
    fn all_valid_empty() {
        assert!(all_valid(&[]));
    }

    #[test]
    fn all_valid_single_valid() {
        let r = StampVerification {
            stamp_index: 0,
            valid: true,
            error: None,
        };
        assert!(all_valid(&[r]));
    }

    #[test]
    fn all_valid_single_invalid() {
        let r = StampVerification {
            stamp_index: 0,
            valid: false,
            error: Some("bad".into()),
        };
        assert!(!all_valid(&[r]));
    }

    // -- VerifyReport -------------------------------------------------------

    #[test]
    fn verify_report_empty() {
        let report = VerifyReport { files: vec![] };
        assert!(report.all_valid());
        assert_eq!(report.invalid_count(), 0);
        assert_eq!(report.valid_count(), 0);
        assert_eq!(report.total_count(), 0);
    }

    #[test]
    fn verify_report_mixed() {
        let report = VerifyReport {
            files: vec![
                FileVerification {
                    path: PathBuf::from("a.einmo"),
                    stamps: vec![],
                    valid: true,
                },
                FileVerification {
                    path: PathBuf::from("b.einmo"),
                    stamps: vec![],
                    valid: false,
                },
            ],
        };
        assert!(!report.all_valid());
        assert_eq!(report.invalid_count(), 1);
        assert_eq!(report.valid_count(), 1);
        assert_eq!(report.total_count(), 2);
    }

    // -- verify stub --------------------------------------------------------

    #[test]
    fn verify_stub_returns_empty_report() {
        let cfg = TestConfig::new("/nonexistent");
        let report = verify(&cfg, Some(Stage::Output));
        assert!(report.all_valid());
        assert_eq!(report.total_count(), 0);
    }
}
