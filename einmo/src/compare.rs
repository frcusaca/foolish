//! Stage-to-stage comparison with per-section matching (FOOP-92 §5).
//!
//! Two `.einmo` files **match** iff both verify against their own stamps and
//! their *configured sections* are byte-identical. STAMPS and metadata are
//! never compared (they legitimately differ between stages / by construction).

use std::path::{Path, PathBuf};

pub use crate::config::MatchSections;
use crate::config::TestConfig;
use crate::error::Result;
use crate::format::EinmoFile;
use crate::stage::{Stage, mirror_input_path, walk_input_tree};
use crate::transitions::normalize_file_path;

/// A file that differs, naming which configured section(s) diverged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffEntry {
    /// The mirror-relative path.
    pub rel_path: PathBuf,
    /// The names of the sections that differed.
    pub sections: Vec<String>,
}

/// The result of comparing two stages over the mirrored tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComparisonResult {
    /// Files whose configured sections are byte-identical.
    pub matching: Vec<PathBuf>,
    /// Files present in both stages but differing (with section names).
    pub differing: Vec<DiffEntry>,
    /// Files present only in stage A.
    pub only_in_a: Vec<PathBuf>,
    /// Files present only in stage B.
    pub only_in_b: Vec<PathBuf>,
    /// Files that failed verify-on-inspect (refused; not compared).
    pub tampered: Vec<PathBuf>,
}

impl ComparisonResult {
    /// `true` if nothing differs, nothing is one-sided, and nothing is tampered.
    ///
    /// This is the gate condition for `--require-match`.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.differing.is_empty()
            && self.only_in_a.is_empty()
            && self.only_in_b.is_empty()
            && self.tampered.is_empty()
    }
}

/// Which section names must be byte-identical for a given file, per policy.
///
/// INPUT and every OUTPUT[i] are always required; DIFF is required when the
/// file is a dependent (has `reference:` metadata); COMMENTS is added by
/// [`MatchSections::InputOutputComments`].
fn required_sections(file: &EinmoFile, policy: MatchSections) -> Vec<String> {
    file.sections()
        .iter()
        .map(|section| section.name())
        .filter(|name| is_required_section(name, policy))
        .map(str::to_string)
        .collect()
}

/// Whether a section name must be byte-identical under `policy`.
fn is_required_section(name: &str, policy: MatchSections) -> bool {
    let is_output = name == "OUTPUT" || (name.starts_with("OUTPUT[") && name.ends_with(']'));
    let always = name == "INPUT" || name == "DIFF" || is_output;
    let comments = name == "COMMENTS" && policy == MatchSections::InputOutputComments;
    always || comments
}

/// Compare `a` against `b` over the mirrored input tree.
///
/// When `files` is `Some`, only those user-provided paths (normalized to
/// mirror-relative) are compared; the full input-tree walk is skipped. When
/// `None`, all mirrored `.einmo` files present in either stage are compared.
///
/// # Errors
///
/// Returns [`crate::EinmoError::Io`] if the input tree cannot be walked.
pub fn compare(
    config: &TestConfig,
    a: Stage,
    b: Stage,
    sections: MatchSections,
    files: Option<&[PathBuf]>,
) -> Result<ComparisonResult> {
    let dir_a = config.stage_dir(a);
    let dir_b = config.stage_dir(b);
    let mut result = ComparisonResult::default();

    let rels: Vec<PathBuf> = if let Some(files) = files {
        let mut v: Vec<PathBuf> = files
            .iter()
            .map(|p| normalize_file_path(p, config))
            .collect();
        v.sort();
        v.dedup();
        v
    } else {
        let inputs = walk_input_tree(&config.input_path(), config.walk_depth_limit())?;
        inputs.into_iter().map(|p| mirror_input_path(&p)).collect()
    };

    for rel in &rels {
        let path_a = dir_a.join(rel);
        let path_b = dir_b.join(rel);
        match (path_a.exists(), path_b.exists()) {
            (false, false) => {}
            (true, false) => result.only_in_a.push(rel.clone()),
            (false, true) => result.only_in_b.push(rel.clone()),
            (true, true) => {
                // Verify-on-inspect both; a failure means tampered, not differing.
                let (file_a, file_b) =
                    match (EinmoFile::from_file(&path_a), EinmoFile::from_file(&path_b)) {
                        (Ok(fa), Ok(fb)) => (fa, fb),
                        _ => {
                            result.tampered.push(rel.clone());
                            continue;
                        }
                    };
                let diverged = compare_sections(&file_a, &file_b, sections);
                if diverged.is_empty() {
                    result.matching.push(rel.clone());
                } else {
                    result.differing.push(DiffEntry {
                        rel_path: rel.clone(),
                        sections: diverged,
                    });
                }
            }
        }
    }
    Ok(result)
}

