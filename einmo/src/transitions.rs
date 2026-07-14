//! Stage transitions: promotion (append a stamp) and flagging (move + advisory).
//!
//! Every promotion **appends** the destination stage's stamp over all prior
//! bytes; existing stamps are never touched. Flagging is a *move* with an
//! unsigned advisory line and no stamp. All reads go through verify-on-inspect
//! ([`EinmoFile::from_file`]); a tampered source is refused.

use std::path::{Path, PathBuf};

use crate::config::{KeySource, TestConfig};
use crate::error::{EinmoError, Result};
use crate::format::EinmoFile;
use crate::signature::{is_computer_key, now_iso8601};
use crate::stage::{Stage, ensure_parent_dir, mirror_input_path, walk_input_tree};

/// One promoted file's outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Promoted {
    /// The mirror-relative path.
    pub rel_path: PathBuf,
    /// The hex pubkey of the appended stamp.
    pub stamp_pubkey: String,
    /// `true` if the appended verified stamp used a well-known computer key
    /// (a non-human attestation — post-hoc detectable, §B.4).
    pub non_human: bool,
}

/// The result of a promotion over a filter set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PromotionReport {
    /// The files promoted.
    pub promoted: Vec<Promoted>,
}

/// The result of flagging.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlagReport {
    /// The mirror-relative paths moved into `flagged/`.
    pub flagged: Vec<PathBuf>,
}

/// A signature-prefix scan result (`confirm-signatures`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SignatureReport {
    /// Files carrying a stamp whose pubkey starts with the prefix.
    pub matched: Vec<PathBuf>,
    /// Files that verified but carry no matching stamp.
    pub unmatched: Vec<PathBuf>,
}

impl SignatureReport {
    /// `true` if every scanned file carries a matching signer.
    #[must_use]
    pub fn all_matched(&self) -> bool {
        self.unmatched.is_empty()
    }
}

/// The legal stage transitions (FOOP-92 §3).
fn is_legal_transition(from: Stage, to: Stage) -> bool {
    matches!(
        (from, to),
        (Stage::Output, Stage::Checked)
            | (Stage::Output, Stage::Verified)
            | (Stage::Checked, Stage::Verified)
            | (Stage::Output, Stage::Flagged)
            | (Stage::Checked, Stage::Flagged)
            | (Stage::Verified, Stage::Flagged)
            // console-review demotion (re-promotion appends another stamp)
            | (Stage::Verified, Stage::Checked)
    )
}

/// Promote every matching file from `from` to `to`, appending the destination
/// stage's stamp.
///
/// `*->flagged` delegates to [`flag`] (move, no stamp). Other destinations copy
/// the file and append the destination stage stamp signed by `key`.
///
/// # Errors
///
/// Returns [`EinmoError::IllegalTransition`] for a disallowed pair,
/// [`EinmoError::Verification`] if a source file fails verify-on-inspect, or
/// [`EinmoError::Io`] on a filesystem failure.
pub fn promote(
    config: &TestConfig,
    from: Stage,
    to: Stage,
    key: &KeySource,
    filter: Option<&str>,
    files: Option<&[PathBuf]>,
) -> Result<PromotionReport> {
    if !is_legal_transition(from, to) {
        return Err(EinmoError::IllegalTransition {
            from: from.to_string(),
            to: to.to_string(),
        });
    }
    if to == Stage::Flagged {
        // Flagging is a move; promote-to-flagged is the same operation.
        let flag_report = flag(config, from, filter, "", files)?;
        return Ok(PromotionReport {
            promoted: flag_report
                .flagged
                .into_iter()
                .map(|rel| Promoted {
                    rel_path: rel,
                    stamp_pubkey: String::new(),
                    non_human: false,
                })
                .collect(),
        });
    }

    let from_dir = config.stage_dir(from);
    let to_dir = config.stage_dir(to);
    let mut report = PromotionReport::default();

    for rel in matching_mirror_paths(config, from, filter, files)? {
        let src = from_dir.join(&rel);
        let dst = to_dir.join(&rel);
        // Verify-on-inspect the source; refuse a tampered file.
        let mut file = EinmoFile::from_file(&src)?;
        let pubkey = file.append_stage_stamp(to.stamp_key(), key.passphrase());
        let non_human = to == Stage::Verified && is_computer_key(&pubkey);
        ensure_parent_dir(&dst)?;
        let bytes = file.serialize()?;
        std::fs::write(&dst, &bytes).map_err(|e| EinmoError::io(&dst, e))?;
        report.promoted.push(Promoted {
            rel_path: rel,
            stamp_pubkey: pubkey,
            non_human,
        });
    }
    Ok(report)
}

