//! Einmo gates for FOOP-36's hand-authored Foolish rendering contract.

use std::path::PathBuf;

use crate::{SequenceMode, Ubca2Sequencer, UbcaEvaluator};

fn einmo_suite2_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("einmo_suite2")
}

#[cfg(test)]
mod einmo_tests {
    use super::*;
    use einmo::{EinmoSuite, Evaluator, Stage, TestConfig, ValidationLevel};
    use std::sync::{Mutex, MutexGuard, PoisonError};

    // These two gates share suite2/output and therefore serialize with each
    // other. They do not share files with einmo_suite's three gates, so a
    // cross-module lock would add latency without protecting any resource.
    static GATE_LOCK: Mutex<()> = Mutex::new(());

    fn gate_lock() -> MutexGuard<'static, ()> {
        GATE_LOCK.lock().unwrap_or_else(PoisonError::into_inner)
    }

    #[derive(Debug, Default, Clone, Copy)]
    struct Ubca2FoolishAdapter;

    impl Evaluator for Ubca2FoolishAdapter {
        fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
            let (storage, firs) = UbcaEvaluator.evaluate_arena(source)?;
            Ok(firs
                .into_iter()
                .map(|fir| Ubca2Sequencer::format(&storage, fir, SequenceMode::Foolish))
                .collect())
        }
    }

    fn config(level: ValidationLevel) -> TestConfig {
        TestConfig::new(einmo_suite2_dir(), level)
    }

    fn assert_evaluation(level: ValidationLevel) -> einmo::TestResults {
        let config = match level {
            ValidationLevel::Checked => {
                config(level).require_correspondence(Stage::Output, Stage::Checked)
            }
            _ => config(level),
        };
        let results = EinmoSuite::new(config)
            .evaluate_all(&Ubca2FoolishAdapter)
            .expect("evaluate_all must not fail at the filesystem level");

        assert!(
            !results.files.is_empty(),
            "einmo suite2 discovered no inputs"
        );
        for file in &results.files {
            assert!(
                file.written_and_verified,
                "{} was not written+verified: {:?}",
                file.rel_path.display(),
                file.detail
            );
        }
        assert!(
            results.integrity.is_clean(),
            "einmo_suite2 is not sound at {level:?}:\n{}",
            results.integrity.report()
        );
        results
    }

    #[test]
    fn einmo_suite2_gate_output() {
        let _gate = gate_lock();
        let _results = assert_evaluation(ValidationLevel::Output);
    }

    #[test]
    fn einmo_suite2_gate_checked() {
        let _gate = gate_lock();
        let results = assert_evaluation(ValidationLevel::Checked);
        assert!(
            results.correspondence_failures.is_empty(),
            "suite2 output differs from the hand-authored rendering contract:\n  {}",
            results.correspondence_failures.join("\n  ")
        );
    }

    /// Every `.foo` file under a directory, relative to that directory,
    /// with the OS-specific separator normalized to `/` for comparison.
    fn foo_inputs_under(dir: &std::path::Path) -> std::collections::BTreeSet<String> {
        fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, root, out);
                } else if path.extension().is_some_and(|ext| ext == "foo") {
                    out.push(path.strip_prefix(root).unwrap().to_path_buf());
                }
            }
        }
        let mut paths = Vec::new();
        walk(dir, dir, &mut paths);
        paths
            .into_iter()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .collect()
    }

    /// T10 (FOOP-36 Phase 5a) — coverage parity. `einmo_suite2` must contain
    /// every input `einmo_suite` does, at the same relative path, so the
    /// suite that replaces it does not quietly test less. This is invisible
    /// from a green gate run — a missing input just means fewer cases, not
    /// a failure — so it needs its own explicit assertion.
    #[test]
    fn einmo_suite2_has_every_einmo_suite_input() {
        let suite_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("einmo_suite");
        let suite2_dir = einmo_suite2_dir();

        let original: std::collections::BTreeSet<String> =
            foo_inputs_under(&suite_dir.join("input"));
        let migrated: std::collections::BTreeSet<String> =
            foo_inputs_under(&suite2_dir.join("input"));

        let missing: Vec<&String> = original.difference(&migrated).collect();
        assert!(
            missing.is_empty(),
            "einmo_suite2 is missing {} input(s) present in einmo_suite: {:?}",
            missing.len(),
            missing
        );

        assert_eq!(
            original.len(),
            179,
            "einmo_suite's own input count moved ({} inputs found) — re-verify the parity \
             check still means what this test says it means before trusting it",
            original.len()
        );
        assert_eq!(
            migrated.len(),
            180,
            "einmo_suite2 should hold the 179 copied inputs plus foop/36/rendering_contract.foo \
             from Phase 3; found {} instead",
            migrated.len()
        );
    }

    /// T3 (FOOP-36 §2, §Test Plan) — corpus-wide Property 1. Walks every
    /// `einmo_suite2` input directly (not through einmo's signed-output
    /// machinery — this must run BEFORE any output is generated, per the
    /// plan, as the cheapest possible check that the renderer survives the
    /// whole corpus) and asserts the Foolish-mode rendering re-parses.
    /// Property 1 only, not idempotence (§2.1) — some inputs may not settle.
    #[test]
    fn einmo_suite2_corpus_wide_foolish_rendering_parses() {
        let suite2_dir = einmo_suite2_dir();
        let inputs = foo_inputs_under(&suite2_dir.join("input"));
        assert!(!inputs.is_empty(), "no einmo_suite2 inputs found to check");

        let mut failures = Vec::new();
        for rel in &inputs {
            let path = suite2_dir.join("input").join(rel);
            let source = match std::fs::read_to_string(&path) {
                Ok(source) => source,
                Err(err) => {
                    failures.push(format!("{rel}: could not read input: {err}"));
                    continue;
                }
            };
            let (storage, roots) = match UbcaEvaluator.evaluate_arena(&source) {
                Ok(pair) => pair,
                Err(err) => {
                    failures.push(format!("{rel}: evaluation failed: {err}"));
                    continue;
                }
            };
            for root in roots {
                let rendered = Ubca2Sequencer::format(&storage, root, SequenceMode::Foolish);
                if let Err(err) = crate::fvm_storage::compose_program_with_system(
                    &mut crate::fvm_storage::FVMStorage::new(),
                    &rendered,
                ) {
                    failures.push(format!(
                        "{rel}: Foolish-mode rendering does not re-parse (Property 1 \
                         violated): {err}\n  rendered:\n{rendered}"
                    ));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "{} case(s) failed Property 1 — a parse failure is a renderer bug, never a \
             baseline problem:\n{}",
            failures.len(),
            failures.join("\n\n")
        );
    }
}
