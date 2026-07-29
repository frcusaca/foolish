//! Verify-on-inspect: the pure verification path plus the `verify` walk.
//!
//! The core verification functions ([`verify_bytes`], [`verify_all`]) touch
//! **no filesystem, no tty, and no Argon2** — they only re-check Ed25519
//! signatures already present in a parsed [`EinmoFile`]. That keeps them
//! WASM-targetable (FOOP-92 Proposal C). The single fs-touching convenience,
//! [`EinmoFile::from_file`], is a thin `fs::read` + [`verify_bytes`] wrapper and
//! is the *only* item here that reads a file.

use std::path::{Path, PathBuf};

use crate::config::TestConfig;
use crate::error::{EinmoError, Result};
use crate::format::EinmoFile;
use crate::signature::StampCheck;
use crate::stage::{Stage, walk_input_tree};
use crate::transitions::normalize_file_path;

// ---------------------------------------------------------------------------
// Pure verification (no fs / tty / argon2 — WASM-targetable)
// ---------------------------------------------------------------------------

/// Per-stamp verification verdict for an [`EinmoFile`].
pub type StampVerification = StampCheck;

/// Verify every stamp of an already-parsed file against its own signed prefix.
#[must_use]
pub fn verify_all(file: &EinmoFile) -> Vec<StampVerification> {
    file.verify_all()
}

/// Parse `bytes` and verify all stamps; return the file only if the chain is
/// fully valid (verify-on-inspect).
///
/// # Errors
///
/// Returns [`EinmoError::Parse`] on a malformed envelope, or
/// [`EinmoError::Verification`] if any stamp fails.
pub fn verify_bytes(bytes: &[u8]) -> Result<EinmoFile> {
    let file = EinmoFile::parse(bytes)?;
    // Verify against the ACTUAL file bytes up to the STAMPS section, not the
    // recomputed canonical form. This detects metadata whitespace tampering
    // that `parse()` would normalize away (e.g. extra spaces after a `key:`).
    let raw_prefix = raw_signed_prefix(bytes, &file)?;
    let checks = file.stamps().verify_chain(raw_prefix);
    let failed: Vec<&str> = checks
        .iter()
        .filter(|c| !c.ok)
        .map(|c| c.key.as_str())
        .collect();
    if failed.is_empty() {
        Ok(file)
    } else {
        Err(EinmoError::Verification(format!(
            "stamp(s) failed: {}",
            failed.join(", ")
        )))
    }
}

/// Extract the raw bytes from the start of the file up to (not including) the
/// STAMPS section's content — i.e. everything up to and including the separator
/// that introduces the STAMPS section. This is the byte range that stage stamps
/// sign as "prior bytes".
///
/// The file layout is:
/// ```text
/// header_line\n
/// <metadata>
/// SEP <body1> SEP <body2> SEP ... SEP <stamps>
/// ```
/// The signed prefix includes the header, metadata, every body, and the final
/// separator that introduces STAMPS. So we must find the `(num_bodies + 1)`-th
/// occurrence of `separator` after the header line.
fn raw_signed_prefix<'a>(bytes: &'a [u8], file: &EinmoFile) -> Result<&'a [u8]> {
    let separator = file.separator().as_bytes();
    let num_bodies = file.sections().len();
    // Skip the header line (everything up to and including the first LF).
    let header_end = bytes
        .iter()
        .position(|&b| b == b'\n')
        .ok_or_else(|| EinmoError::Parse("missing header newline".into()))?;
    let mut search_from = header_end + 1;
    let mut prefix_end = 0usize;
    for i in 0..(num_bodies + 1) {
        match find_subsequence(&bytes[search_from..], separator) {
            Some(pos) => {
                prefix_end = search_from + pos + separator.len();
                search_from = prefix_end;
            }
            None => {
                return Err(EinmoError::Parse(format!(
                    "could not find separator #{} for raw signed prefix",
                    i + 1
                )));
            }
        }
    }
    Ok(&bytes[..prefix_end])
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

// ---------------------------------------------------------------------------
// fs-touching convenience (the only item in this module that reads a file)
// ---------------------------------------------------------------------------