/// Move every matching file from `stage` into `flagged/`, appending an unsigned
/// advisory line. On collision, the new file gets a timestamp suffix.
///
/// # Errors
///
/// Returns [`EinmoError::Verification`] if a source fails verify-on-inspect, or
/// [`EinmoError::Io`] on a filesystem failure.
pub fn flag(
    config: &TestConfig,
    stage: Stage,
    filter: Option<&str>,
    reason: &str,
    files: Option<&[PathBuf]>,
) -> Result<FlagReport> {
    let from_dir = config.stage_dir(stage);
    let flagged_dir = config.stage_dir(Stage::Flagged);
    let mut report = FlagReport::default();
    let timestamp = now_iso8601();

    for rel in matching_mirror_paths(config, stage, filter, files)? {
        let src = from_dir.join(&rel);
        // Verify-on-inspect before moving.
        let mut file = EinmoFile::from_file(&src)?;
        let advisory = format!("# flagged: {reason} {timestamp}");
        file.set_advisory(advisory);

        let dst = collision_free_dest(&flagged_dir, &rel, &timestamp);
        ensure_parent_dir(&dst)?;
        let bytes = file.serialize()?;
        std::fs::write(&dst, &bytes).map_err(|e| EinmoError::io(&dst, e))?;
        // Move semantics: remove from origin.
        std::fs::remove_file(&src).map_err(|e| EinmoError::io(&src, e))?;
        report.flagged.push(rel);
    }
    Ok(report)
}

/// Compute a collision-free destination path in `flagged/`.
///
/// If `flagged/<rel>` already exists, insert `.<timestamp>` before `.einmo`.
fn collision_free_dest(flagged_dir: &Path, rel: &Path, timestamp: &str) -> PathBuf {
    let candidate = flagged_dir.join(rel);
    if !candidate.exists() {
        return candidate;
    }
    // `rel` ends with `.einmo`; strip it, add `.<timestamp>.einmo`.
    let rel_str = rel.to_string_lossy();
    let base = rel_str
        .strip_suffix(".einmo")
        .unwrap_or(&rel_str)
        .to_string();
    let safe_ts = timestamp.replace([':', '/'], "-");
    flagged_dir.join(format!("{base}.{safe_ts}.einmo"))
}

/// Scan every `.einmo` under `path`, reporting which files carry a stamp whose
/// pubkey starts with `pubkey_prefix`.
///
/// # Errors
///
/// Returns [`EinmoError::Io`] if the directory cannot be walked, or
/// [`EinmoError::Verification`] if a file fails verify-on-inspect.
pub fn confirm_signatures(path: &Path, pubkey_prefix: &str) -> Result<SignatureReport> {
    let mut report = SignatureReport::default();
    let mut files = Vec::new();
    collect_einmo_files(path, &mut files, MAX_EINMO_WALK_DEPTH)?;
    files.sort();
    for file_path in files {
        let file = EinmoFile::from_file(&file_path)?;
        let rel = file_path
            .strip_prefix(path)
            .unwrap_or(&file_path)
            .to_path_buf();
        if file.stamps().stamped_by(pubkey_prefix) {
            report.matched.push(rel);
        } else {
            report.unmatched.push(rel);
        }
    }
    Ok(report)
}

/// Recursively collect all `.einmo` files under `dir`.
fn collect_einmo_files(dir: &Path, out: &mut Vec<PathBuf>, depth_limit: usize) -> Result<()> {
    collect_einmo_files_depth(dir, out, 0, depth_limit)
}

const MAX_EINMO_WALK_DEPTH: usize = 64;

