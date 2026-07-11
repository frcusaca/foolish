//! Stage directories, hierarchical mirroring, and stage transitions.
//!
//! Provides [`mirror_input_path`] for mapping input files to their stage
//! counterparts, [`walk_input_tree`] for discovering inputs,
//! [`ensure_stage_dirs`] for creating the stage directory tree, and
//! [`promote`]/[`flag`]/[`confirm_signatures`] for stage transitions.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::config::{ConfigError, Stage, TestConfig};
use crate::format::EinmoFile;
use crate::signature::{
    SignatureError, compiled_keypair, create_stage_stamp, derive_keypair, verify_all_stamps,
};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from stage directory operations.
#[derive(Debug, Error)]
pub enum StageError {
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("path error: {0}")]
    Path(String),
    #[error("format error: {0}")]
    Format(#[from] crate::format::EinmoError),
    #[error("signature error: {0}")]
    Signature(#[from] SignatureError),
    #[error("verification failed: {0}")]
    VerificationFailed(String),
    #[error("chain integrity error: {0}")]
    ChainIntegrity(String),
}

// ---------------------------------------------------------------------------
// Reference resolution (dependent einmos, spec §4.7)
// ---------------------------------------------------------------------------

/// Resolve the reference input name for a dependent test.
///
/// A name containing the dependent separator (default `++`) references the
/// same-directory input named by stripping the **last** `++segment`.
/// Chains resolve recursively: `base++a++b` → `base++a` → `base`.
///
/// Returns `None` if the name does not contain the separator (it is a root
/// reference test, not a dependent).
pub fn resolve_reference(name: &str, separator: &str) -> Option<String> {
    name.rfind(separator).map(|pos| {
        let after_sep = &name[pos + separator.len()..];
        let base = &name[..pos];
        if let Some(dot_pos) = after_sep.find('.') {
            let ext = &after_sep[dot_pos..];
            format!("{base}{ext}")
        } else {
            base.to_string()
        }
    })
}

/// Topologically sort input names so that references are evaluated before
/// their dependents.
///
/// Within a group sharing the same root (before the first `++`), the sort
/// is by dependency depth first, then lexicographic for stability.
pub fn topo_sort_inputs(names: &mut [String], separator: &str) {
    names.sort_by(|a, b| {
        let depth_a = a.matches(separator).count();
        let depth_b = b.matches(separator).count();
        let root_a = root_name(a, separator);
        let root_b = root_name(b, separator);
        root_a
            .cmp(root_b)
            .then(depth_a.cmp(&depth_b))
            .then(a.cmp(b))
    });
}

fn root_name<'a>(name: &'a str, separator: &str) -> &'a str {
    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
    stem.split(separator).next().unwrap_or(stem)
}

// ---------------------------------------------------------------------------
// Hierarchical mirroring
// ---------------------------------------------------------------------------

/// Given an input-relative path, produce the mirror stage path.
///
/// Appends `.einmo` to the filename component.  Discovery is extension-agnostic —
/// any file under `input/` is a test trigger (`.foo`, `.py`, `.js`, etc.).
///
/// # Examples
///
/// ```ignore
/// // stage1/section3/specific.test → stage1/section3/specific.test.einmo
/// let mirrored = mirror_input_path(Path::new("stage1/section3/specific.test"));
/// assert_eq!(mirrored, PathBuf::from("stage1/section3/specific.test.einmo"));
/// ```
pub fn mirror_input_path(input_rel_path: &Path) -> PathBuf {
    let mut name = input_rel_path.file_name().unwrap_or_default().to_owned();
    name.push(".einmo");
    input_rel_path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(name)
}

// ---------------------------------------------------------------------------
// Input tree discovery
// ---------------------------------------------------------------------------

/// Discover all input files under the configured input directory.
///
/// Returns their mirror-relative paths (with `.einmo` appended), sorted.
/// Returns an empty vec if the input directory does not exist.
pub fn walk_input_tree(config: &TestConfig) -> Result<Vec<PathBuf>, StageError> {
    let input_dir = config.input_path();

    if !input_dir.exists() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    walk_recursive(&input_dir, &input_dir, &mut results)?;
    results.sort();
    Ok(results)
}

