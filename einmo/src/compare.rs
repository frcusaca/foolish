//! Per-section stage comparison (FOOP-54 §5).
//!
//! Walks two stage trees in parallel by mirror-relative path, verifies each
//! file (verify-on-inspect), and compares configured sections byte-for-byte.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::config::{MatchSections, Stage, TestConfig};
use crate::format::EinmoFile;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Per-path comparison outcome.
#[derive(Debug)]
pub struct DiffEntry {
    pub path: PathBuf,
    pub sections: Vec<String>,
}

/// Result of comparing two stages.
#[derive(Debug)]
pub struct ComparisonResult {
    pub matching: Vec<PathBuf>,
    pub differing: Vec<DiffEntry>,
    pub only_in_a: Vec<PathBuf>,
    pub only_in_b: Vec<PathBuf>,
    pub tampered: Vec<PathBuf>,
}

impl ComparisonResult {
    pub fn is_clean(&self) -> bool {
        self.differing.is_empty()
            && self.only_in_a.is_empty()
            && self.only_in_b.is_empty()
            && self.tampered.is_empty()
    }
}

// ---------------------------------------------------------------------------
// compare
// ---------------------------------------------------------------------------

/// Compare two stages by walking their directory trees and comparing configured
/// sections byte-for-byte.
///
/// Files that fail verify-on-inspect land in `tampered`, not `differing`.
/// STAMPS and metadata are never compared.
pub fn compare(
    config: &TestConfig,
    a: Stage,
    b: Stage,
    sections: MatchSections,
) -> ComparisonResult {
    let dir_a = config.stage_dir(a);
    let dir_b = config.stage_dir(b);

    let files_a = collect_einmo_rel_paths(&dir_a);
    let files_b = collect_einmo_rel_paths(&dir_b);

    let set_a: BTreeSet<&PathBuf> = files_a.iter().collect();
    let set_b: BTreeSet<&PathBuf> = files_b.iter().collect();

    let mut matching = Vec::new();
    let mut differing = Vec::new();
    let mut only_in_a = Vec::new();
    let mut only_in_b = Vec::new();
    let mut tampered = Vec::new();

    for path in &files_a {
        if !set_b.contains(path) {
            only_in_a.push(path.clone());
        }
    }
    for path in &files_b {
        if !set_a.contains(path) {
            only_in_b.push(path.clone());
        }
    }

    let common: Vec<&PathBuf> = set_a.intersection(&set_b).copied().collect();

    for rel_path in common {
        let path_a = dir_a.join(rel_path);
        let path_b = dir_b.join(rel_path);

        let einmo_a = match EinmoFile::from_file(&path_a) {
            Ok(e) => e,
            Err(_) => {
                tampered.push(rel_path.clone());
                continue;
            }
        };

        let einmo_b = match EinmoFile::from_file(&path_b) {
            Ok(e) => e,
            Err(_) => {
                tampered.push(rel_path.clone());
                continue;
            }
        };

        let diff_sections = compare_sections(&einmo_a, &einmo_b, sections);
        if diff_sections.is_empty() {
            matching.push(rel_path.clone());
        } else {
            differing.push(DiffEntry {
                path: rel_path.clone(),
                sections: diff_sections,
            });
        }
    }

    matching.sort();
    differing.sort_by(|a, b| a.path.cmp(&b.path));
    only_in_a.sort();
    only_in_b.sort();
    tampered.sort();

    ComparisonResult {
        matching,
        differing,
        only_in_a,
        only_in_b,
        tampered,
    }
}

// ---------------------------------------------------------------------------
// Section comparison
// ---------------------------------------------------------------------------

fn compare_sections(a: &EinmoFile, b: &EinmoFile, sections: MatchSections) -> Vec<String> {
    let mut diffs = Vec::new();

    let section_names = required_section_names(a, b, sections);

    for name in &section_names {
        let content_a = a.section(name);
        let content_b = b.section(name);

        match (content_a, content_b) {
            (Some(bytes_a), Some(bytes_b)) => {
                if bytes_a != bytes_b {
                    diffs.push(name.clone());
                }
            }
            (None, Some(_)) | (Some(_), None) => {
                diffs.push(name.clone());
            }
            (None, None) => {}
        }
    }

    diffs
}

