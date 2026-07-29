use std::path::PathBuf;

use crate::evaluator::UbcaEvaluator;

/// Work directory of the einmo suite (FOOP-64).
#[cfg(test)]
fn einmo_suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("einmo_suite")
}

/// Einmo tests for the UBCa FVM.
///
/// Two gates at escalating validation levels (FOOP-64):
///
/// * [`run_einmo_tests`] — Checked level: output matches signed `checked/` baseline.
/// * `einmo_verified_gate` — Verified level: plus `verified/` under human key. (To come.)
#[cfg(test)]
mod einmo_tests {
    use super::*;
    use einmo::{EinmoSuite, Evaluator, Stage, TestConfig, ValidationLevel};

    /// Adapts the UBCa evaluator to einmo's language-agnostic `Evaluator`:
    /// one OUTPUT chunk per top-level statement, formatted by the humanizing
    /// sequencer — the same rendering the legacy corpus used.
    ///
    /// Constructed per call (not stored) because the FVM is `!Send`; the unit
    /// struct itself is trivially `Sync`, which `evaluate_all` requires.
    #[derive(Debug, Default, Clone, Copy)]
    struct UbcaEinmoAdapter;

    impl Evaluator for UbcaEinmoAdapter {
        fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
            use foolish_core::Evaluator as CoreEvaluator;
            let firs = UbcaEvaluator.evaluate(source)?;
            Ok(firs
                .iter()
                .map(|fir_ref| {
                    let fir = foolish_core::clone_steppable(fir_ref);
                    foolish_core::FirSequencer::format(&fir)
                })
                .collect())
        }
    }

    /// The suite config at a stated escalating level (FOOP-64 §"The escalating
    /// validation levels"). The level is required — einmo has no default.
    fn config(level: ValidationLevel) -> TestConfig {
        // The Foolish separator (`!!` + LF, a Foolish line comment) — Foolish
        // sources may contain einmo's default `①` glyph.
        TestConfig::new(einmo_suite_dir(), level).foolish_separator()
    }

    /// **Feature-complete test suite**: every input evaluates, is written and
    /// self-verifies in `output/`, and matches the signed `checked/` baseline.
    #[test]
    fn run_einmo_tests() {
        // The feature-complete test suite validates at the Checked level: a
        // reviewed, signed baseline that output must match. It says nothing
        // about verified/ — signing is the merge gate's business.
        let config =
            config(ValidationLevel::Checked).require_correspondence(Stage::Output, Stage::Checked);
        let suite = EinmoSuite::new(config);
        let results = suite
            .evaluate_all(&UbcaEinmoAdapter)
            .expect("evaluate_all must not fail at the filesystem level");

        // Anti-vacuity: a suite that discovered no inputs is a failure, not a
        // pass. (`compare --require-match` exits 0 on an empty tree — verified
        // 2026-07-14; FOOP-64 §"The escalating validation levels".)
        assert!(
            !results.files.is_empty(),
            "einmo suite discovered no inputs — check einmo_suite/input/"
        );

        for file in &results.files {
            assert!(
                file.written_and_verified,
                "{} was not written+verified: {:?}",
                file.rel_path.display(),
                file.detail
            );
        }

        // The suite's shape is judged at the level it declared: no extraneous
        // files, no orphaned artifacts, output <-> checked matching up exactly.
        // An editor swap file left in input/ reds the gate on purpose — a dirty
        // tree is not a clean baseline.
        assert!(
            results.integrity.is_clean(),
            "einmo_suite is not sound at the Checked level:\n{}",
            results.integrity.report()
        );

        assert!(
            results.correspondence_failures.is_empty(),
            "output does not match the signed checked/ baseline:\n  {}\n\
             Review the diff (`einmo compare output checked foolish-ubca/einmo_suite/`), then \
             either repair the code or promote after review \
             (`einmo promote output->checked foolish-ubca/einmo_suite/`).",
            results.correspondence_failures.join("\n  ")
        );
    }
}