fn walk_recursive(base: &Path, dir: &Path, results: &mut Vec<PathBuf>) -> Result<(), StageError> {
    let read_dir = std::fs::read_dir(dir)?;
    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            walk_recursive(base, &path, results)?;
        } else if metadata.is_file() {
            let rel = path
                .strip_prefix(base)
                .map_err(|e| StageError::Path(e.to_string()))?;
            results.push(mirror_input_path(rel));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Stage directory creation
// ---------------------------------------------------------------------------

/// Create all stage directories under the work directory.
///
/// Idempotent — succeeds silently if directories already exist.
pub fn ensure_stage_dirs(config: &TestConfig) -> Result<(), StageError> {
    for stage in Stage::all() {
        let dir = config.stage_dir(stage);
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Stage transitions: promote / flag / confirm_signatures
// ---------------------------------------------------------------------------

/// Result of a promotion operation.
#[derive(Debug)]
pub struct PromotionReport {
    pub files_promoted: Vec<PathBuf>,
    pub files_skipped: Vec<PathBuf>,
}

/// Result of a flag operation.
#[derive(Debug)]
pub struct FlagReport {
    pub files_flagged: Vec<PathBuf>,
}

/// Result of a confirm-signatures scan.
#[derive(Debug)]
pub struct SignatureReport {
    pub matching: Vec<PathBuf>,
    pub non_matching: Vec<PathBuf>,
}

impl SignatureReport {
    pub fn all_match(&self) -> bool {
        self.non_matching.is_empty()
    }
}

/// Promote a file from one stage to another.
///
/// Copies the file, appending the destination stage's stamp. For `*->flagged`,
/// delegates to [`flag`] (move + advisory line, no stamp).
///
/// Refuses if the source file fails verify-on-inspect or if any existing stamp
/// fails chain integrity.
pub fn promote(
    config: &TestConfig,
    from: Stage,
    to: Stage,
    rel_path: &Path,
    passphrase: &str,
) -> Result<PromotionReport, StageError> {
    if to == Stage::Flagged {
        let report = flag(config, from, rel_path, &format!("promoted from {from}"))?;
        return Ok(PromotionReport {
            files_promoted: report.files_flagged,
            files_skipped: Vec::new(),
        });
    }

    let src_path = config.stage_dir(from).join(rel_path);
    if !src_path.exists() {
        return Ok(PromotionReport {
            files_promoted: Vec::new(),
            files_skipped: vec![rel_path.to_path_buf()],
        });
    }

    let src_bytes = std::fs::read(&src_path)?;
    let (_compiled_sk, compiled_vk) = compiled_keypair()?;
    let einmo = EinmoFile::parse(&src_bytes)?;
    let signed_bytes = einmo.signed_bytes()?;

    let verification = verify_all_stamps(&signed_bytes, einmo.stamps(), &compiled_vk);
    for (stamp, status) in &verification {
        if let crate::signature::StampStatus::Invalid(reason) = status {
            return Err(StageError::VerificationFailed(format!(
                "stamp '{}' invalid: {reason}",
                stamp.key()
            )));
        }
    }

    let dest_path = config.stage_dir(to).join(rel_path);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let stage_name = to.dir_name();
    let (_stage_sk, _stage_vk) = derive_keypair(passphrase)?;

    let mut accumulated = signed_bytes;
    for entry in einmo.stamps().entries() {
        let line = serde_json::to_string(entry).expect("stamp serializes");
        accumulated.extend_from_slice(line.as_bytes());
        accumulated.push(b'\n');
    }

    let new_stamp = create_stage_stamp(&_stage_sk, stage_name, &accumulated);
    let mut new_stamps = einmo.stamps().clone();
    new_stamps.push(new_stamp);

    let einmo_with_stamp = einmo.with_stamps(new_stamps)?;
    let dest_bytes = einmo_with_stamp.serialize()?;
    std::fs::write(&dest_path, &dest_bytes)?;

    Ok(PromotionReport {
        files_promoted: vec![rel_path.to_path_buf()],
        files_skipped: Vec::new(),
    })
}

/// Move a file from a stage to `flagged/`, appending an advisory line.
///
/// The advisory `# flagged: <reason> <ISO8601>` is written outside signed
/// content. Collision handling: if `flagged/<rel>` already exists, the new
/// file gets a timestamp suffix.
pub fn flag(
    config: &TestConfig,
    from: Stage,
    rel_path: &Path,
    reason: &str,
) -> Result<FlagReport, StageError> {
    let src_path = config.stage_dir(from).join(rel_path);
    if !src_path.exists() {
        return Err(StageError::Path(format!(
            "source file not found: {}",
            src_path.display()
        )));
    }

    let src_bytes = std::fs::read(&src_path)?;
    let (_compiled_sk, compiled_vk) = compiled_keypair()?;
    let einmo = EinmoFile::parse(&src_bytes)?;
    let signed_bytes = einmo.signed_bytes()?;

    let verification = verify_all_stamps(&signed_bytes, einmo.stamps(), &compiled_vk);
    for (stamp, status) in &verification {
        if let crate::signature::StampStatus::Invalid(reason) = status {
            return Err(StageError::VerificationFailed(format!(
                "stamp '{}' invalid: {reason}",
                stamp.key()
            )));
        }
    }

    let flagged_dir = config.stage_dir(Stage::Flagged);
    let mut dest_path = flagged_dir.join(rel_path);

    if dest_path.exists() {
        let now = time::OffsetDateTime::now_utc();
        let ts = now
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
            .replace(':', "-");
        let stem = rel_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let ext = rel_path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let parent = rel_path.parent().unwrap_or_else(|| Path::new(""));
        let new_name = format!("{stem}.{ts}{ext}");
        dest_path = flagged_dir.join(parent).join(new_name);
    }

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let now = time::OffsetDateTime::now_utc();
    let ts = now
        .format(&time::format_description::well_known::Iso8601::DEFAULT)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into());
    let advisory = format!("# flagged: {reason} {ts}");

    let einmo_with_advisory = einmo.with_advisory_line(&advisory);
    let dest_bytes = einmo_with_advisory.serialize()?;
    std::fs::write(&dest_path, &dest_bytes)?;
    std::fs::remove_file(&src_path)?;

    Ok(FlagReport {
        files_flagged: vec![rel_path.to_path_buf()],
    })
}

/// Scan all `.einmo` files under `path` for signers whose pubkey starts with
/// `pubkey_prefix`.
pub fn confirm_signatures(path: &Path, pubkey_prefix: &str) -> Result<SignatureReport, StageError> {
    let (_compiled_sk, compiled_vk) = compiled_keypair()?;
    let mut matching = Vec::new();
    let mut non_matching = Vec::new();

    let mut einmo_files = Vec::new();
    collect_einmo_files(path, &mut einmo_files)?;
    einmo_files.sort();

    for file_path in &einmo_files {
        let bytes = match std::fs::read(file_path) {
            Ok(b) => b,
            Err(_) => {
                non_matching.push(file_path.clone());
                continue;
            }
        };

        let einmo = match EinmoFile::parse(&bytes) {
            Ok(e) => e,
            Err(_) => {
                non_matching.push(file_path.clone());
                continue;
            }
        };

        let signed_bytes = match einmo.signed_bytes() {
            Ok(b) => b,
            Err(_) => {
                non_matching.push(file_path.clone());
                continue;
            }
        };

        let verification = verify_all_stamps(&signed_bytes, einmo.stamps(), &compiled_vk);
        let all_valid = verification
            .iter()
            .all(|(_, s)| matches!(s, crate::signature::StampStatus::Valid));
        if !all_valid {
            non_matching.push(file_path.clone());
            continue;
        }

        let has_match = einmo
            .stamps()
            .entries()
            .iter()
            .any(|s| s.pubkey().starts_with(pubkey_prefix));

        if has_match {
            matching.push(file_path.clone());
        } else {
            non_matching.push(file_path.clone());
        }
    }

    Ok(SignatureReport {
        matching,
        non_matching,
    })
}

fn collect_einmo_files(dir: &Path, results: &mut Vec<PathBuf>) -> Result<(), StageError> {
    if !dir.exists() {
        return Ok(());
    }
    let read_dir = std::fs::read_dir(dir)?;
    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_einmo_files(&path, results)?;
        } else if metadata.is_file() && path.extension().map(|e| e == "einmo").unwrap_or(false) {
            results.push(path);
        }
    }
    Ok(())
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StageDirs;

    // -- mirror_input_path --------------------------------------------------

    #[test]
    fn flat_input_flat_stage_path() {
        let input = Path::new("specific.test");
        let mirrored = mirror_input_path(input);
        assert_eq!(mirrored, PathBuf::from("specific.test.einmo"));
    }

    #[test]
    fn hierarchical_input_mirrored_stage_path() {
        let input = Path::new("stage1/section3/specific.test");
        let mirrored = mirror_input_path(input);
        assert_eq!(
            mirrored,
            PathBuf::from("stage1/section3/specific.test.einmo")
        );
    }

    #[test]
    fn deeply_nested_path() {
        let input = Path::new("a/b/c/d/e/test.foo");
        let mirrored = mirror_input_path(input);
        assert_eq!(mirrored, PathBuf::from("a/b/c/d/e/test.foo.einmo"));
    }

    #[test]
    fn same_basename_different_branches_coexist() {
        let a = mirror_input_path(Path::new("branch_a/test.foo"));
        let b = mirror_input_path(Path::new("branch_b/test.foo"));
        assert_ne!(a, b);
        assert_eq!(a, PathBuf::from("branch_a/test.foo.einmo"));
        assert_eq!(b, PathBuf::from("branch_b/test.foo.einmo"));
    }

    #[test]
    fn non_foo_extension_discovered() {
        let py = mirror_input_path(Path::new("algo/test.py"));
        let js = mirror_input_path(Path::new("algo/test.js"));
        let txt = mirror_input_path(Path::new("algo/test.txt"));
        assert_eq!(py, PathBuf::from("algo/test.py.einmo"));
        assert_eq!(js, PathBuf::from("algo/test.js.einmo"));
        assert_eq!(txt, PathBuf::from("algo/test.txt.einmo"));
    }

    // -- stage_dir ----------------------------------------------------------

    #[test]
    fn stage_dir_per_stage() {
        let cfg = TestConfig::new("/tmp/suite");
        assert_eq!(
            cfg.stage_dir(Stage::Output),
            PathBuf::from("/tmp/suite/output")
        );
        assert_eq!(
            cfg.stage_dir(Stage::Checked),
            PathBuf::from("/tmp/suite/checked")
        );
        assert_eq!(
            cfg.stage_dir(Stage::Flagged),
            PathBuf::from("/tmp/suite/flagged")
        );
        assert_eq!(
            cfg.stage_dir(Stage::Verified),
            PathBuf::from("/tmp/suite/verified")
        );
    }

    // -- walk_input_tree + ensure_stage_dirs (filesystem) --------------------

    #[test]
    fn walk_flat_input_tree() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("a.foo"), "a").unwrap();
        std::fs::write(input.join("b.foo"), "b").unwrap();

        let cfg = TestConfig::new(dir.path());
        let mut files = walk_input_tree(&cfg).unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![PathBuf::from("a.foo.einmo"), PathBuf::from("b.foo.einmo"),]
        );
    }

    #[test]
    fn walk_hierarchical_input_tree() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(input.join("s1/s2")).unwrap();
        std::fs::write(input.join("top.foo"), "").unwrap();
        std::fs::write(input.join("s1/mid.foo"), "").unwrap();
        std::fs::write(input.join("s1/s2/deep.foo"), "").unwrap();

        let cfg = TestConfig::new(dir.path());
        let files = walk_input_tree(&cfg).unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.contains(&PathBuf::from("top.foo.einmo")));
        assert!(files.contains(&PathBuf::from("s1/mid.foo.einmo")));
        assert!(files.contains(&PathBuf::from("s1/s2/deep.foo.einmo")));
    }

    #[test]
    fn walk_non_foo_extensions_discovered() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("test.py"), "").unwrap();
        std::fs::write(input.join("test.js"), "").unwrap();
        std::fs::write(input.join("test.foo"), "").unwrap();

        let cfg = TestConfig::new(dir.path());
        let mut files = walk_input_tree(&cfg).unwrap();
        files.sort();
        assert_eq!(
            files,
            vec![
                PathBuf::from("test.foo.einmo"),
                PathBuf::from("test.js.einmo"),
                PathBuf::from("test.py.einmo"),
            ]
        );
    }

    #[test]
    fn walk_missing_input_dir_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = TestConfig::new(dir.path());
        let files = walk_input_tree(&cfg).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn ensure_stage_dirs_creates_all() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = TestConfig::new(dir.path());
        ensure_stage_dirs(&cfg).unwrap();

        assert!(dir.path().join("output").is_dir());
        assert!(dir.path().join("checked").is_dir());
        assert!(dir.path().join("flagged").is_dir());
        assert!(dir.path().join("verified").is_dir());
    }

    #[test]
    fn ensure_stage_dirs_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = TestConfig::new(dir.path());
        ensure_stage_dirs(&cfg).unwrap();
        // Second call should succeed silently.
        ensure_stage_dirs(&cfg).unwrap();
    }

    #[test]
    fn ensure_stage_dirs_custom_names() {
        let dir = tempfile::tempdir().unwrap();
        let stages = StageDirs::new("gen", "review", "hold", "sign").unwrap();
        let cfg = TestConfig::new(dir.path()).with_stages(stages);
        ensure_stage_dirs(&cfg).unwrap();

        assert!(dir.path().join("gen").is_dir());
        assert!(dir.path().join("review").is_dir());
        assert!(dir.path().join("hold").is_dir());
        assert!(dir.path().join("sign").is_dir());
    }

    // -- promote / flag / confirm_signatures ---------------------------------

    use crate::format::EinmoFile;
    use crate::snapshot_suite::{EinmoSuite, Evaluator};

    struct EchoEval;
    impl Evaluator for EchoEval {
        fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
            Ok(vec![format!("echo: {source}")])
        }
    }

    fn generate_output(dir: &Path) {
        let config = TestConfig::new(dir);
        let suite = EinmoSuite::new(config);

        let input = dir.join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("test.foo"), "hello").unwrap();

        let result = suite.evaluate(&input.join("test.foo"), &EchoEval);
        assert!(result.is_ok(), "generate output: {:?}", result.err());
    }

    #[test]
    fn promote_output_to_checked_appends_stage_checked() {
        let dir = tempfile::tempdir().unwrap();
        generate_output(dir.path());

        let cfg = TestConfig::new(dir.path());
        let rel = Path::new("test.foo.einmo");

        let report = promote(&cfg, Stage::Output, Stage::Checked, rel, "").unwrap();
        assert_eq!(report.files_promoted.len(), 1);

        let checked_path = dir.path().join("checked").join(rel);
        assert!(checked_path.exists(), "checked file should exist");

        let bytes = std::fs::read(&checked_path).unwrap();
        let einmo = EinmoFile::parse(&bytes).unwrap();
        let stage_keys: Vec<&str> = einmo
            .stamps()
            .entries()
            .iter()
            .filter_map(|s| s.stage_name())
            .collect();
        assert!(
            stage_keys.contains(&"checked"),
            "should have stage:checked stamp, got {stage_keys:?}"
        );
        assert!(
            stage_keys.contains(&"output"),
            "should still have stage:output stamp"
        );
    }

    #[test]
    fn promote_checked_to_verified_appends_stage_verified() {
        let dir = tempfile::tempdir().unwrap();
        generate_output(dir.path());

        let cfg = TestConfig::new(dir.path());
        let rel = Path::new("test.foo.einmo");

        promote(&cfg, Stage::Output, Stage::Checked, rel, "").unwrap();
        promote(&cfg, Stage::Checked, Stage::Verified, rel, "human-secret").unwrap();

        let verified_path = dir.path().join("verified").join(rel);
        assert!(verified_path.exists());

        let bytes = std::fs::read(&verified_path).unwrap();
        let einmo = EinmoFile::parse(&bytes).unwrap();
        let stage_keys: Vec<&str> = einmo
            .stamps()
            .entries()
            .iter()
            .filter_map(|s| s.stage_name())
            .collect();
        assert!(stage_keys.contains(&"verified"));
        assert!(stage_keys.contains(&"output"));
    }

    #[test]
    fn promote_refuses_on_tampered_source() {
        let dir = tempfile::tempdir().unwrap();
        generate_output(dir.path());

        let output_path = dir.path().join("output").join("test.foo.einmo");
        let bytes = std::fs::read(&output_path).unwrap();
        let tampered = String::from_utf8_lossy(&bytes).replace("echo: hello", "TAMPERED");
        std::fs::write(&output_path, tampered.as_bytes()).unwrap();

        let cfg = TestConfig::new(dir.path());
        let rel = Path::new("test.foo.einmo");
        let result = promote(&cfg, Stage::Output, Stage::Checked, rel, "");
        assert!(result.is_err(), "should refuse on tampered source");
        assert!(
            format!("{}", result.unwrap_err()).contains("invalid"),
            "error should mention invalid stamp"
        );
    }

    #[test]
    fn flag_moves_file_origin_vacated() {
        let dir = tempfile::tempdir().unwrap();
        generate_output(dir.path());

        let cfg = TestConfig::new(dir.path());
        let rel = Path::new("test.foo.einmo");
        let report = flag(&cfg, Stage::Output, rel, "out of date").unwrap();
        assert_eq!(report.files_flagged.len(), 1);

        let origin = dir.path().join("output").join(rel);
        assert!(!origin.exists(), "origin should be vacated");

        let flagged = dir.path().join("flagged").join(rel);
        assert!(flagged.exists(), "flagged file should exist");
    }

    #[test]
    fn flag_collision_gets_timestamp_suffix() {
        let dir = tempfile::tempdir().unwrap();
        generate_output(dir.path());

        let cfg = TestConfig::new(dir.path());
        let rel = Path::new("test.foo.einmo");

        flag(&cfg, Stage::Output, rel, "first flag").unwrap();

        generate_output(dir.path());
        flag(&cfg, Stage::Output, rel, "second flag").unwrap();

        let flagged_dir = dir.path().join("flagged");
        let mut entries: Vec<String> = std::fs::read_dir(&flagged_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        entries.sort();

        assert!(
            entries.len() >= 2,
            "should have at least 2 flagged files, got {entries:?}"
        );
    }

    #[test]
    fn flag_advisory_line_outside_signed_bytes() {
        let dir = tempfile::tempdir().unwrap();
        generate_output(dir.path());

        let cfg = TestConfig::new(dir.path());
        let rel = Path::new("test.foo.einmo");

        flag(&cfg, Stage::Output, rel, "test reason").unwrap();

        let flagged_path = dir.path().join("flagged").join(rel);
        let bytes = std::fs::read(&flagged_path).unwrap();
        let einmo = EinmoFile::parse(&bytes).unwrap();

        assert_eq!(einmo.advisory_lines().len(), 1);
        assert!(einmo.advisory_lines()[0].contains("flagged: test reason"));

        let signed = einmo.signed_bytes().unwrap();
        let signed_str = String::from_utf8_lossy(&signed);
        assert!(
            !signed_str.contains("# flagged:"),
            "advisory line must not be in signed bytes"
        );
    }

    #[test]
    fn confirm_signatures_matches_prefix() {
        let dir = tempfile::tempdir().unwrap();
        generate_output(dir.path());

        let bytes = std::fs::read(dir.path().join("output").join("test.foo.einmo")).unwrap();
        let einmo = EinmoFile::parse(&bytes).unwrap();
        let pubkey = einmo.stamps().entries()[0].pubkey().to_string();
        let prefix = &pubkey[..8];

        let output_dir = dir.path().join("output");
        let report = confirm_signatures(&output_dir, prefix).unwrap();
        assert_eq!(report.matching.len(), 1, "should match the file");
        assert!(report.all_match());
    }

    #[test]
    fn confirm_signatures_no_match_on_wrong_prefix() {
        let dir = tempfile::tempdir().unwrap();
        generate_output(dir.path());

        let output_dir = dir.path().join("output");
        let report = confirm_signatures(&output_dir, "deadbeef").unwrap();
        assert_eq!(report.matching.len(), 0);
        assert_eq!(report.non_matching.len(), 1);
        assert!(!report.all_match());
    }

    // -- resolve_reference / topo_sort_inputs --------------------------------

    #[test]
    fn reference_resolution_simple() {
        assert_eq!(
            resolve_reference("base.foo++variant.foo", "++"),
            Some("base.foo.foo".to_string())
        );
    }

    #[test]
    fn reference_resolution_preserves_extension() {
        assert_eq!(
            resolve_reference("base++inc.foo", "++"),
            Some("base.foo".to_string())
        );
    }

    #[test]
    fn reference_resolution_chain() {
        assert_eq!(
            resolve_reference("base++a++b.foo", "++"),
            Some("base++a.foo".to_string())
        );
    }

    #[test]
    fn reference_resolution_no_separator() {
        assert_eq!(resolve_reference("base.foo", "++"), None);
    }

    #[test]
    fn reference_resolution_custom_separator() {
        assert_eq!(
            resolve_reference("base::variant", "::"),
            Some("base".to_string())
        );
    }

    #[test]
    fn topo_sort_references_before_dependents() {
        let mut names = vec![
            "base++a++b".to_string(),
            "base".to_string(),
            "base++a".to_string(),
        ];
        topo_sort_inputs(&mut names, "++");
        assert_eq!(names, vec!["base", "base++a", "base++a++b"]);
    }

    #[test]
    fn topo_sort_multiple_roots() {
        let mut names = vec![
            "other++x".to_string(),
            "base++a".to_string(),
            "other".to_string(),
            "base".to_string(),
        ];
        topo_sort_inputs(&mut names, "++");
        assert_eq!(names, vec!["base", "base++a", "other", "other++x"]);
    }

    #[test]
    fn topo_sort_no_dependents() {
        let mut names = vec!["z.foo".to_string(), "a.foo".to_string()];
        topo_sort_inputs(&mut names, "++");
        assert_eq!(names, vec!["a.foo", "z.foo"]);
    }
}
