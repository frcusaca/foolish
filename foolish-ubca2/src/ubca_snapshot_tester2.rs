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
}
