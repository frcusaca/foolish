use std::path::PathBuf;

use crate::evaluator::UbcaEvaluator;

/// Work directory of the einmo suite (FOOP-64).
#[cfg(test)]
fn einmo_suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("einmo_suite")
}

/// Einmo gate tests for the UBCa FVM.
///
/// Three gates at escalating validation levels (FOOP-64), each runnable
/// independently by CI to produce a per-gate status badge:
///
/// * [`einmo_gate_output`] — Output level: every input evaluates and
///   self-verifies in `output/`.
/// * [`einmo_gate_checked`] — Checked level: output matches signed `checked/`
///   baseline.
/// * [`einmo_gate_verified`] — Verified level: `checked/` matches `verified/`
///   under human reviewer key.
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

    /// **Output gate**: every input evaluates, is written and self-verifies
    /// in `output/`. No correspondence check — just that the FVM can run all
    /// inputs without crashing and produce valid signed artifacts.
    #[test]
    fn einmo_gate_output() {
        let config = config(ValidationLevel::Output);
        let suite = EinmoSuite::new(config);
        let results = suite
            .evaluate_all(&UbcaEinmoAdapter)
            .expect("evaluate_all must not fail at the filesystem level");

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

        assert!(
            results.integrity.is_clean(),
            "einmo_suite is not sound at the Output level:\n{}",
            results.integrity.report()
        );
    }

    /// **Checked gate**: output matches the signed `checked/` baseline.
    /// Escalates from Output level — evaluates all inputs, writes `output/`,
    /// then asserts byte-identical correspondence against `checked/`.
    #[test]
    fn einmo_gate_checked() {
        let config =
            config(ValidationLevel::Checked).require_correspondence(Stage::Output, Stage::Checked);
        let suite = EinmoSuite::new(config);
        let results = suite
            .evaluate_all(&UbcaEinmoAdapter)
            .expect("evaluate_all must not fail at the filesystem level");

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

    /// **Verified gate**: `checked/` matches `verified/` under human reviewer
    /// key. Escalates from Checked level — evaluates all inputs, asserts
    /// output↔checked correspondence, then asserts checked↔verified
    /// correspondence with human attestation.
    #[test]
    fn einmo_gate_verified() {
        let config = config(ValidationLevel::Verified)
            .require_correspondence(Stage::Output, Stage::Checked)
            .require_correspondence(Stage::Checked, Stage::Verified);
        let suite = EinmoSuite::new(config);
        let results = suite
            .evaluate_all(&UbcaEinmoAdapter)
            .expect("evaluate_all must not fail at the filesystem level");

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

        assert!(
            results.integrity.is_clean(),
            "einmo_suite is not sound at the Verified level:\n{}",
            results.integrity.report()
        );

        assert!(
            results.correspondence_failures.is_empty(),
            "correspondence failure:\n  {}\n\
             Review the diff (`einmo compare output checked foolish-ubca/einmo_suite/`), then \
             either repair the code or promote after review.",
            results.correspondence_failures.join("\n  ")
        );
    }
}