fn collect_einmo_files_depth(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    depth: usize,
    depth_limit: usize,
) -> Result<()> {
    if depth > depth_limit {
        return Err(EinmoError::Io {
            path: dir.to_path_buf(),
            source: std::io::Error::other(format!(
                "directory walk exceeded max depth {depth_limit} (possible symlink cycle)"
            )),
        });
    }
    if !dir.exists() {
        return Ok(());
    }
    if dir.is_file() {
        if dir.extension().map(|e| e == "einmo").unwrap_or(false) {
            out.push(dir.to_path_buf());
        }
        return Ok(());
    }
    for entry in std::fs::read_dir(dir).map_err(|e| EinmoError::io(dir, e))? {
        let entry = entry.map_err(|e| EinmoError::io(dir, e))?;
        let p = entry.path();
        let file_type = entry.file_type().map_err(|e| EinmoError::io(&p, e))?;
        if file_type.is_symlink() {
            let metadata = match std::fs::metadata(&p) {
                Ok(m) => m,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(EinmoError::io(&p, e)),
            };
            if metadata.is_dir() {
                collect_einmo_files_depth(&p, out, depth + 1, depth_limit)?;
            } else if metadata.is_file() && p.extension().map(|e| e == "einmo").unwrap_or(false) {
                out.push(p);
            }
        } else if file_type.is_dir() {
            collect_einmo_files_depth(&p, out, depth + 1, depth_limit)?;
        } else if p.extension().map(|e| e == "einmo").unwrap_or(false) {
            out.push(p);
        }
    }
    Ok(())
}

/// The mirror-relative `.einmo` paths present in `stage`.
///
/// When `files` is `Some`, only those user-provided paths are considered (after
/// normalization via [`normalize_file_path`]); `filter` is ignored. When `None`,
/// the full input tree is walked and optionally narrowed by `filter`.
fn matching_mirror_paths(
    config: &TestConfig,
    stage: Stage,
    filter: Option<&str>,
    files: Option<&[PathBuf]>,
) -> Result<Vec<PathBuf>> {
    if let Some(files) = files {
        let stage_dir = config.stage_dir(stage);
        let mut paths: Vec<PathBuf> = files
            .iter()
            .map(|p| normalize_file_path(p, config))
            .filter(|p| stage_dir.join(p).exists())
            .collect();
        paths.sort();
        paths.dedup();
        return Ok(paths);
    }
    let inputs = walk_input_tree(&config.input_path(), config.walk_depth_limit())?;
    let stage_dir = config.stage_dir(stage);
    let mut paths = Vec::new();
    for input_rel in inputs {
        if let Some(pat) = filter
            && !glob_match(&input_rel.to_string_lossy(), pat)
        {
            continue;
        }
        let rel = mirror_input_path(&input_rel);
        if stage_dir.join(&rel).exists() {
            paths.push(rel);
        }
    }
    Ok(paths)
}

/// Normalize a user-provided file path to a mirror-relative `.einmo` path.
///
/// Accepts any of:
/// - `test.einmo` — bare mirror-relative name (used as-is)
/// - `subdir/test.einmo` — mirror-relative path (used as-is)
/// - `output/test.einmo` — stage-relative path (strips the stage-dir prefix)
/// - `checked/sub/test.einmo` — stage-relative path (strips the stage-dir prefix)
/// - `/abs/path/to/suite/output/test.einmo` — absolute path (strips everything
///   up to and including the stage dir)
/// - `test.foo` — input name without `.einmo` (appends `.einmo`)
///
/// The stage-dir prefix check uses both the canonical stage dir names
/// ([`Stage::dir_name`]) and the suite's configured stage dir paths
/// ([`TestConfig::stage_dir`]) so customized directory names are honored.
#[must_use]
pub(crate) fn normalize_file_path(path: &Path, config: &TestConfig) -> PathBuf {
    let path_str = path.to_string_lossy().into_owned();

    // Ends with `.einmo` → mirror-relative, stage-relative, or absolute.
    if path_str.ends_with(".einmo") {
        // Stage-relative: `<stage_dir>/<rel>` for any configured stage name.
        for stage in Stage::ALL {
            let stage_name = config
                .stage_dir(stage)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| stage.dir_name().to_string());
            let prefix = format!("{stage_name}/");
            if path_str.starts_with(&prefix) {
                return PathBuf::from(&path_str[prefix.len()..]);
            }
        }
        // Absolute path: strip everything up to and including the stage dir.
        if path.is_absolute() {
            for stage in Stage::ALL {
                let stage_dir = config.stage_dir(stage);
                if let Ok(rel) = path.strip_prefix(&stage_dir) {
                    return rel.to_path_buf();
                }
            }
        }
        return path.to_path_buf();
    }

    // Doesn't end with `.einmo` → treat as an input name, append `.einmo`.
    mirror_input_path(path)
}