impl EinmoFile {
    /// Read, parse, and **verify all stamps**; refuse a tampered file.
    ///
    /// This is the verify-on-inspect entry point every reading code path uses.
    ///
    /// # Errors
    ///
    /// Returns [`EinmoError::Io`] if the file cannot be read,
    /// [`EinmoError::Parse`] if malformed, or [`EinmoError::Verification`] if
    /// any stamp fails.
    pub fn from_file(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(|e| EinmoError::io(path, e))?;
        verify_bytes(&bytes).map_err(|e| match e {
            // Annotate verification/parse failures with the path for the caller.
            EinmoError::Verification(msg) => {
                EinmoError::Verification(format!("{}: {msg}", path.display()))
            }
            EinmoError::Parse(msg) => EinmoError::Parse(format!("{}: {msg}", path.display())),
            other => other,
        })
    }
}

// ---------------------------------------------------------------------------
// The `verify` walk over a stage (or all stages)
// ---------------------------------------------------------------------------

/// The outcome of verifying one file in a stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileVerification {
    /// The mirror-relative path within the stage directory.
    pub rel_path: PathBuf,
    /// `true` if every stamp verified.
    pub ok: bool,
    /// A human-readable detail when verification failed.
    pub detail: Option<String>,
}

/// A report over one or more stages.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VerificationReport {
    /// Per-file verdicts, in deterministic path order.
    pub files: Vec<FileVerification>,
}

impl VerificationReport {
    /// `true` if every file verified (or the report is empty).
    #[must_use]
    pub fn all_ok(&self) -> bool {
        self.files.iter().all(|f| f.ok)
    }

    /// The count of files that failed verification.
    #[must_use]
    pub fn failures(&self) -> usize {
        self.files.iter().filter(|f| !f.ok).count()
    }
}