fn required_section_names(a: &EinmoFile, b: &EinmoFile, sections: MatchSections) -> Vec<String> {
    let mut names = BTreeSet::new();
    names.insert("INPUT".to_string());

    for file in [a, b] {
        for sec in file.sections_list() {
            if sec == "STAMPS" || sec == "COMMENTS" {
                continue;
            }
            if sec.starts_with("OUTPUT") || sec == "DIFF" {
                names.insert(sec.clone());
            }
        }
    }

    if sections == MatchSections::InputOutputComments {
        names.insert("COMMENTS".to_string());
    }

    names.into_iter().collect()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn collect_einmo_rel_paths(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if dir.exists() {
        collect_recursive(dir, dir, &mut results);
    }
    results.sort();
    results
}

fn collect_recursive(base: &Path, dir: &Path, results: &mut Vec<PathBuf>) {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if metadata.is_dir() {
            collect_recursive(base, &path, results);
        } else if metadata.is_file()
            && path.extension().map(|e| e == "einmo").unwrap_or(false)
            && let Ok(rel) = path.strip_prefix(base)
        {
            results.push(rel.to_path_buf());
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot_suite::{EinmoSuite, Evaluator};

    struct EchoEval;
    impl Evaluator for EchoEval {
        fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
            Ok(vec![format!("echo: {source}")])
        }
    }

    fn setup_suite(dir: &Path) {
        let input = dir.join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("test.foo"), "hello").unwrap();
    }

    fn generate_to(dir: &Path, stage: Stage) {
        let config = TestConfig::new(dir);
        let suite = EinmoSuite::new(config);
        let input = dir.join("input").join("test.foo");
        suite.evaluate(&input, &EchoEval).unwrap();

        if stage != Stage::Output {
            let cfg = TestConfig::new(dir);
            let rel = Path::new("test.foo.einmo");
            crate::stage::promote(&cfg, Stage::Output, stage, rel, "").unwrap();
        }
    }

    #[test]
    fn identical_stages_all_matching() {
        let dir = tempfile::tempdir().unwrap();
        setup_suite(dir.path());
        generate_to(dir.path(), Stage::Output);

        let dir_a = dir.path().join("checked");
        std::fs::create_dir_all(&dir_a).unwrap();
        let src = dir.path().join("output").join("test.foo.einmo");
        let dst = dir_a.join("test.foo.einmo");
        std::fs::copy(&src, &dst).unwrap();

        let cfg = TestConfig::new(dir.path());
        let result = compare(
            &cfg,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
        );
        assert!(result.is_clean(), "should be clean: {result:?}");
        assert_eq!(result.matching.len(), 1);
    }

    #[test]
    fn missing_files_only_in_a_only_in_b() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("a.foo"), "a").unwrap();
        std::fs::write(input.join("b.foo"), "b").unwrap();

        let config = TestConfig::new(dir.path());
        let suite = EinmoSuite::new(config);
        suite.evaluate(&input.join("a.foo"), &EchoEval).unwrap();
        suite.evaluate(&input.join("b.foo"), &EchoEval).unwrap();

        let dir_b = dir.path().join("checked");
        std::fs::create_dir_all(&dir_b).unwrap();
        let src = dir.path().join("output").join("a.foo.einmo");
        std::fs::copy(&src, dir_b.join("a.foo.einmo")).unwrap();

        let cfg = TestConfig::new(dir.path());
        let result = compare(
            &cfg,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
        );
        assert_eq!(result.only_in_a.len(), 1);
        assert_eq!(result.only_in_a[0], PathBuf::from("b.foo.einmo"));
        assert!(result.only_in_b.is_empty());
    }

    struct DifferentEval;
    impl Evaluator for DifferentEval {
        fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
            Ok(vec![format!("different: {source}")])
        }
    }

    #[test]
    fn content_diff_in_output_differing() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("test.foo"), "hello").unwrap();

        let config = TestConfig::new(dir.path());
        let suite = EinmoSuite::new(config);
        suite.evaluate(&input.join("test.foo"), &EchoEval).unwrap();

        let checked_dir = dir.path().join("checked");
        std::fs::create_dir_all(&checked_dir).unwrap();
        let src = dir.path().join("output").join("test.foo.einmo");
        std::fs::copy(&src, checked_dir.join("test.foo.einmo")).unwrap();

        let config2 = TestConfig::new(dir.path());
        let suite2 = EinmoSuite::new(config2);
        suite2
            .evaluate(&input.join("test.foo"), &DifferentEval)
            .unwrap();

        let cfg = TestConfig::new(dir.path());
        let result = compare(
            &cfg,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
        );
        assert_eq!(result.differing.len(), 1);
        assert!(result.differing[0].sections.contains(&"OUTPUT".to_string()));
    }

    #[test]
    fn comments_diff_with_inputoutput_is_matching() {
        let dir = tempfile::tempdir().unwrap();
        setup_suite(dir.path());
        generate_to(dir.path(), Stage::Output);

        let dir_b = dir.path().join("checked");
        std::fs::create_dir_all(&dir_b).unwrap();
        let src = dir.path().join("output").join("test.foo.einmo");
        std::fs::copy(&src, dir_b.join("test.foo.einmo")).unwrap();

        let cfg = TestConfig::new(dir.path());
        let result = compare(
            &cfg,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
        );
        assert!(
            result
                .matching
                .iter()
                .any(|p| p == &PathBuf::from("test.foo.einmo")),
            "identical files should match with InputOutput: {result:?}"
        );
    }

    #[test]
    fn tampered_file_goes_to_tampered_not_differing() {
        let dir = tempfile::tempdir().unwrap();
        setup_suite(dir.path());
        generate_to(dir.path(), Stage::Output);

        let dir_b = dir.path().join("checked");
        std::fs::create_dir_all(&dir_b).unwrap();
        let original = std::fs::read(dir.path().join("output").join("test.foo.einmo")).unwrap();
        let tampered = String::from_utf8_lossy(&original).replace("echo: hello", "TAMPERED");
        std::fs::write(dir_b.join("test.foo.einmo"), tampered.as_bytes()).unwrap();

        let cfg = TestConfig::new(dir.path());
        let result = compare(
            &cfg,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
        );
        assert_eq!(
            result.tampered.len(),
            1,
            "tampered file should be in tampered"
        );
        assert!(result.differing.is_empty(), "should not be in differing");
    }

    #[test]
    fn stamps_only_diff_is_matching() {
        let dir = tempfile::tempdir().unwrap();
        setup_suite(dir.path());
        generate_to(dir.path(), Stage::Output);

        let dir_b = dir.path().join("checked");
        std::fs::create_dir_all(&dir_b).unwrap();
        let src = dir.path().join("output").join("test.foo.einmo");
        std::fs::copy(&src, dir_b.join("test.foo.einmo")).unwrap();

        let cfg = TestConfig::new(dir.path());
        let result = compare(
            &cfg,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
        );
        assert!(result.is_clean());
        assert_eq!(result.matching.len(), 1);
    }

    #[test]
    fn empty_stages_both_empty() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = TestConfig::new(dir.path());
        let result = compare(
            &cfg,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
        );
        assert!(result.is_clean());
        assert!(result.matching.is_empty());
    }

    #[test]
    fn comparison_result_is_clean() {
        let clean = ComparisonResult {
            matching: vec![PathBuf::from("a")],
            differing: vec![],
            only_in_a: vec![],
            only_in_b: vec![],
            tampered: vec![],
        };
        assert!(clean.is_clean());

        let dirty = ComparisonResult {
            matching: vec![],
            differing: vec![DiffEntry {
                path: PathBuf::from("x"),
                sections: vec!["OUTPUT".into()],
            }],
            only_in_a: vec![],
            only_in_b: vec![],
            tampered: vec![],
        };
        assert!(!dirty.is_clean());
    }
}