/// Return the names of required sections that differ (or are missing) between
/// the two files.
fn compare_sections(a: &EinmoFile, b: &EinmoFile, policy: MatchSections) -> Vec<String> {
    // Required-section names are taken from A; a missing section in B counts as
    // a difference too.
    let required = required_sections(a, policy);
    let mut diverged = Vec::new();
    for name in required {
        let body_a = a.section(&name).map(|s| s.body());
        let body_b = b.section(&name).map(|s| s.body());
        if body_a != body_b {
            diverged.push(name);
        }
    }
    diverged
}

/// Descend a `differing` file's subtree and report the deepest differing
/// descendants (the candidate root causes) — the `--root-cause` diagnostic.
///
/// Given the full comparison, for each `differing` path this returns those
/// differing paths that have no differing descendant strictly below them.
#[must_use]
pub fn root_causes(result: &ComparisonResult) -> Vec<PathBuf> {
    let differing: Vec<&PathBuf> = result.differing.iter().map(|d| &d.rel_path).collect();
    differing
        .iter()
        .filter(|candidate| {
            // A candidate is a root cause if no *other* differing path is
            // strictly deeper within its subtree.
            !differing.iter().any(|other| {
                other != *candidate && is_strict_descendant(other.as_path(), candidate.as_path())
            })
        })
        .map(|p| (*p).clone())
        .collect()
}

/// `true` if `descendant` lives in `ancestor`'s subtree and is strictly deeper.
///
/// A shallower file `ops/division.foo.einmo` owns the subtree directory
/// `ops/division/`; a deeper file under that directory is its descendant.
fn is_strict_descendant(descendant: &Path, ancestor: &Path) -> bool {
    let Some(subtree) = subtree_dir(ancestor) else {
        return false;
    };
    descendant.starts_with(&subtree)
        && descendant.components().count() > ancestor.components().count()
}

