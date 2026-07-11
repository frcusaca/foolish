//! # zweimomo
//!
//! Einmo's companion test crate — three pure-Rust `Evaluator` implementations
//! (Foolish via `foolish-ubca`, Python via `rustpython-vm`, JavaScript via
//! `boa_engine`) proving the `einmo::Evaluator` trait is language-agnostic.

pub mod evaluators;

pub use evaluators::{
    BoaEvaluator, RustPythonEvaluator, UbcaEvaluatorAdapter, brane_name_perspective,
};

#[cfg(test)]
mod suite_tests {
    use super::*;
    use einmo::config::{Perspective, PerspectiveOf, Stage, TestConfig};
    use einmo::snapshot_suite::EinmoSuite;
    use std::path::Path;

    fn copy_tree(src: &Path, dst: &Path) {
        for entry in walkdir(src) {
            let rel = entry.strip_prefix(src).unwrap();
            let target = dst.join(rel);
            if entry.is_dir() {
                std::fs::create_dir_all(&target).unwrap();
            } else {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                std::fs::copy(entry, &target).unwrap();
            }
        }
    }

    fn walkdir(dir: &Path) -> Vec<std::path::PathBuf> {
        let mut result = Vec::new();
        if !dir.exists() {
            return result;
        }
        walkdir_recursive(dir, &mut result);
        result
    }

    fn walkdir_recursive(dir: &Path, result: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                result.push(path.clone());
                if path.is_dir() {
                    walkdir_recursive(&path, result);
                }
            }
        }
    }

    #[test]
    fn foolish_suite_approval() {
        let work_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("suites/foolish");

        let p = Perspective::new("brane_names", PerspectiveOf::Input, |s| {
            brane_name_perspective(s)
        });

        let config = TestConfig::new(&work_dir)
            .with_separator("!!\n")
            .with_perspective(p)
            .require(Stage::Output, Stage::Checked);

        let suite = EinmoSuite::new(config);
        let evaluator = UbcaEvaluatorAdapter;

        let results = suite.evaluate_all(&evaluator);
        for f in &results.files {
            if let Err(e) = &f.result {
                eprintln!("foolish eval error for {}: {e}", f.path.display());
            }
        }

        let output_dir = work_dir.join("output");
        let checked_dir = work_dir.join("checked");
        copy_tree(&output_dir, &checked_dir);

        let config2 = TestConfig::new(&work_dir)
            .with_separator("!!\n")
            .require(Stage::Output, Stage::Checked);

        let suite2 = EinmoSuite::new(config2);
        let results2 = suite2.evaluate_all(&evaluator);

        assert!(
            results2.all_output_written_and_verified(),
            "Foolish correspondence failures: {:?}",
            results2.correspondence_failures
        );
    }

    #[test]
    fn python_suite_approval() {
        let work_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("suites/python");

        let config = TestConfig::new(&work_dir).require(Stage::Output, Stage::Checked);

        let suite = EinmoSuite::new(config);
        let evaluator = RustPythonEvaluator;

        let results = suite.evaluate_all(&evaluator);
        for f in &results.files {
            if let Err(e) = &f.result {
                eprintln!("python eval error for {}: {e}", f.path.display());
            }
        }

        let output_dir = work_dir.join("output");
        let checked_dir = work_dir.join("checked");
        copy_tree(&output_dir, &checked_dir);

        let config2 = TestConfig::new(&work_dir).require(Stage::Output, Stage::Checked);
        let suite2 = EinmoSuite::new(config2);
        let results2 = suite2.evaluate_all(&evaluator);

        assert!(
            results2.all_output_written_and_verified(),
            "Python correspondence failures: {:?}",
            results2.correspondence_failures
        );
    }

    #[test]
    fn javascript_suite_approval() {
        let work_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("suites/javascript");

        let config = TestConfig::new(&work_dir).require(Stage::Output, Stage::Checked);

        let suite = EinmoSuite::new(config);
        let evaluator = BoaEvaluator;

        let results = suite.evaluate_all(&evaluator);
        for f in &results.files {
            if let Err(e) = &f.result {
                eprintln!("javascript eval error for {}: {e}", f.path.display());
            }
        }

        let output_dir = work_dir.join("output");
        let checked_dir = work_dir.join("checked");
        copy_tree(&output_dir, &checked_dir);

        let config2 = TestConfig::new(&work_dir).require(Stage::Output, Stage::Checked);
        let suite2 = EinmoSuite::new(config2);
        let results2 = suite2.evaluate_all(&evaluator);

        assert!(
            results2.all_output_written_and_verified(),
            "JavaScript correspondence failures: {:?}",
            results2.correspondence_failures
        );
    }
}
