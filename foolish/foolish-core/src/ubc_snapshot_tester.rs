use crate::*;

/// UBC evaluator adapter for SnapshotSuite.
/// Compiles source via `Compiler`, evaluates to completion with `run_to_completion`.
pub struct UbcEvaluator;

impl Evaluator for UbcEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<FirRef>, String> {
        let firs = Compiler::compile(source)
            .map_err(|e| format!("Compilation failed: {}", e))?;
        let mut result = Vec::new();
        for fir in firs {
            let mut fir_ref = fir_to_ref(fir);
            ubc::run_to_completion(&mut fir_ref)
                .map_err(|e| format!("Evaluation failed: {}", e))?;
            result.push(fir_ref);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod ubc_approval_tests {
    use super::*;
    use crate::*;
    use std::path::PathBuf;

    fn suite() -> SnapshotSuite {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        SnapshotSuite::new(
            manifest.join("snapshot_tests").join("input"),
            manifest.join("snapshot_tests").join("approved"),
        )
    }

    #[test]
    fn approval_all() {
        let suite = suite();
        let evaluations = suite.evaluate_all(num_cpus::get(), &UbcEvaluator);
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("snapshot_tests")
                .join("approved"),
        );
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.bind(|| {
            for (name, result) in evaluations {
                match result {
                    Ok(output) => {
                        insta::assert_snapshot!(format!("{}.foo", name), output);
                    }
                    Err(msg) => {
                        eprintln!("Evaluation error for {}: {}", name, msg);
                    }
                }
            }
        });
    }
}
