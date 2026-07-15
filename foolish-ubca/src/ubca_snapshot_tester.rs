use std::path::PathBuf;

use crate::evaluator::UbcaEvaluator;

fn suite() -> foolish_core::SnapshotSuite {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    foolish_core::SnapshotSuite::new(
        base.join("snapshot_tests").join("input"),
        base.join("snapshot_tests").join("approved"),
    )
}

/// Work directory of the einmo suite (FOOP-64).
#[cfg(test)]
fn einmo_suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("einmo_suite")
}

#[cfg(test)]
mod approval_tests {
    use super::*;

    /// LEGACY (insta). Retired by FOOP-64 once the einmo suite is the gate.
    ///
    /// Known red: generation stamps a wall-clock `generated:` timestamp *inside*
    /// the signed, byte-compared content, so a fresh run can never byte-match a
    /// stored corpus. That structural defect is precisely what the einmo suite
    /// below fixes (einmo's `compare` reads INPUT/OUTPUT only; stamps and
    /// metadata are excluded).
    #[test]
    #[ignore = "FOOP-64: structurally red (signature/timestamp churn); superseded by einmo_approval_all"]
    fn approval_all() {
        let eval = UbcaEvaluator;
        let suite = suite();
        let evaluations = suite.evaluate_all(num_cpus::get(), &eval);
        let approved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("snapshot_tests")
            .join("approved");
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(&approved);
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.bind(|| {
            for (name, result) in evaluations {
                eprintln!("Evaluating: {}", name);
                match result {
                    Ok(output) => {
                        insta::assert_snapshot!(format!("{}.foo", name), output);
                    }
                    Err(msg) => {
                        eprintln!("  ERROR: {}", msg);
                    }
                }
            }
        });
    }
}

/// The einmo suite: einmo is the harness, the UBCa FVM is the evaluator.
///
/// (Inverse of zweimomo, where the FVM is an evaluator used to test einmo.)
///
/// Two gates run against the one `einmo_suite/` directory, at escalating
/// validation levels (FOOP-64 §"The escalating validation levels"):
///
/// * [`einmo_approval_all`] — the **feature-complete test suite**, at the
///   **Checked** level: the suite is well-formed and evaluates (Output level),
///   *plus* output matches the signed `checked/` baseline. The computer key is
///   acceptable here; AI promotion `output->checked` after review is the einmo
///   design. It says nothing about `verified/`.
/// * `einmo_verified_gate` — the **merge-ready test suite**, escalating to the
///   **Verified** level: everything Checked requires, *plus* `verified/` matches
///   under the human reviewer's key with no computer-key attestation. (To come.)
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
    fn einmo_approval_all() {
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