/// Verify every `.einmo` file in `stage` (or across all stages when `None`).
///
/// Discovery mirrors the `input/` tree: for each input, the stage's
/// corresponding `.einmo` is verified if present. When `files` is `Some`, only
/// those user-provided paths (normalized to mirror-relative) are verified.
///
/// # Errors
///
/// Returns [`EinmoError::Io`] if the input tree cannot be walked.
pub fn verify(
    config: &TestConfig,
    stage: Option<Stage>,
    files: Option<&[PathBuf]>,
) -> Result<VerificationReport> {
    let stages: Vec<Stage> = match stage {
        Some(s) => vec![s],
        None => Stage::ALL.to_vec(),
    };
    let mut files_vec = Vec::new();
    for s in &stages {
        let stage_dir = config.stage_dir(*s);
        let rels: Vec<PathBuf> = if let Some(provided) = files {
            let mut v: Vec<PathBuf> = provided
                .iter()
                .map(|p| normalize_file_path(p, config))
                .filter(|p| stage_dir.join(p).exists())
                .collect();
            v.sort();
            v.dedup();
            v
        } else {
            let inputs = walk_input_tree(&config.input_path(), config.walk_depth_limit())?;
            inputs
                .into_iter()
                .map(|p| crate::stage::mirror_input_path(&p))
                .collect()
        };
        for rel in rels {
            let path = stage_dir.join(&rel);
            // Absence is not a verification failure. A stage that has not been
            // promoted to yet simply has no artifact — reporting "FAILED: no
            // such file" for every unpopulated `verified/` entry drowns real
            // tampering in noise and makes `verify --all` red by default.
            // Whether an artifact *ought* to exist is a correspondence question
            // (`compare`); whether a file that exists is sound is this
            // function's question. Missing inputs for existing artifacts are
            // caught by `check_suite_integrity`.
            if !path.exists() {
                continue;
            }
            let verdict = match EinmoFile::from_file(&path) {
                Ok(_) => FileVerification {
                    rel_path: rel,
                    ok: true,
                    detail: None,
                },
                Err(e) => FileVerification {
                    rel_path: rel,
                    ok: false,
                    detail: Some(e.to_string()),
                },
            };
            files_vec.push(verdict);
        }
    }
    Ok(VerificationReport { files: files_vec })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{DEFAULT_SEPARATOR, Metadata, Section, Status};
    use crate::signature::{Stamps, derive_keypair};

    fn build_valid_file() -> EinmoFile {
        let meta = Metadata {
            test: "t.foo".into(),
            suite: "s".into(),
            producer: "abc".into(),
            producer_diff: String::new(),
            generated: "2026-07-11T07:00:00Z".into(),
            status: Status::Normal,
            status_detail: String::new(),
            reference: String::new(),
            sections: vec![
                "INPUT".into(),
                "OUTPUT".into(),
                "COMMENTS".into(),
                "STAMPS".into(),
            ],
        };
        let bodies = vec![
            Section::new("INPUT", "{5;}"),
            Section::new("OUTPUT", "OUTVAL"),
            Section::new("COMMENTS", ""),
        ];
        let mut file = EinmoFile::new("utf-8", DEFAULT_SEPARATOR, meta, bodies, Stamps::new());
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        file.set_stamps(Stamps::generate(&file.signed_prefix(), &configured, &stage));
        file
    }

    #[test]
    fn valid_file_verifies() {
        let file = build_valid_file();
        let bytes = file.serialize().unwrap();
        assert!(verify_bytes(&bytes).is_ok());
    }

    #[test]
    fn tampered_input_refused() {
        let file = build_valid_file();
        let bytes = file.serialize().unwrap();
        let corrupted = flip_first(&bytes, b"{5;}", b"{6;}");
        let err = verify_bytes(&corrupted).unwrap_err();
        assert!(matches!(err, EinmoError::Verification(_)));
    }

    #[test]
    fn tampered_output_refused() {
        let file = build_valid_file();
        let bytes = file.serialize().unwrap();
        // Replace the distinctive OUTPUT body "OUTVAL" with "OUTBAD".
        let corrupted = flip_first(&bytes, b"OUTVAL", b"OUTBAD");
        assert!(verify_bytes(&corrupted).is_err());
    }

    #[test]
    fn tampered_metadata_refused() {
        let file = build_valid_file();
        let bytes = file.serialize().unwrap();
        let corrupted = flip_first(&bytes, b"producer: abc", b"producer: xyz");
        assert!(verify_bytes(&corrupted).is_err());
    }

    #[test]
    fn metadata_whitespace_tamper_detected() {
        // parse() trims values, so a whitespace change in metadata on disk is
        // invisible to the recomputed signed_prefix(). verify_bytes must check
        // the ACTUAL raw bytes, not the canonical recomputation — otherwise this
        // tamper silently passes.
        let file = build_valid_file();
        let bytes = file.serialize().unwrap();
        // Extra space after "producer:" — parse() trims it, so only raw-bytes
        // verification catches this tamper.
        let corrupted = String::from_utf8(bytes.clone()).unwrap();
        let corrupted = corrupted
            .replacen("producer: abc", "producer:  abc", 1)
            .into_bytes();
        let err = verify_bytes(&corrupted).unwrap_err();
        assert!(
            matches!(err, EinmoError::Verification(_)),
            "whitespace tamper must fail verification, got {err:?}"
        );
    }

    #[test]
    fn from_file_refuses_tampered() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("t.foo.einmo");
        let file = build_valid_file();
        let bytes = file.serialize().unwrap();
        let corrupted = flip_first(&bytes, b"{5;}", b"{6;}");
        std::fs::write(&path, corrupted).unwrap();
        assert!(EinmoFile::from_file(&path).is_err());
    }

    #[test]
    fn from_file_accepts_valid() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("t.foo.einmo");
        std::fs::write(&path, build_valid_file().serialize().unwrap()).unwrap();
        assert!(EinmoFile::from_file(&path).is_ok());
    }

    // Replace the first occurrence of `from` with `to` (same length assumed).
    fn flip_first(haystack: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
        assert_eq!(from.len(), to.len());
        let pos = haystack
            .windows(from.len())
            .position(|w| w == from)
            .expect("pattern present");
        let mut out = haystack.to_vec();
        out[pos..pos + to.len()].copy_from_slice(to);
        out
    }
}
