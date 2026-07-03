use std::path::PathBuf;

use crate::evaluator::UbcaEvaluator;

fn suite() -> foolish_core::SnapshotSuite {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    foolish_core::SnapshotSuite::new(
        base.join("snapshot_tests").join("input"),
        base.join("snapshot_tests").join("approved"),
    )
}

#[cfg(test)]
mod approval_tests {
    use super::*;

    #[test]
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