/// The subtree directory a file owns: its path with the `.einmo` and the
/// input extension stripped (e.g. `ops/division.foo.einmo` → `ops/division`).
fn subtree_dir(path: &Path) -> Option<PathBuf> {
    // Strip `.einmo` then the input extension (`.foo`, `.py`, …).
    let without_einmo = path.file_name()?.to_string_lossy();
    let base = without_einmo
        .strip_suffix(".einmo")
        .unwrap_or(&without_einmo);
    let stem = Path::new(base).file_stem()?.to_string_lossy().into_owned();
    Some(match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.join(stem),
        _ => PathBuf::from(stem),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::einmo_suite::ValidationLevel;
    use crate::format::{DEFAULT_SEPARATOR, Metadata, Section, Status};
    use crate::signature::{Stamps, derive_keypair};
    use std::fs;
    use std::path::Path;

    fn write_file(dir: &Path, rel: &str, bodies: Vec<Section>, reference: &str) {
        let mut section_names: Vec<String> = bodies.iter().map(|s| s.name().to_string()).collect();
        section_names.push("STAMPS".into());
        let meta = Metadata {
            test: rel.into(),
            suite: "s".into(),
            producer: "abc".into(),
            producer_diff: String::new(),
            generated: "2026-07-11T07:00:00Z".into(),
            status: Status::Normal,
            status_detail: String::new(),
            reference: reference.into(),
            sections: section_names,
        };
        let mut file = EinmoFile::new("utf-8", DEFAULT_SEPARATOR, meta, bodies, Stamps::new());
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        file.set_stamps(Stamps::generate(&file.signed_prefix(), &configured, &stage));
        let path = dir.join(mirror_input_path(Path::new(rel)));
        crate::stage::ensure_parent_dir(&path).unwrap();
        fs::write(&path, file.serialize().unwrap()).unwrap();
    }

    fn suite() -> (tempfile::TempDir, TestConfig) {
        let tmp = tempfile::tempdir().unwrap();
        let config = TestConfig::new(tmp.path(), ValidationLevel::Output);
        config.ensure_stage_dirs().unwrap();
        (tmp, config)
    }

    fn body(input: &str, output: &str, comments: &str) -> Vec<Section> {
        vec![
            Section::new("INPUT", input),
            Section::new("OUTPUT", output),
            Section::new("COMMENTS", comments),
        ]
    }

    #[test]
    fn identical_stages_all_matching() {
        let (tmp, config) = suite();
        fs::create_dir_all(config.input_path()).unwrap();
        fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        write_file(
            &config.stage_dir(Stage::Output),
            "a.foo",
            body("{5;}", "5", ""),
            "",
        );
        write_file(
            &config.stage_dir(Stage::Checked),
            "a.foo",
            body("{5;}", "5", ""),
            "",
        );

        let r = compare(
            &config,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
            None,
        )
        .unwrap();
        assert_eq!(r.matching.len(), 1);
        assert!(r.is_clean());
        drop(tmp);
    }

    #[test]
    fn only_in_a_and_only_in_b() {
        let (_tmp, config) = suite();
        fs::create_dir_all(config.input_path()).unwrap();
        fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        fs::write(config.input_path().join("b.foo"), "{6;}").unwrap();
        write_file(
            &config.stage_dir(Stage::Output),
            "a.foo",
            body("{5;}", "5", ""),
            "",
        );
        write_file(
            &config.stage_dir(Stage::Checked),
            "b.foo",
            body("{6;}", "6", ""),
            "",
        );

        let r = compare(
            &config,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
            None,
        )
        .unwrap();
        assert_eq!(r.only_in_a, vec![PathBuf::from("a.foo.einmo")]);
        assert_eq!(r.only_in_b, vec![PathBuf::from("b.foo.einmo")]);
        assert!(!r.is_clean());
    }

    #[test]
    fn output_diff_is_differing() {
        let (_tmp, config) = suite();
        fs::create_dir_all(config.input_path()).unwrap();
        fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        write_file(
            &config.stage_dir(Stage::Output),
            "a.foo",
            body("{5;}", "5", ""),
            "",
        );
        write_file(
            &config.stage_dir(Stage::Checked),
            "a.foo",
            body("{5;}", "9", ""),
            "",
        );

        let r = compare(
            &config,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
            None,
        )
        .unwrap();
        assert_eq!(r.differing.len(), 1);
        assert_eq!(r.differing[0].sections, vec!["OUTPUT"]);
    }

    #[test]
    fn comments_diff_ignored_by_default_but_caught_when_required() {
        let (_tmp, config) = suite();
        fs::create_dir_all(config.input_path()).unwrap();
        fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        write_file(
            &config.stage_dir(Stage::Output),
            "a.foo",
            body("{5;}", "5", "note A"),
            "",
        );
        write_file(
            &config.stage_dir(Stage::Checked),
            "a.foo",
            body("{5;}", "5", "note B"),
            "",
        );

        let default = compare(
            &config,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
            None,
        )
        .unwrap();
        assert_eq!(
            default.matching.len(),
            1,
            "COMMENTS drift ignored by default"
        );

        let strict = compare(
            &config,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutputComments,
            None,
        )
        .unwrap();
        assert_eq!(strict.differing.len(), 1);
        assert_eq!(strict.differing[0].sections, vec!["COMMENTS"]);
    }

    #[test]
    fn tampered_file_is_tampered_not_differing() {
        let (_tmp, config) = suite();
        fs::create_dir_all(config.input_path()).unwrap();
        fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        write_file(
            &config.stage_dir(Stage::Output),
            "a.foo",
            body("{5;}", "5", ""),
            "",
        );
        write_file(
            &config.stage_dir(Stage::Checked),
            "a.foo",
            body("{5;}", "5", ""),
            "",
        );
        // Corrupt the checked file on disk (flip a byte inside INPUT `{5;}`).
        let checked_path = config.stage_dir(Stage::Checked).join("a.foo.einmo");
        let mut bytes = fs::read(&checked_path).unwrap();
        let pos = bytes.windows(4).position(|w| w == b"{5;}").unwrap();
        bytes[pos + 1] = b'8';
        fs::write(&checked_path, bytes).unwrap();

        let r = compare(
            &config,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
            None,
        )
        .unwrap();
        assert_eq!(r.tampered, vec![PathBuf::from("a.foo.einmo")]);
        assert!(
            r.differing.is_empty(),
            "tampered must not be counted as differing"
        );
    }

    #[test]
    fn stamps_only_diff_still_matches() {
        // Two files with identical INPUT/OUTPUT but generated at different
        // times → different stamp bytes, but content matches.
        let (_tmp, config) = suite();
        fs::create_dir_all(config.input_path()).unwrap();
        fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        write_file(
            &config.stage_dir(Stage::Output),
            "a.foo",
            body("{5;}", "5", ""),
            "",
        );
        write_file(
            &config.stage_dir(Stage::Checked),
            "a.foo",
            body("{5;}", "5", ""),
            "",
        );
        let r = compare(
            &config,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
            None,
        )
        .unwrap();
        assert_eq!(r.matching.len(), 1, "STAMPS section is never compared");
    }

    #[test]
    fn root_cause_descends_to_deepest() {
        let result = ComparisonResult {
            differing: vec![
                DiffEntry {
                    rel_path: PathBuf::from("ops.foo.einmo"),
                    sections: vec!["OUTPUT".into()],
                },
                DiffEntry {
                    rel_path: PathBuf::from("ops/division.foo.einmo"),
                    sections: vec!["OUTPUT".into()],
                },
                DiffEntry {
                    rel_path: PathBuf::from("ops/division/by_zero.foo.einmo"),
                    sections: vec!["OUTPUT".into()],
                },
            ],
            ..Default::default()
        };
        let roots = root_causes(&result);
        assert_eq!(roots, vec![PathBuf::from("ops/division/by_zero.foo.einmo")]);
    }

    #[test]
    fn compare_single_file() {
        let (_tmp, config) = suite();
        fs::create_dir_all(config.input_path()).unwrap();
        fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        fs::write(config.input_path().join("b.foo"), "{6;}").unwrap();
        // a matches; b differs.
        write_file(
            &config.stage_dir(Stage::Output),
            "a.foo",
            body("{5;}", "5", ""),
            "",
        );
        write_file(
            &config.stage_dir(Stage::Checked),
            "a.foo",
            body("{5;}", "5", ""),
            "",
        );
        write_file(
            &config.stage_dir(Stage::Output),
            "b.foo",
            body("{6;}", "6", ""),
            "",
        );
        write_file(
            &config.stage_dir(Stage::Checked),
            "b.foo",
            body("{6;}", "9", ""),
            "",
        );

        // Only compare a.foo.einmo — b must not appear.
        let files = vec![PathBuf::from("a.foo.einmo")];
        let r = compare(
            &config,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
            Some(&files),
        )
        .unwrap();
        assert_eq!(r.matching.len(), 1);
        assert!(r.differing.is_empty());

        // Only compare b.foo.einmo — it differs.
        let files = vec![PathBuf::from("b.foo.einmo")];
        let r = compare(
            &config,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
            Some(&files),
        )
        .unwrap();
        assert_eq!(r.differing.len(), 1);
        assert_eq!(r.differing[0].rel_path, PathBuf::from("b.foo.einmo"));
        assert!(r.matching.is_empty());
    }

    #[test]
    fn compare_files_only_in_one_stage() {
        let (_tmp, config) = suite();
        fs::create_dir_all(config.input_path()).unwrap();
        fs::write(config.input_path().join("a.foo"), "{5;}").unwrap();
        write_file(
            &config.stage_dir(Stage::Output),
            "a.foo",
            body("{5;}", "5", ""),
            "",
        );
        // Nothing in checked/.
        let files = vec![PathBuf::from("a.foo.einmo")];
        let r = compare(
            &config,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
            Some(&files),
        )
        .unwrap();
        assert_eq!(r.only_in_a, vec![PathBuf::from("a.foo.einmo")]);
        assert!(r.only_in_b.is_empty());
    }
}