/// A minimal glob: `*` matches any run of characters; everything else literal.
///
/// Kept intentionally small (no `**`, no `?`) — sufficient for `--filter
/// algorithms/sorting/*`; a bare substring is expressible as `*sub*`.
fn glob_match(text: &str, pattern: &str) -> bool {
    // Collapse consecutive `*` into a single `*` to prevent exponential
    // backtracking on pathological patterns like `*****x`.
    let normalized: String = pattern.chars().fold(String::new(), |mut acc, c| {
        if c == '*' && acc.ends_with('*') {
            return acc; // skip: already have a trailing *
        }
        acc.push(c);
        acc
    });
    fn matches(t: &[u8], p: &[u8]) -> bool {
        match (p.first(), t.first()) {
            (None, None) => true,
            (Some(b'*'), _) => {
                // `*` matches zero chars, or one char then `*` again.
                matches(t, &p[1..]) || (!t.is_empty() && matches(&t[1..], p))
            }
            (Some(pc), Some(tc)) if pc == tc => matches(&t[1..], &p[1..]),
            _ => false,
        }
    }
    matches(text.as_bytes(), normalized.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{DEFAULT_SEPARATOR, Metadata, Section, Status};
    use crate::signature::{Stamps, derive_keypair};
    use std::fs;

    fn suite() -> (tempfile::TempDir, TestConfig) {
        let tmp = tempfile::tempdir().unwrap();
        let config = TestConfig::new(tmp.path());
        config.ensure_stage_dirs().unwrap();
        fs::create_dir_all(config.input_path()).unwrap();
        (tmp, config)
    }

    fn write_output(config: &TestConfig, rel: &str, output: &str) {
        fs::write(config.input_path().join(rel), "{5;}").unwrap();
        let bodies = vec![
            Section::new("INPUT", "{5;}"),
            Section::new("OUTPUT", output),
            Section::new("COMMENTS", ""),
        ];
        let meta = Metadata {
            test: rel.into(),
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
        let mut file = EinmoFile::new("utf-8", DEFAULT_SEPARATOR, meta, bodies, Stamps::new());
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        file.set_stamps(Stamps::generate(&file.signed_prefix(), &configured, &stage));
        let path = config.stage_dir(Stage::Output).join(format!("{rel}.einmo"));
        ensure_parent_dir(&path).unwrap();
        fs::write(&path, file.serialize().unwrap()).unwrap();
    }

    #[test]
    fn promote_output_to_checked_appends_stamp() {
        let (_tmp, config) = suite();
        write_output(&config, "a.foo", "5");
        let key = KeySource::from_passphrase("");
        let report = promote(&config, Stage::Output, Stage::Checked, &key, None, None).unwrap();
        assert_eq!(report.promoted.len(), 1);

        let checked = config.stage_dir(Stage::Checked).join("a.foo.einmo");
        let file = EinmoFile::from_file(&checked).unwrap();
        let keys: Vec<&str> = file.stamps().entries().iter().map(|s| s.key()).collect();
        assert_eq!(
            keys,
            vec!["compiled", "configured", "stage:output", "stage:checked"]
        );
        assert!(file.chain_valid());
    }

    #[test]
    fn promote_checked_to_verified_appends_verified_stamp() {
        let (_tmp, config) = suite();
        write_output(&config, "a.foo", "5");
        let key = KeySource::from_passphrase("");
        promote(&config, Stage::Output, Stage::Checked, &key, None, None).unwrap();
        let hkey = KeySource::from_passphrase("human-passphrase");
        let report = promote(&config, Stage::Checked, Stage::Verified, &hkey, None, None).unwrap();
        assert!(
            !report.promoted[0].non_human,
            "human key is not the computer key"
        );
        let verified = config.stage_dir(Stage::Verified).join("a.foo.einmo");
        let file = EinmoFile::from_file(&verified).unwrap();
        assert_eq!(
            file.stamps().highest_stage_stamp().unwrap().key(),
            "stage:verified"
        );
    }

    #[test]
    fn empty_passphrase_verified_is_flagged_non_human() {
        let (_tmp, config) = suite();
        write_output(&config, "a.foo", "5");
        let key = KeySource::from_passphrase("");
        promote(&config, Stage::Output, Stage::Verified, &key, None, None).unwrap();
        let report = {
            // Re-promote a fresh output to verified with empty passphrase.
            write_output(&config, "b.foo", "6");
            promote(&config, Stage::Output, Stage::Verified, &key, None, None).unwrap()
        };
        assert!(
            report.promoted.iter().any(|p| p.non_human),
            "empty-passphrase verified stamp must be flagged non-human"
        );
    }

    #[test]
    fn promote_refuses_tampered_source() {
        let (_tmp, config) = suite();
        write_output(&config, "a.foo", "5");
        // Corrupt the output file (flip a byte inside INPUT `{5;}`).
        let out = config.stage_dir(Stage::Output).join("a.foo.einmo");
        let mut bytes = fs::read(&out).unwrap();
        let pos = bytes.windows(4).position(|w| w == b"{5;}").unwrap();
        bytes[pos + 1] = b'8';
        fs::write(&out, bytes).unwrap();
        let key = KeySource::from_passphrase("");
        let err = promote(&config, Stage::Output, Stage::Checked, &key, None, None).unwrap_err();
        assert!(matches!(err, EinmoError::Verification(_)));
    }

    #[test]
    fn illegal_transition_refused() {
        let (_tmp, config) = suite();
        let key = KeySource::from_passphrase("");
        let err = promote(&config, Stage::Verified, Stage::Output, &key, None, None).unwrap_err();
        assert!(matches!(err, EinmoError::IllegalTransition { .. }));
    }

    #[test]
    fn flag_moves_file_and_writes_advisory() {
        let (_tmp, config) = suite();
        write_output(&config, "a.foo", "5");
        let report = flag(&config, Stage::Output, None, "looks wrong", None).unwrap();
        assert_eq!(report.flagged, vec![PathBuf::from("a.foo.einmo")]);
        // Origin vacated.
        assert!(!config.stage_dir(Stage::Output).join("a.foo.einmo").exists());
        // Flagged file present with advisory, still chain-valid.
        let flagged = config.stage_dir(Stage::Flagged).join("a.foo.einmo");
        let file = EinmoFile::from_file(&flagged).unwrap();
        assert!(
            file.advisory()
                .unwrap()
                .starts_with("# flagged: looks wrong")
        );
        assert!(file.chain_valid(), "advisory must not invalidate stamps");
    }

    #[test]
    fn flag_collision_gets_timestamp_suffix() {
        let (_tmp, config) = suite();
        write_output(&config, "a.foo", "5");
        flag(&config, Stage::Output, None, "first", None).unwrap();
        // Regenerate and flag again to the same path.
        write_output(&config, "a.foo", "5");
        flag(&config, Stage::Output, None, "second", None).unwrap();
        let flagged_dir = config.stage_dir(Stage::Flagged);
        let count = fs::read_dir(&flagged_dir).unwrap().count();
        assert_eq!(count, 2, "collision must produce a second, suffixed file");
    }

    #[test]
    fn confirm_signatures_matches_prefix() {
        let (_tmp, config) = suite();
        write_output(&config, "a.foo", "5");
        // The output stage key is the empty-passphrase computer key.
        let (_, computer) = derive_keypair("");
        let prefix = &hex::encode(computer.to_bytes())[..8];
        let report = confirm_signatures(&config.stage_dir(Stage::Output), prefix).unwrap();
        assert_eq!(report.matched, vec![PathBuf::from("a.foo.einmo")]);
        assert!(report.all_matched());

        let none = confirm_signatures(&config.stage_dir(Stage::Output), "ffffffff").unwrap();
        assert!(!none.all_matched());
        assert_eq!(none.unmatched, vec![PathBuf::from("a.foo.einmo")]);
    }

    #[test]
    fn glob_matches_subtree() {
        assert!(glob_match(
            "algorithms/sorting/quick.foo",
            "algorithms/sorting/*"
        ));
        assert!(!glob_match(
            "algorithms/searching/bin.foo",
            "algorithms/sorting/*"
        ));
        assert!(glob_match("anything", "*"));
        assert!(glob_match("a/b/c", "*b*"));
    }

    #[test]
    fn glob_match_consecutive_stars_dont_backtrack() {
        assert!(glob_match("hello", "*****hello"));
        assert!(!glob_match("hello", "****x"));
    }

    #[test]
    fn normalize_paths() {
        let (_tmp, config) = suite();
        let out_dir = config.stage_dir(Stage::Output);
        let checked_dir = config.stage_dir(Stage::Checked);

        assert_eq!(
            normalize_file_path(Path::new("test.einmo"), &config),
            PathBuf::from("test.einmo")
        );
        assert_eq!(
            normalize_file_path(Path::new("sub/test.einmo"), &config),
            PathBuf::from("sub/test.einmo")
        );
        assert_eq!(
            normalize_file_path(Path::new("output/test.einmo"), &config),
            PathBuf::from("test.einmo")
        );
        assert_eq!(
            normalize_file_path(Path::new("checked/sub/test.einmo"), &config),
            PathBuf::from("sub/test.einmo")
        );
        assert_eq!(
            normalize_file_path(Path::new("test.foo"), &config),
            PathBuf::from("test.foo.einmo")
        );
        assert_eq!(
            normalize_file_path(&out_dir.join("deep/nested.einmo"), &config),
            PathBuf::from("deep/nested.einmo")
        );
        assert_eq!(
            normalize_file_path(&checked_dir.join("x.einmo"), &config),
            PathBuf::from("x.einmo")
        );
    }

    fn write_three(config: &TestConfig) {
        write_output(config, "a.foo", "5");
        write_output(config, "b.foo", "6");
        write_output(config, "c.foo", "7");
    }

    #[test]
    fn promote_single_file() {
        let (_tmp, config) = suite();
        write_three(&config);
        let key = KeySource::from_passphrase("");
        let files = vec![PathBuf::from("a.foo.einmo")];
        let report = promote(
            &config,
            Stage::Output,
            Stage::Checked,
            &key,
            None,
            Some(&files),
        )
        .unwrap();
        assert_eq!(report.promoted.len(), 1);
        assert_eq!(report.promoted[0].rel_path, PathBuf::from("a.foo.einmo"));
        assert!(
            config
                .stage_dir(Stage::Checked)
                .join("a.foo.einmo")
                .exists()
        );
        assert!(
            !config
                .stage_dir(Stage::Checked)
                .join("b.foo.einmo")
                .exists()
        );
        assert!(
            !config
                .stage_dir(Stage::Checked)
                .join("c.foo.einmo")
                .exists()
        );
    }

    #[test]
    fn promote_multiple_files() {
        let (_tmp, config) = suite();
        write_three(&config);
        let key = KeySource::from_passphrase("");
        let files = vec![PathBuf::from("a.foo.einmo"), PathBuf::from("c.foo.einmo")];
        let report = promote(
            &config,
            Stage::Output,
            Stage::Checked,
            &key,
            None,
            Some(&files),
        )
        .unwrap();
        assert_eq!(report.promoted.len(), 2);
        let promoted: Vec<PathBuf> = report.promoted.iter().map(|p| p.rel_path.clone()).collect();
        assert!(promoted.contains(&PathBuf::from("a.foo.einmo")));
        assert!(promoted.contains(&PathBuf::from("c.foo.einmo")));
        assert!(
            config
                .stage_dir(Stage::Checked)
                .join("a.foo.einmo")
                .exists()
        );
        assert!(
            config
                .stage_dir(Stage::Checked)
                .join("c.foo.einmo")
                .exists()
        );
        assert!(
            !config
                .stage_dir(Stage::Checked)
                .join("b.foo.einmo")
                .exists()
        );
    }

    #[test]
    fn promote_files_stage_relative_and_absolute() {
        let (_tmp, config) = suite();
        write_three(&config);
        let key = KeySource::from_passphrase("");
        // `output/b.foo.einmo` is stage-relative.
        let files = vec![PathBuf::from("output/b.foo.einmo")];
        let report = promote(
            &config,
            Stage::Output,
            Stage::Checked,
            &key,
            None,
            Some(&files),
        )
        .unwrap();
        assert_eq!(report.promoted.len(), 1);
        assert!(
            config
                .stage_dir(Stage::Checked)
                .join("b.foo.einmo")
                .exists()
        );
    }

    #[test]
    fn flag_single_file() {
        let (_tmp, config) = suite();
        write_three(&config);
        let files = vec![PathBuf::from("b.foo.einmo")];
        let report = flag(&config, Stage::Output, None, "one bad", Some(&files)).unwrap();
        assert_eq!(report.flagged, vec![PathBuf::from("b.foo.einmo")]);
        assert!(!config.stage_dir(Stage::Output).join("b.foo.einmo").exists());
        assert!(config.stage_dir(Stage::Output).join("a.foo.einmo").exists());
        assert!(config.stage_dir(Stage::Output).join("c.foo.einmo").exists());
        assert!(
            config
                .stage_dir(Stage::Flagged)
                .join("b.foo.einmo")
                .exists()
        );
    }

    #[test]
    fn promote_files_ignores_filter() {
        let (_tmp, config) = suite();
        write_three(&config);
        let key = KeySource::from_passphrase("");
        let files = vec![PathBuf::from("a.foo.einmo")];
        let report = promote(
            &config,
            Stage::Output,
            Stage::Checked,
            &key,
            Some("*b*"),
            Some(&files),
        )
        .unwrap();
        assert_eq!(report.promoted.len(), 1, "files must override --filter");
    }
}
